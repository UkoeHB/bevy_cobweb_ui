use std::collections::HashMap;

use bevy::prelude::Commands;
use bevy::reflect::TypeRegistry;

use super::*;
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_loadable(
    id_scratch: String,
    seen_shortnames: &mut Vec<&'static str>,
    type_registry: &TypeRegistry,
    caf_cache: &mut CobwebAssetCache,
    file: &CafFile,
    current_path: &ScenePath,
    loadable_index: usize,
    loadable: &CafLoadable,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
) -> String
{
    // Get the loadable's longname.
    let id_scratch = loadable.id.to_canonical(Some(id_scratch));
    let Some((short_name, long_name, type_id, deserializer)) =
        get_loadable_meta(type_registry, file, current_path, id_scratch.as_str(), name_shortcuts)
    else {
        return id_scratch;
    };

    // Check for duplicate.
    if seen_shortnames[..loadable_index]
        .iter()
        .any(|other| *other == short_name)
    {
        tracing::warn!("ignoring duplicate loadable {} at {:?} in {:?}", short_name, current_path, file);
        return id_scratch;
    }

    seen_shortnames.push(short_name);

    // Get the loadable's value.
    let loadable_value = get_loadable_value(deserializer, loadable);

    // Save this loadable.
    caf_cache.insert_loadable(
        &SceneRef {
            file: SceneFile::File(file.clone()),
            path: current_path.clone(),
        },
        loadable_index,
        loadable_value,
        type_id,
        long_name,
    );

    id_scratch
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_scene_node(
    id_scratch: String,
    seen_shortnames: &mut Vec<&'static str>,
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    scene_layer: &mut SceneLayer,
    scene: &SceneRef,
    parent_path: &ScenePath,
    caf_layer: &CafSceneLayer,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
) -> String
{
    let Some(node_path) = parent_path.extend_single(&*caf_layer.name) else {
        tracing::error!("failed parsing scene node {:?} at {:?} in {:?}, node ID is a multi-segment path, only \
            single-segment node ids are allowed in scene definitions", *caf_layer.name, parent_path, scene.file);
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
        caf_cache,
        scene_loader,
        child_layer,
        scene,
        &node_path,
        caf_layer,
        name_shortcuts,
    )
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_scene_layer(
    mut id_scratch: String,
    seen_shortnames: &mut Vec<&'static str>,
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    scene_layer: &mut SceneLayer,
    scene: &SceneRef,
    current_path: &ScenePath,
    caf_layer: &CafSceneLayer,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
) -> String
{
    // Prep the node.
    let scene_location = SceneRef { file: scene.file.clone(), path: current_path.clone() };
    caf_cache.prepare_scene_node(scene_location.clone());

    // Begin layer update.
    scene_layer.start_update(caf_layer.entries.len());

    // Add loadables.
    let mut loadable_count = 0;
    seen_shortnames.clear();

    for entry in caf_layer.entries.iter() {
        match entry {
            CafSceneLayerEntry::Loadable(loadable) => {
                id_scratch = handle_loadable(
                    id_scratch,
                    seen_shortnames,
                    type_registry,
                    caf_cache,
                    scene
                        .file
                        .file()
                        .expect("all SceneFile should contain CafFile in scene extraction"),
                    current_path,
                    loadable_count,
                    loadable,
                    name_shortcuts,
                );
                loadable_count += 1;
            }
            // Do this one after we are done using the `seen_shortnames` buffer.
            CafSceneLayerEntry::Layer(_) => (),
            CafSceneLayerEntry::LoadableMacroCall(_) => {
                tracing::warn!("ignoring loadable macro call in scene node {:?} in {:?}", current_path, scene.file);
            }
            CafSceneLayerEntry::SceneMacroCall(_) => {
                tracing::warn!("ignoring scene macro call in scene node {:?} in {:?}", current_path, scene.file);
            }
            CafSceneLayerEntry::SceneMacroParam(_) => {
                tracing::warn!("ignoring scene macro param in scene node {:?} in {:?}", current_path, scene.file);
            }
        }
    }

    #[cfg(feature = "hot_reload")]
    caf_cache.end_loadable_insertion(&scene_location, loadable_count);

    // Add layers.
    for entry in caf_layer.entries.iter() {
        match entry {
            CafSceneLayerEntry::Layer(next_caf_layer) => {
                id_scratch = handle_scene_node(
                    id_scratch,
                    seen_shortnames,
                    type_registry,
                    c,
                    caf_cache,
                    scene_loader,
                    scene_layer,
                    scene,
                    current_path,
                    next_caf_layer,
                    name_shortcuts,
                );
            }
            _ => (),
        }
    }

    // End layer update and handle removed nodes.
    for SceneLayerData { id, .. } in scene_layer.end_update() {
        #[cfg(feature = "hot_reload")]
        {
            scene_loader.cleanup_deleted_scene_node(c, scene, &id);
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

// TODO: handle anonymous nodes
// TODO: disallow duplicate node names, excluding anonymous nodes
pub(super) fn extract_scenes(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    file: &CafFile,
    section: &CafScenes,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let mut scene_registry = scene_loader.take_scene_registry();
    let mut id_scratch = String::default();
    let mut seen_shortnames = vec![];

    for caf_layer in section.scenes.iter() {
        // Get this scene for editing.
        let Some(path) = ScenePath::parse_single(&*caf_layer.name) else {
            tracing::error!("failed parsing scene {:?} in {:?}, scene root ID is a multi-segment path, only \
                single-segment node ids are allowed in scene definitions", *caf_layer.name, file);
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
            caf_cache,
            scene_loader,
            scene_layer,
            &scene_ref,
            &scene_ref.path,
            caf_layer,
            name_shortcuts,
        );
    }

    scene_loader.return_scene_registry(scene_registry);
}

//-------------------------------------------------------------------------------------------------------------------
