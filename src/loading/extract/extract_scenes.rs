use bevy::prelude::Commands;
use bevy::reflect::TypeRegistry;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_loadable(
    id_scratch: String,
    seen_shortnames: &mut Vec<&'static str>,
    type_registry: &TypeRegistry,
    scene_buffer: &mut SceneBuffer,
    file: &CobFile,
    current_path: &ScenePath,
    loadable: &mut CobLoadable,
    loadables: &LoadableRegistry,
    resolver: &CobLoadableResolver,
) -> String
{
    // Get the loadable's longname.
    let id_scratch = loadable.id.to_canonical(Some(id_scratch));
    let Some((short_name, long_name, type_id, deserializer)) =
        get_loadable_meta(type_registry, file, current_path, id_scratch.as_str(), loadables)
    else {
        return id_scratch;
    };

    // Check for duplicate.
    if seen_shortnames.iter().any(|other| *other == short_name) {
        tracing::warn!("ignoring duplicate loadable {} at {:?} in {:?}; use Multi<{}> instead",
            short_name, current_path, file, short_name);
        return id_scratch;
    }

    // Resolve defs.
    if let Err(err) = loadable.resolve(resolver) {
        tracing::warn!("failed extracting loadable {:?} at {:?} in {:?}; error resolving defs: {:?}",
            short_name, current_path, file, err.as_str());
        return id_scratch;
    }

    // Get the loadable's value.
    let loadable_value = get_loadable_value(deserializer, loadable);

    // Save this loadable.
    let loadable_index = seen_shortnames.len();
    seen_shortnames.push(short_name);

    scene_buffer.insert_loadable(
        &SceneRef {
            file: SceneFile::File(file.clone()),
            path: current_path.clone(),
        },
        Some(loadable_index),
        loadable_value,
        type_id,
        long_name,
    );

    id_scratch
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_scene_node(
    mut id_scratch: String,
    seen_shortnames: &mut Vec<&'static str>,
    type_registry: &TypeRegistry,
    c: &mut Commands,
    scene_buffer: &mut SceneBuffer,
    scene_loader: &mut SceneLoader,
    scene_layer: &mut SceneLayer,
    scene: &SceneRef,
    parent_path: &ScenePath,
    cob_layer: &mut CobSceneLayer,
    loadables: &LoadableRegistry,
    resolver: &mut CobResolver,
    anonymous_count: &mut usize,
) -> String
{
    // If node is anonymous, give it a unique name.
    let layer_name = if cob_layer.name.as_str() == "" {
        id_scratch.clear();
        let _ = write!(&mut id_scratch, "_{}", *anonymous_count);
        *anonymous_count += 1;
        id_scratch.as_str()
    } else {
        cob_layer.name.as_str()
    };

    let Some(node_path) = parent_path.extend_single(layer_name) else {
        tracing::error!("failed parsing scene node {:?} at {:?} in {:?}, node ID is a multi-segment path, only \
            single-segment node ids are allowed in scene definitions", layer_name, parent_path, scene.file);
        return id_scratch;
    };

    // Save this node in the scene.
    let child_layer = match scene_layer.insert(&node_path) {
        #[cfg(feature = "hot_reload")]
        SceneLayerInsertionResult::NoChange(child_layer) => child_layer,
        #[cfg(feature = "hot_reload")]
        SceneLayerInsertionResult::Updated(index, child_layer) => {
            scene_loader.handle_rearranged_scene_node(c, scene, parent_path, &node_path, index);
            child_layer
        }
        SceneLayerInsertionResult::Added(_index, child_layer) => {
            #[cfg(feature = "hot_reload")]
            {
                scene_loader.handle_inserted_scene_node(c, scene, parent_path, &node_path, _index);
            }
            child_layer
        }
    };

    // Parse the child layer of this node.
    extract_scene_layer(
        id_scratch,
        seen_shortnames,
        type_registry,
        c,
        scene_buffer,
        scene_loader,
        child_layer,
        scene,
        &node_path,
        cob_layer,
        loadables,
        resolver,
    )
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_scene_layer(
    mut id_scratch: String,
    seen_shortnames: &mut Vec<&'static str>,
    type_registry: &TypeRegistry,
    c: &mut Commands,
    scene_buffer: &mut SceneBuffer,
    scene_loader: &mut SceneLoader,
    scene_layer: &mut SceneLayer,
    scene: &SceneRef,
    current_path: &ScenePath,
    cob_layer: &mut CobSceneLayer,
    loadables: &LoadableRegistry,
    resolver: &mut CobResolver,
) -> String
{
    // Prep the node.
    let scene_location = SceneRef { file: scene.file.clone(), path: current_path.clone() };
    scene_buffer.prepare_scene_node(scene_location.clone());

    // Resolve the scene layer.
    if let Err(err) = cob_layer.resolve(resolver, SceneResolveMode::OneLayerSceneOnly) {
        tracing::warn!("failed extracting scene layer {:?} at {:?} in {:?}; error resolving defs: {:?}",
            cob_layer.name.as_str(), current_path, scene.file, err.as_str());
        return id_scratch;
    }

    // Begin layer update.
    scene_layer.start_update(cob_layer.entries.len());

    // Add loadables.
    seen_shortnames.clear();

    for entry in cob_layer.entries.iter_mut() {
        match entry {
            CobSceneLayerEntry::Loadable(loadable) => {
                id_scratch = handle_loadable(
                    id_scratch,
                    seen_shortnames,
                    type_registry,
                    scene_buffer,
                    scene
                        .file
                        .file()
                        .expect("all SceneFile should contain CobFile in scene extraction"),
                    current_path,
                    loadable,
                    loadables,
                    &resolver.loadables,
                );
            }
            // Do this one after we are done using the `seen_shortnames` buffer.
            CobSceneLayerEntry::Layer(_) => (),
            CobSceneLayerEntry::SceneMacroCommand(_) => {
                tracing::error!("ignoring unexpectedly unresolved scene macro command in scene layer {:?} at {:?} \
                    in {:?} (this is a bug)", cob_layer.name.as_str(), current_path, scene.file);
            }
            CobSceneLayerEntry::SceneMacroCall(_) => {
                tracing::error!("ignoring unexpectedly unresolved scene macro call in scene layer {:?} at {:?} \
                    in {:?} (this is a bug)", cob_layer.name.as_str(), current_path, scene.file);
            }
        }
    }

    #[cfg(feature = "hot_reload")]
    scene_buffer.end_loadable_insertion(&scene_location, seen_shortnames.len());

    // Add layers.
    let mut anonymous_count = 0;
    for entry in cob_layer.entries.iter_mut() {
        match entry {
            CobSceneLayerEntry::Layer(next_cob_layer) => {
                id_scratch = handle_scene_node(
                    id_scratch,
                    seen_shortnames,
                    type_registry,
                    c,
                    scene_buffer,
                    scene_loader,
                    scene_layer,
                    scene,
                    current_path,
                    next_cob_layer,
                    loadables,
                    resolver,
                    &mut anonymous_count,
                );
            }
            _ => (),
        }
    }

    // End layer update and handle removed nodes.
    for SceneLayerData { id, .. } in scene_layer.end_update() {
        #[cfg(feature = "hot_reload")]
        {
            scene_loader.cleanup_deleted_scene_node(c, scene_buffer, loadables, scene, &id);
        }
        #[cfg(not(feature = "hot_reload"))]
        {
            tracing::error!("scene node {:?} unexpectedly removed from {:?} while parsing scene (this is a bug)",
                id, scene);
        }
    }

    id_scratch
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: disallow duplicate node names, excluding anonymous nodes
pub(super) fn extract_scenes(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    scene_buffer: &mut SceneBuffer,
    scene_loader: &mut SceneLoader,
    file: &CobFile,
    section: &mut CobScenes,
    loadables: &LoadableRegistry,
    resolver: &mut CobResolver,
)
{
    let mut scene_registry = scene_loader.take_scene_registry();
    let mut id_scratch = String::default();
    let mut seen_shortnames = vec![];

    for cob_layer in section.scenes.iter_mut() {
        // Get this scene for editing.
        let Some(path) = ScenePath::parse_single(&*cob_layer.name) else {
            tracing::error!("failed parsing scene {:?} in {:?}, scene root ID is a multi-segment path, only \
                single-segment node ids are allowed in scene definitions", *cob_layer.name, file);
            continue;
        };
        let scene_ref = SceneRef { file: SceneFile::File(file.clone()), path };
        let scene_layer = scene_registry.get_or_insert(scene_ref.clone());

        // Parse the scene.
        id_scratch = extract_scene_layer(
            id_scratch,
            &mut seen_shortnames,
            type_registry,
            c,
            scene_buffer,
            scene_loader,
            scene_layer,
            &scene_ref,
            &scene_ref.path,
            cob_layer,
            loadables,
            resolver,
        );
    }

    scene_loader.return_scene_registry(scene_registry);
}

//-------------------------------------------------------------------------------------------------------------------
