//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_cobweb::prelude::*;
use serde::Deserialize;

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

fn setup_block_primitive(mut rc: ReactCommands, mut meshes: ResMut<Assets<Mesh>>)
{
    rc.commands().insert_resource(
        BlockAssetCache{
            mesh   : Mesh2dHandle(meshes.add(Rectangle::new(1.0, 1.0))),
            colors : HashMap::default(),
        }
    );

    // Update block styles when their `Block` components are mutated.
    rc.on(mutation::<Block>(),
        |
            event         : MutationEvent<Block>,
            nodes         : Query<(&Children, &React<Block>)>,
            mut blocks    : Query<&mut Handle<ColorMaterial>>,
            mut cache     : ResMut<BlockAssetCache>,
            mut materials : ResMut<Assets<ColorMaterial>>
        |
        {
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
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn spawn_block(
    In((node, style)) : In<(Entity, Block)>,
    mut cache         : ResMut<BlockAssetCache>,
    mut rc            : ReactCommands,
    mut materials     : ResMut<Assets<ColorMaterial>>
){
    // Get material handle for the color.
    let color = cache.get_color_handle(style.color, &mut materials);

    // Spawn block entity as child of node.
    let block = rc.commands().spawn(
            MaterialMesh2dBundle{
                mesh     : cache.mesh.clone(),
                material : color.clone(),
                ..default()
            }
        )
        .set_parent(node)
        .id();

    // Scale the block mesh based on the parent's node size.
    let token = rc.on(entity_mutation::<NodeSize>(node),
        move |nodes: Query<&React<NodeSize>>, mut transforms: Query<&mut Transform>|
        {
            let Ok(node) = nodes.get(node)
            else { tracing::debug!(?node, "node missing for block size update"); return; };
            let Ok(mut transform) = transforms.get_mut(block)
            else { tracing::debug!(?block, "block missing for block size update"); return; };

            transform.scale = node.extend(0.);
        }
    );
    cleanup_reactor_on_despawn(&mut rc, node, token);
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// [`CobwebStyle`] primitive for creating single-color rectangular blocks.
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, Deserialize)]
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

pub(crate) struct PrimitivesPlugin;

impl Plugin for PrimitivesPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Block>()
            .add_systems(PreStartup, setup_block_primitive);
    }
}

//-------------------------------------------------------------------------------------------------------------------
