//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct BasicImageAssetCache
{
    images: HashMap<String, Handle<Image>>,
}

impl BasicImageAssetCache
{
    fn get_handle(&mut self, path: String, asset_server: &mut AssetServer) -> Handle<Image>
    {
        if path == "" { return Handle::default() }
        if let Some(handle) = self.images.get(&path) { return handle.clone(); }
        let handle = asset_server.load(&path);
        self.images.insert(path, handle.clone());
        handle
    }

    fn get_handle_by_ref(&mut self, path: &String, asset_server: &mut AssetServer) -> Handle<Image>
    {
        if path == "" { return Handle::default() }
        if let Some(handle) = self.images.get(path) { return handle.clone(); }
        let handle = asset_server.load(path);
        self.images.insert(path.clone(), handle.clone());
        handle
    }
}

impl Default for BasicImageAssetCache
{
    fn default() -> Self
    {
        Self{ images: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates basic image styles when their `BasicImage` components are mutated.
fn basic_image_style_reactor(
    event            : MutationEvent<BasicImage>,
    mut nodes        : Query<(&mut Handle<Image>, &React<BasicImage>)>,
    mut cache        : ResMut<BasicImageAssetCache>,
    mut asset_server : ResMut<AssetServer>
){
    let Some(entity) = event.read()
    else { tracing::error!("entity mutation event missing for basic image primitive refresh"); return; };
    let Ok((mut texture, basic_image)) = nodes.get_mut(entity)
    else { tracing::debug!(?entity, "entity missing for basic image primitive refresh"); return; };

    *texture = cache.get_handle_by_ref(&basic_image.path, &mut asset_server);
}

struct BasicImageStyleReactor;
impl WorldReactor for BasicImageStyleReactor
{
    type StartingTriggers = MutationTrigger<BasicImage>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(basic_image_style_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates basic image sprite on node size change.
fn basic_image_reactor(
    event     : MutationEvent<NodeSize>,
    mut nodes : Query<(&mut Sprite, &React<NodeSize>)>,
){
    let Some(node) = event.read()
    else { tracing::error!("basic image reactor event did not fire as expected"); return; };
    let Ok((mut sprite, node_size)) = nodes.get_mut(node)
    else { tracing::debug!(?node, "node missing for basic image size update"); return; };

    sprite.custom_size = Some(***node_size);
}

struct BasicImageReactor;
impl WorldReactor for BasicImageReactor
{
    type StartingTriggers = ();
    type Triggers = EntityMutationTrigger<NodeSize>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(basic_image_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn prepare_basic_image(
    In((node, style)) : In<(Entity, BasicImage)>,
    mut rc            : ReactCommands,
    mut asset_server  : ResMut<AssetServer>,
    mut cache         : ResMut<BasicImageAssetCache>,
    mut reactor       : Reactor<BasicImageReactor>,
){
    // Get image handle.
    let texture = cache.get_handle(style.path, &mut asset_server);

    // Insert image to the entity
    let Some(mut entity) = rc.commands().get_entity(node)
    else { tracing::debug!("failed spawning basic image, node entity {:?} is missing", node); return; };

    entity.insert(SpriteBundle{ texture, ..Default::default()});

    // Track the node size.
    reactor.add_triggers(&mut rc, entity_mutation::<NodeSize>(node));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// [`CobwebStyle`] primitive for adding basic images.
///
/// Adds a [`SpriteBundle`] to the node. Only [`Sprite::custom_size`] and the [`Handle<Image>`] component are controlled
/// by this style.
#[derive(ReactComponent, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasicImage
{
    /// An [`AssetPath`](bevy::asset::AssetPath) string for the image.
    pub path: String,
}

impl BasicImage
{
    /// Makes a new basic image.
    pub fn new(path: impl Into<String>) -> Self
    {
        Self{ path: path.into() }
    }
}

impl CobwebStyle for BasicImage
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        rc.commands().syscall((node, self.clone()), prepare_basic_image);
    }
}

impl Default for BasicImage
{
    fn default() -> Self
    {
        Self{ path: String::from("") }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BasicImagePrimitivePlugin;

impl Plugin for BasicImagePrimitivePlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<BasicImage>()
            .init_resource::<BasicImageAssetCache>()
            .add_reactor_with(BasicImageStyleReactor, mutation::<BasicImage>())
            .add_reactor(BasicImageReactor);
    }
}

//-------------------------------------------------------------------------------------------------------------------
