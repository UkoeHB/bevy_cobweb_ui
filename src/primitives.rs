//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_cobweb::prelude::*;

//standard shortcuts
use std::collections::HashMap;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct BlockAssetCache
{
    mesh: Mesh2dHandle,
    colors: HashMap<u32, Handle<ColorMaterial>>,
}

fn spawn_block(
    In((node, style)) : In<(Entity, BlockStyle)>,
    mut cache         : Local<Option<BlockAssetCache>>,
    mut rc            : ReactCommands,
    mut meshes        : ResMut<Assets<Mesh>>,
    mut materials     : ResMut<Assets<ColorMaterial>>
){
    // Get cached asset handles.
    if cache.is_none()
    {
        *cache = Some(BlockAssetCache{
            mesh   : Mesh2dHandle(meshes.add(Rectangle::new(1.0, 1.0))),
            colors : HashMap::default(),
        });
    }
    let cache = cache.as_mut().unwrap();
    let color = cache.colors.entry(style.color.as_rgba_u32()).or_insert_with(|| materials.add(style.color));

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

/// Style of the [`Block`] UI primitive.
#[derive(Copy, Clone)]
pub struct BlockStyle
{
    /// The color of the block.
    pub color: Color,
}

//-------------------------------------------------------------------------------------------------------------------

/// [`UiInstruction`] primitive for creating a single-color rectangular block.
pub struct Block
{
    /// The style of the block.
    pub style: BlockStyle,
}

impl Block
{
    /// Makes a new block.
    pub fn new(style: BlockStyle) -> Self
    {
        Self{ style }
    }
}

impl UiInstruction for Block
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        // Create a block.
        rc.commands().syscall((node, self.style), spawn_block);
    }
}

//-------------------------------------------------------------------------------------------------------------------
