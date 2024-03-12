//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the layout ref of children of a node when the node's size changes.
///
/// Does nothing if a node has no children.
fn parent_update_reactor(
    parent_size : MutationEvent<NodeSize>,
    mut rc      : ReactCommands,
    sizes       : Query<(&Children, &React<NodeSize>)>,
    mut nodes   : Query<&mut React<SizeRef>>,
){
    let Some(node) = parent_size.read()
    else { tracing::error!("failed updating children layout refs, event is missing"); return; };
    let Ok((children, node_size)) = sizes.get(node) else { return; };

    // Update the children with the parent's size.
    let parent_size = **node_size;
    let parent_ref = SizeRef{ parent_size };

    for child in children.iter()
    {
        let Ok(mut layout_ref) = nodes.get_mut(*child) else { continue; };
        layout_ref.set_if_not_eq(&mut rc, parent_ref);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Refreshes the layout ref of a node from its parent.
fn parent_refresh_reactor(
    finish    : EntityEvent<FinishNode>,
    mut rc    : ReactCommands,
    mut nodes : Query<(&bevy::hierarchy::Parent, &mut React<SizeRef>)>,
    sizes     : Query<&React<NodeSize>>,
){
    let Some((node, _)) = finish.read()
    else { tracing::error!("failed updating parent layout ref, event is missing"); return; };
    let Ok((parent, mut layout_ref)) = nodes.get_mut(*node)
    else { tracing::debug!(?node, "failed updating parent layout ref, node is missing"); return; };
    let Ok(parent_size) = sizes.get(**parent)
    else { tracing::debug!(?node, "failed updating parent layout ref, parent node not found"); return; };

    // Update the target node with the parent's size.
    // - Note: Since we are refreshing, we don't use set_if_not_eq().
    let parent_size = **parent_size;
    let parent_ref = SizeRef{ parent_size };
    *layout_ref.get_mut(&mut rc) = parent_ref;
}

struct ParentRefreshReactor;
impl WorldReactor for ParentRefreshReactor
{
    type StartingTriggers = ();
    type Triggers = EntityEventTrigger<FinishNode>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(parent_refresh_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ParentUpdateReactor;
impl WorldReactor for ParentUpdateReactor
{
    type StartingTriggers = ();
    type Triggers = EntityMutationTrigger<NodeSize>;
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(parent_update_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for adding a UI node within a specific parent node.
///
/// The node is set as a child of the parent entity.
///
/// Adds `SpatialBundle`, `React<`[`NodeSize`]`>`, and `React<`[`SizeRef`]`>` to the node.
///
/// The node's `Transform` will be updated automatically if you use a [`Layout`] instruction.
//todo: need to validate that the node doesn't already have a parent (set_parent() just replaces the current parent)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Deref, DerefMut)]
pub struct Parent(pub Entity);

impl UiInstruction for Parent
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        let parent_entity = self.0;

        // Set this node as a child of the parent.
        rc.commands()
            .entity(node)
            .set_parent(parent_entity)
            .insert(SpatialBundle::default());

        // Prep entity.
        rc.insert(node, NodeSize::default());
        rc.insert(node, SizeRef::default());

        // Refresh the node's layout ref on node finish, and refresh children layouts on update.
        rc.commands().syscall(node,
            |
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut update  : Reactor<ParentUpdateReactor>,
                mut refresh : Reactor<ParentRefreshReactor>
            |
            {
                update.add_triggers(&mut rc, entity_mutation::<NodeSize>(node));
                refresh.add_triggers(&mut rc, entity_event::<FinishNode>(node));
            }
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ParentPlugin;

impl Plugin for ParentPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor(ParentUpdateReactor)
            .add_reactor(ParentRefreshReactor);
    }
}

//-------------------------------------------------------------------------------------------------------------------
