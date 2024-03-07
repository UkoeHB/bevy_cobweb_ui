//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct BlockAssetCache
{
    mesh: Mesh2dHandle,
    //todo: is this not mapping not precise?
    colors: HashMap<u32, Handle<ColorMaterial>>,
}

impl BlockAssetCache
{
    fn get_color_handle(&mut self, color: Color, materials: &mut Assets<ColorMaterial>) -> Handle<ColorMaterial>
    {
        self.colors.entry(color.as_rgba_u32()).or_insert_with(|| materials.add(color)).clone()
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Marker component for the child entity of block primitives that contains the block's mesh.
#[derive(Component)]
struct BlockMesh;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates block styles when their `Block` components are mutated.
fn block_reactor(
    event         : MutationEvent<Block>,
    nodes         : Query<(&Children, &React<Block>)>,
    mut blocks    : Query<&mut Handle<ColorMaterial>>,
    mut cache     : ResMut<BlockAssetCache>,
    mut materials : ResMut<Assets<ColorMaterial>>
){
    let Some(entity) = event.read()
    else { tracing::error!("entity mutation event missing for block primitive refresh"); return; };
    let Ok((children, block)) = nodes.get(entity)
    else { tracing::debug!(?entity, "entity missing for block primitive refresh"); return; };

    for child in children.iter()
    {
        let Ok(mut handle) = blocks.get_mut(*child) else { continue; };
        *handle = cache.get_color_handle(block.color, &mut materials);
        return;
    }
    tracing::warn!(?entity, "failed finding block child for updating style {:?}", **block);
}

struct BlockReactor;
impl WorldReactor for BlockReactor
{
    type StartingTriggers = MutationTrigger<Block>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(block_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates block mesh sizes when a block's node size updates.
fn block_mesh_reactor(
    event      : MutationEvent<NodeSize>,
    nodes      : Query<(&Children, &React<NodeSize>)>,
    mut blocks : Query<&mut Transform, With<BlockMesh>>
){
    let Some(node) = event.read()
    else { tracing::error!("block mesh reactor event did not fire as expected"); return; };
    let Ok((children, node_size)) = nodes.get(node)
    else { tracing::debug!(?node, "node missing for block size update"); return; };

    for child in children.iter()
    {
        let Ok(mut transform) = blocks.get_mut(*child) else { continue; };
        transform.scale = node_size.extend(0.);
    }
}

struct BlockMeshReactor;
impl WorldReactor for BlockMeshReactor
{
    type StartingTriggers = ();
    type Triggers = EntityMutationTrigger<NodeSize>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(block_mesh_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_block_primitive(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>)
{
    commands.insert_resource(
        BlockAssetCache{
            mesh   : Mesh2dHandle(meshes.add(Rectangle::new(1.0, 1.0))),
            colors : HashMap::default(),
        }
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn spawn_block(
    In((node, style)) : In<(Entity, Block)>,
    mut cache         : ResMut<BlockAssetCache>,
    mut rc            : ReactCommands,
    mut materials     : ResMut<Assets<ColorMaterial>>,
    mut reactor       : Reactor<BlockMeshReactor>,
){
    // Get material handle for the color.
    let color = cache.get_color_handle(style.color, &mut materials);

    // Spawn block entity as child of node.
    rc.commands().spawn(
            MaterialMesh2dBundle{
                mesh     : cache.mesh.clone(),
                material : color.clone(),
                ..default()
            }
        )
        .insert(BlockMesh)
        .set_parent(node);

    // Track the parent's node size
    reactor.add_triggers(&mut rc, entity_mutation::<NodeSize>(node));
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// [`CobwebStyle`] primitive for creating single-color rectangular blocks.
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block
{
    /// The color of the block.
    pub color: Color,
}

impl CobwebStyle for Block
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        // Create a block.
        rc.commands().syscall((node, *self), spawn_block);
    }
}

impl Default for Block
{
    fn default() -> Self
    {
        Self{ color: Color::NONE }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BlockPrimitivePlugin;

impl Plugin for BlockPrimitivePlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Block>()
            .add_reactor_with(BlockReactor, mutation::<Block>())
            .add_reactor(BlockMeshReactor)
            .add_systems(PreStartup, setup_block_primitive);
    }
}

//-------------------------------------------------------------------------------------------------------------------
