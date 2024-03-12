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
    fn get_handle(&mut self, path: &String, asset_server: &mut AssetServer) -> Handle<Image>
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

/// Gets visibility based on avilability of the basic image.
///
/// If no image, then visibility is hidden, otherwise it's inherited.
fn get_basic_image_visibility(path: &str) -> Visibility
{
    match path
    {
        "" => Visibility::Hidden,
        _ => Visibility::Inherited,
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates basic image styles when their `BasicImage` components are mutated.
///
/// Sets the visibility to [`Visibility::Hidden`] if there is currently no image selected.
fn basic_image_style_reactor(
    event            : EntityEvent<FinishNode>,
    image            : MutationEvent<BasicImage>,
    mut nodes        : Query<(&mut Handle<Image>, &mut Visibility, &React<BasicImage>)>,
    mut cache        : ResMut<BasicImageAssetCache>,
    mut asset_server : ResMut<AssetServer>
){
    let Some(entity) = event.read().map(|(e, _)| e).cloned().or_else(|| image.read())
    else { tracing::error!("event missing for basic image primitive refresh"); return; };
    let Ok((mut texture, mut visibility, basic_image)) = nodes.get_mut(entity)
    else { tracing::debug!(?entity, "entity missing for basic image style refresh"); return; };

    *texture = cache.get_handle(&basic_image.path, &mut asset_server);

    // Disable visibility when waiting for a texture.
    let new_visibility = get_basic_image_visibility(basic_image.path.as_str());
    if *visibility != new_visibility { *visibility = new_visibility; }
}

struct BasicImageStyleReactor;
impl WorldReactor for BasicImageStyleReactor
{
    type StartingTriggers = MutationTrigger<BasicImage>;
    type Triggers = EntityEventTrigger<FinishNode>;
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
        rc.commands().entity(node).insert(SpriteBundle::default());
        rc.commands().syscall(node,
            |    
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut reactor : Reactor<BasicImageReactor>,
                mut refresh : Reactor<BasicImageStyleReactor>,
            |
            {
                reactor.add_triggers(&mut rc, entity_mutation::<NodeSize>(node));
                refresh.add_triggers(&mut rc, entity_event::<FinishNode>(node));
            }
        );
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
