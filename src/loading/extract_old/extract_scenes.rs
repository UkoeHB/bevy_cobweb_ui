use std::collections::HashMap;

use bevy::prelude::Commands;
use bevy::reflect::TypeRegistry;
use serde_json::{Map, Value};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_loadable(
    type_registry: &TypeRegistry,
    caf_cache: &mut CobwebAssetCache,
    file: &SceneFile,
    current_path: &ScenePath,
    short_name: &str,
    value: Value,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    // Get the loadable's longname.
    let Some((_short_name, long_name, type_id, deserializer)) =
        get_loadable_meta(type_registry, file, current_path, short_name, name_shortcuts)
    else {
        return;
    };

    // Get the loadable's value.
    let loadable_value = get_loadable_value(deserializer, value);

    // Save this loadable.
    caf_cache.insert_loadable(
        &SceneRef { file: file.clone(), path: current_path.clone() },
        loadable_value,
        type_id,
        long_name,
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_scene_node(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    scene_layer: &mut SceneLayer,
    scene: &SceneRef,
    parent_path: &ScenePath,
    key: &str,
    value: Value,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let Value::Object(data) = value else {
        tracing::error!("failed parsing scene node {:?} at {:?} in {:?}, node is not an Object",
            key, parent_path, scene.file);
        return;
    };

    let Some(node_path) = parent_path.extend_single(key) else {
        tracing::error!("failed parsing scene node {:?} at {:?} in {:?}, node ID is a multi-segment path, only \
            single-segment node ids are allowed in scene definitions", key, parent_path, scene.file);
        return;
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
    parse_scene_layer(
        type_registry,
        c,
        caf_cache,
        scene_loader,
        child_layer,
        scene,
        &node_path,
        data,
        name_shortcuts,
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn parse_scene_layer(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    scene_layer: &mut SceneLayer,
    scene: &SceneRef,
    current_path: &ScenePath,
    mut data: Map<String, Value>,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    // Prep the node.
    caf_cache.prepare_scene_node(SceneRef { file: scene.file.clone(), path: current_path.clone() });

    // Begin layer update.
    scene_layer.start_update(data.len());

    for (key, value) in data.iter_mut() {
        // Skip keyword map entries.
        if is_any_keyword(key) {
            continue;
        }

        // Remove qualifier from key if it has one.
        let key = match key.split_once('(') {
            Some((key, _)) => key,
            _ => key,
        };

        let value = value.take();

        if is_loadable_entry(key) {
            handle_loadable(
                type_registry,
                caf_cache,
                &scene.file,
                current_path,
                key,
                value,
                name_shortcuts,
            );
        } else {
            handle_scene_node(
                type_registry,
                c,
                caf_cache,
                scene_loader,
                scene_layer,
                scene,
                current_path,
                key,
                value,
                name_shortcuts,
            );
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
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn parse_scenes(
    type_registry: &TypeRegistry,
    c: &mut Commands,
    caf_cache: &mut CobwebAssetCache,
    scene_loader: &mut SceneLoader,
    file: &SceneFile,
    mut data: Map<String, Value>,
    name_shortcuts: &mut HashMap<&'static str, &'static str>,
)
{
    let mut scene_registry = scene_loader.take_scene_registry();

    for (key, value) in data.iter_mut() {
        // Skip keyword map entries.
        if is_any_keyword(key) {
            continue;
        }

        // Remove qualifier from key if it has one.
        let key = match key.split_once('(') {
            Some((key, _)) => key,
            _ => key,
        };

        // Reject loadables at the scene root layer.
        if is_loadable_entry(key) {
            tracing::error!("ignoring loadable {:?} in the base layer of {:?}, only scene root nodes are allowed in \
                the base layer of cobweb asset files", key, file);
            continue;
        }

        // Get this scene for editing.
        let Some(path) = ScenePath::parse_single(key) else {
            tracing::error!("failed parsing scene {:?} in {:?}, scene root ID is a multi-segment path, only \
                single-segment node ids are allowed in scene definitions", key, file);
            continue;
        };
        let scene_ref = SceneRef { file: file.clone(), path };
        let scene_layer = scene_registry.get_or_insert(scene_ref.clone());

        // Expect the scene to have a map in it.
        let value = value.take();
        let Value::Object(data) = value else {
            tracing::error!("failed parsing scene {:?}, content is not an Object", scene_ref);
            continue;
        };

        // Parse the scene.
        parse_scene_layer(
            type_registry,
            c,
            caf_cache,
            scene_loader,
            scene_layer,
            &scene_ref,
            &scene_ref.path,
            data,
            name_shortcuts,
        );
    }

    scene_loader.return_scene_registry(scene_registry);
}

//-------------------------------------------------------------------------------------------------------------------
