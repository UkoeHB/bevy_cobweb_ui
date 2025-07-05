use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy::render::camera::RenderTarget;
use bevy::window::{EnabledButtons, PrimaryWindow, WindowRef, WindowResolution, WindowTheme};
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui_core::editor::*;
use serde::de::DeserializeSeed;

use super::*;
use crate::cob::*;
use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default, Clone, Deref, DerefMut)]
struct EditorFileSelection(Option<CobFile>);

//-------------------------------------------------------------------------------------------------------------------

fn build_widgets(
    h: &mut UiSceneHandle,
    widgets: &CobWidgetRegistry,
    file_hash: CobFileHash,
    scene_ref: SceneRef,
    longname: &'static str,
    shortname: &'static str,
    loadable: Box<dyn PartialReflect + 'static>,
    death_signal: DeathSignal,
)
{
    // Check for loadable widget
    if let Some(spawn_fn) = widgets.get(longname) {
        let content_entity = h.id();
        let editor_ref = CobEditorRef {
            file_hash,
            scene_ref,
            loadable_name: shortname,
            structure_path: ReflectStructurePath { path: Arc::from([]) },
            death_signal,
        };
        let (loader, builder) = h.inner();
        if !(spawn_fn)(
            builder.commands(),
            loader,
            content_entity,
            &editor_ref,
            loadable.as_ref(),
        ) {
            h.spawn_scene_simple(("editor.frame", "destructure_unsupported"));
        }

        return;
    }

    // Fallback
    h.spawn_scene_simple(("editor.frame", "destructure_unsupported"));

    // TODO: Destructure and look for widgets for internal values
    // - TODO: If no widget found for an enum, provide a drop-down.
    // - TODO: If no widget found for a primitive type, serialize it to COB and display it directly.
    // - TODO: If no widget found for a set, display "<cannot destructure sets>" (you can only add/remove entries,
    //   TODO)
}

//-------------------------------------------------------------------------------------------------------------------

fn build_loadable(
    h: &mut UiSceneHandle,
    registry: &TypeRegistry,
    loadables: &LoadableRegistry,
    widgets: &CobWidgetRegistry,
    file_hash: CobFileHash,
    scene_ref: SceneRef,
    loadable: &CobLoadable,
)
{
    // Look up loadable type
    let name = loadable.id.to_canonical(None);
    let Some((deserializer, _, longname, shortname)) = get_deserializer(registry, name.as_str(), loadables) else {
        h.spawn_scene_simple(("editor.frame", "unsupported"));
        return;
    };

    // Build view
    h.spawn_scene(("editor.frame", "loadable"), |h| {
        // Set the loadable's name.
        h.get("name")
            .update(move |id: TargetId, mut e: TextEditor| {
                write_text!(e, id, "{}", shortname);
            });

        // Set the content.
        // TODO: reflection can fail because of internal constants; we may want those to be editable/inspectable,
        // but it requires destructuring the CobLoadable representation
        match deserializer.deserialize(loadable) {
            Ok(reflected) => {
                h.edit("content", |h| {
                    let (signaler, signal) = DeathSignaler::new();
                    h.insert(signaler);

                    build_widgets(h, widgets, file_hash, scene_ref, longname, shortname, reflected, signal);
                });
            }
            Err(_) => {
                h.get("content")
                    .spawn_scene_simple(("editor.frame", "reflect_fail"));
            }
        }
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_scene_simple_layer(
    h: &mut UiSceneHandle,
    registry: &TypeRegistry,
    loadables: &LoadableRegistry,
    widgets: &CobWidgetRegistry,
    file_hash: CobFileHash,
    scene_ref: SceneRef,
    layer: &CobSceneLayer,
)
{
    // Extend scene ref.
    let scene_ref = scene_ref + layer.name.as_str();

    // Build view
    h.spawn_scene(("editor.frame", "scene_node"), |h| {
        // Set node name.
        let ref_path = scene_ref.path.clone();
        h.get("name")
            .update(move |id: TargetId, mut e: TextEditor| {
                write_text!(e, id, "\"{}\"", ref_path.iter().rev().next().unwrap());
            });

        // Add entries.
        h.edit("content", |h| {
            for entry in layer.entries.iter() {
                match entry {
                    CobSceneLayerEntry::Loadable(loadable) => {
                        build_loadable(h, registry, loadables, widgets, file_hash, scene_ref.clone(), loadable);
                    }
                    CobSceneLayerEntry::Layer(scene_layer) => {
                        spawn_scene_simple_layer(
                            h,
                            registry,
                            loadables,
                            widgets,
                            file_hash,
                            scene_ref.clone(),
                            scene_layer,
                        );
                    }
                    _ => {
                        h.spawn_scene_simple(("editor.frame", "unsupported"));
                    }
                }
            }
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

fn build_file_view(In((base_entity, file)): In<(Entity, CobFile)>, mut c: Commands)
{
    // Build file view.
    // - We do this roundabout via a reactor in order to auto-rebuild when the file data changes.
    let mut ec = c.entity(base_entity);
    ec.update_on(
        (broadcast::<EditorFileExternalChange>(), broadcast::<EditorFileSaved>()),
        move |//
            _: TargetId,
            mut tracked_hash: Local<Option<CobFileHash>>,
            external_change: BroadcastEvent<EditorFileExternalChange>,
            file_saved: BroadcastEvent<EditorFileSaved>,
            mut c: Commands,
            mut s: SceneBuilder,
            registry: Res<AppTypeRegistry>,
            loadables: Res<LoadableRegistry>,
            widgets: Res<CobWidgetRegistry>,
            editor: Res<CobEditor>,//
        | {
            // If we are running this system because of an event, exit if the event targets a different file.
            if let Ok(external) = external_change.try_read() {
                if external.file != file {
                    return;
                }
            }
            if let Ok(file_saved) = file_saved.try_read() {
                // We watch for when the file gets saved. If saving a file makes a new file hash, then
                // existing widgets are invalid and need to be replaced.
                if file_saved.file != file || Some(file_saved.hash) == *tracked_hash {
                    return;
                }
            }

            // Clean up existing children.
            c.entity(base_entity).despawn_related::<Children>();

            // Look up file in editor to get file data.
            let Some(file_data) = editor.get_file(&file) else { return };

            // Handle non-editable files.
            // Note: these are filtered out by the dropdown but we handle it just in case.
            if !file_data.is_editable() {
                c.ui_builder(base_entity).spawn_scene_simple(("editor.frame", "file_not_editable"), &mut s);
                return;
            }

            // Save tracked hash, used to coordinate rebuilds on save.
            *tracked_hash = Some(file_data.last_save_hash);

            // Construct scene.
            let registry = registry.read();

            c.ui_builder(base_entity).spawn_scene(("editor.frame", "file_frame"), &mut s, |h| {
                // Commands section
                h.edit("commands", |h| {
                    let commands_ref = SceneRef{ file: file.clone().into(), path: ScenePath::new("#commands") };

                    for commands_section in file_data.data.sections.iter().filter_map(|s| {
                        let CobSection::Commands(commands) = s else { return None };
                        Some(commands)
                    }) {
                        for command in commands_section.entries.iter() {
                            build_loadable(
                                h,
                                &registry,
                                &loadables,
                                &widgets,
                                file_data.last_save_hash,
                                commands_ref.clone(),
                                &command.0,
                            );
                        }
                    }
                });

                // Scenes section
                h.edit("scenes", |h| {
                    let scene_ref = SceneRef{ file: file.clone().into(), path: ScenePath::empty() };

                    for scenes_section in file_data.data.sections.iter().filter_map(|s| {
                        let CobSection::Scenes(scenes) = s else { return None };
                        Some(scenes)
                    }) {
                        for scene_layer in scenes_section.scenes.iter() {
                            spawn_scene_simple_layer(
                                h,
                                &registry,
                                &loadables,
                                &widgets,
                                file_data.last_save_hash,
                                scene_ref.clone(),
                                scene_layer
                            );
                        }
                    }
                });
            });
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn build_editor_view(mut c: Commands, mut s: SceneBuilder, camera: Query<Entity, With<EditorCamera>>)
{
    let camera_entity = camera.single().unwrap();
    let scene = ("editor.frame", "base");

    c.ui_root().spawn_scene(scene, &mut s, |h| {
        // Editor is in a separate window.
        h.insert(UiTargetCamera(camera_entity));

        // Get content entity.
        let content_entity = h.get("content").id();

        // Build dropdown
        // TODO: use a proper dropdown widget that tracks selected automatically? (might be harder to get proper
        // CobFile value when selection is an opaque index)
        h.edit("dropdown", |h| {
            let dropdown_entity = h.id();

            // Core reactor for setting up content.
            h.on_event::<Option<CobFile>>().r(
                move |//
                    event: EntityEvent<Option<CobFile>>,
                    mut c: Commands,
                    mut selection: ResMut<EditorFileSelection>//
                | {
                    let (_, maybe_file) = event.read();

                    // Nothing to do if selection remains the same.
                    if **selection == *maybe_file { return }
                    **selection = maybe_file.clone();

                    // Clean up old content.
                    c.entity(content_entity).despawn_related::<Children>();

                    // Spawn new content.
                    let Some(file) = maybe_file else { return };
                    c.syscall((content_entity, file.clone()), build_file_view);
                },
            );

            // Handle dropdown opening.
            h.on_open(
                move |//
                    mut c: Commands,
                    mut s: SceneBuilder,
                    editor: Res<CobEditor>,
                    selection: Res<EditorFileSelection>//
                | {
                    // Despawn current options.
                    c.entity(dropdown_entity).despawn_related::<Children>();

                    // Add empty entry at the top. Pressing closes the dropdown.
                    let mut builder = c.ui_builder(dropdown_entity);
                    builder.spawn_scene(("editor.frame", "empty_dropdown_entry"), &mut s, |h| {
                        h.on_pressed(move |mut c: Commands| {
                            c.react().entity_event(dropdown_entity, Close);
                        });
                    });

                    // Get options and sort them lexicographically.
                    let mut entries: Vec<Option<CobFile>> =
                        editor.iter_files().filter(|(_, d)| d.is_editable()).map(|(f, _)| Some(f.clone())).collect();
                    entries.push(None);
                    entries.sort_unstable_by(|a, b| {
                        let a = a.as_ref().map(|f| f.as_str()).unwrap_or("");
                        let b = b.as_ref().map(|f| f.as_str()).unwrap_or("");
                        a.cmp(b)
                    });

                    for entry in entries {
                        builder.spawn_scene(("editor.frame", "dropdown_entry"), &mut s, |h| {
                            // Handle pressed.
                            let entry_clone = entry.clone();
                            h.on_pressed(move |mut c: Commands| {
                                // Close after updating selection so the on-close handler can use the right
                                // selection value.
                                c.react().entity_event(dropdown_entity, entry_clone.clone());
                                c.react().entity_event(dropdown_entity, Close);
                            });

                            // Set the option's text.
                            let entry_clone = entry.clone();
                            h.get("text").update(
                                move |id: TargetId, mut e: TextEditor| {
                                    let text = entry_clone.as_ref().map(|f| f.as_str()).unwrap_or("<none>");
                                    write_text!(e, id, "{}", text);
                                }
                            );

                            // Select current selection for proper styling.
                            if entry == **selection {
                                let entry_entity = h.id();
                                h.react().entity_event(entry_entity, Select);
                            }
                        });
                    }
                },
            );

            // Handle dropdown closing.
            h.on_close(
                move |mut c: Commands, mut s: SceneBuilder, selection: Res<EditorFileSelection>| {
                    // Despawn current options.
                    c.entity(dropdown_entity).despawn_related::<Children>();

                    // Spawn single option for the selected file.
                    c.ui_builder(dropdown_entity).spawn_scene(
                        ("editor.frame", "dropdown_entry"),
                        &mut s,
                        |h| {
                            // Set option as 'folded' for proper styling.
                            let entry_entity = h.id();
                            h.react().entity_event(entry_entity, Fold);

                            // Set the selection text.
                            let text: EditorFileSelection = selection.deref().clone();
                            h.get("text").update(
                                move |id: TargetId, mut e: TextEditor| {
                                    let text = text.as_ref().map(|f| f.as_str()).unwrap_or("<none>");
                                    write_text!(e, id, "{}", text);
                                }
                            );

                            // On pressed, open the dropdown.
                            h.on_pressed(move |mut c: Commands| {
                                c.react().entity_event(dropdown_entity, Open);
                            });
                        },
                    );
                },
            );

            // Refresh dropdown when the list changes.
            h.on_event::<EditorNewFile>()
                .r(move |mut c: Commands, p: PseudoStateParam| {
                    if p.entity_has(dropdown_entity, PseudoState::Open) {
                        c.react().entity_event(dropdown_entity, Close);
                        c.react().entity_event(dropdown_entity, Open);
                    }
                });

            // On EditorFileLost (TODO?)
            // - If currently-selected option is not in file list, then send empty file as entity event to self.
            // - if open, close and re-open

            // Initialize. Point to the "main.cob" file if there is one.
            // TODO: starting point should be obtained from EditorStack
            h.commands()
                .syscall_once((), move |mut c: Commands, editor: Res<CobEditor>| {
                    let file = CobFile::try_new("main.cob").unwrap();
                    let init = if editor.get_file(&file).is_some() {
                        Some(file)
                    } else {
                        None
                    };
                    c.react().entity_event(dropdown_entity, init);
                });

            // Initialize as Closed.
            h.react().entity_event(dropdown_entity, Close);
        });

        // Build unsaved indicator.
        // TODO: put an indicator on individual file names in the dropdown instead?
        let unsaved = h.get("footer::unsaved").id();
        h.react().on(
            (broadcast::<EditorFileUnsaved>(), broadcast::<EditorFileSaved>()),
            move |mut c: Commands, p: PseudoStateParam, editor: Res<CobEditor>| {
                if editor.any_unsaved() {
                    p.try_enable(&mut c, unsaved);
                } else {
                    p.try_disable(&mut c, unsaved);
                }
            },
        );
        h.react().entity_event(unsaved, Disable);

        // Build save button.
        // TODO: use CMD-S instead?
        h.get("footer::save")
            .on_pressed(|w: &mut World| SaveEditor.apply(w));
    });
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: try to make auto-moving the window smoother
// - winit supports 'child windows' which should solve these problems. Need to manually construct the window, see
//   WinitWindow::create_window.
// TODO: the editor's position does not sync with the window on startup until you move the window
// - maybe infer it from window starting size + monitor dimensions?
// TODO: the editor does not sync properly if you shrink the window from the top down
// TODO: give user option to make the editor be free-floating and resizable and spawn on the left side of
// the app
fn refresh_editor_window(
    primary_win: Query<&Window, (With<PrimaryWindow>, Without<EditorWindow>)>,
    mut editor_win: Query<&mut Window, (With<EditorWindow>, Without<PrimaryWindow>)>,
)
{
    let primary_window = primary_win.single().unwrap();
    let mut editor_window = editor_win.single_mut().unwrap();

    // Check if the editor's position needs to change.
    // TODO: incorporate MacOS 'content area' to avoid overlapping with the dock when on left side
    // - https://stackoverflow.com/a/42898625
    // - https://github.com/rustunit/bevy_device_lang/blob/main/src/apple.rs
    let WindowPosition::At(primary_pos) = primary_window.position else { return };
    let mut desired_pos = primary_pos;
    desired_pos.x -= (EDITOR_WIDTH * primary_window.resolution.scale_factor()) as i32;
    desired_pos.x = desired_pos.x.max(0);

    if WindowPosition::At(desired_pos) != editor_window.position {
        editor_window.position = WindowPosition::At(desired_pos);
    }

    // Check if the editor's height needs to change.
    let primary_height = primary_window.resolution.size().y;

    if primary_height != editor_window.resolution.size().y {
        editor_window.resolution.set(EDITOR_WIDTH, primary_height);
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: don't hard-code this, use a resource instead? maybe use resource on startup then allow resizes and
// any resize change gets saved back the to resource, which can then be saved back to user settings
const EDITOR_WIDTH: f32 = 300.0;

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the editor's window.
#[derive(Component, Debug)]
pub(crate) struct EditorWindow;

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the editor's camera.
#[derive(Component, Debug)]
pub(crate) struct EditorCamera;

//-------------------------------------------------------------------------------------------------------------------

/// Depends on bevy's `WindowPlugin`.
pub(crate) struct CobEditorBuildPlugin;

impl Plugin for CobEditorBuildPlugin
{
    fn build(&self, app: &mut App)
    {
        // Get primary window's starting height.
        let mut query = app
            .world_mut()
            .query_filtered::<&Window, With<PrimaryWindow>>();
        let primary_window = query.single(app.world()).unwrap();
        let initial_height = primary_window.resolution.size().y;

        // Make editor window.
        let mut resolution = WindowResolution::new(0., 0.);
        resolution.set(EDITOR_WIDTH, initial_height); // TODO: don't hard-code the width, and maybe don't fix height to primary?
        let editor_window = app
            .world_mut()
            .spawn((
                Window {
                    title: "Cob Editor".into(),
                    resolution,
                    resizable: false,
                    enabled_buttons: EnabledButtons { minimize: false, maximize: false, close: false },
                    window_theme: Some(WindowTheme::Dark), // TODO: don't hard-code this?
                    ..default()
                },
                EditorWindow,
            ))
            .id();

        // Add editor camera.
        // TODO: this camera needs to ignore non-UI entities, but render layers seems like an awkward solution
        app.world_mut().spawn((
            Camera2d,
            Camera {
                target: RenderTarget::Window(WindowRef::Entity(editor_window)),
                ..default()
            },
            EditorCamera,
        ));

        app.add_plugins(CobEditorTemplatePlugin)
            .init_resource::<EditorFileSelection>()
            .add_systems(First, refresh_editor_window)
            .add_systems(OnEnter(LoadState::Done), build_editor_view);
    }
}

//-------------------------------------------------------------------------------------------------------------------
