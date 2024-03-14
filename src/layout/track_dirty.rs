//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::entity::{Entities, EntityHashSet};
use bevy::hierarchy::HierarchyEvent;
use bevy_cobweb::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Marks nodes dirty if their parent changed.
//todo: mark frames dirty if their children change
fn track_hierarchy_changes(
    mut tracker : ResMut<DirtyNodeTracker>,
    mut events  : EventReader<HierarchyEvent>,
    nodes       : Query<(), With<CobwebNode>>,
){
    for event in events.read()
    {
        match *event
        {
            HierarchyEvent::ChildAdded{ child, .. } |
            HierarchyEvent::ChildRemoved{ child, .. } |
            HierarchyEvent::ChildMoved{ child, .. } =>
            {
                if nodes.contains(child) { tracker.insert(child); }
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn detect_new_nodes(
    event       : EntityEvent<NodeBuilt>,
    entities    : &Entities,
    mut tracker : ResMut<DirtyNodeTracker>
){
    let (entity, _) = event.read().unwrap();
    if entities.get(entity).is_none() { return; }
    tracker.insert(entity);
}

struct DetectNewNodes;
impl WorldReactor for DetectNewNodes
{
    type StartingTriggers = AnyEntityEventTrigger<NodeBuilt>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_new_nodes) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Tracks UI nodes that need layout updates this tick.
///
/// Use [`Self::insert`] to mark nodes dirty as needed.
///
/// Built-in [`UiInstructions`](UiInstruction) will automatically mark nodes dirty when their tracked components are
/// changed (e.g. [`Dims`], [`Position`], etc.).
///
/// Hierarchy changes are automatically detected, and the relevant nodes will be updated. Note that hierarchy changes
/// are only detected once per tick, so any reactions during a layout update that change the tree structure (re-parenting
/// nodes) won't necessarily be processed until the next tick.
#[derive(Resource, Default)]
pub struct DirtyNodeTracker
{
    dirty: EntityHashSet,
    list: Vec<Entity>,
}

impl DirtyNodeTracker
{
    /// Inserts a dirty entity to the tracker.
    pub fn insert(&mut self, entity: Entity)
    {
        let _ = self.dirty.insert(entity);
        self.list.push(entity);
    }

    /// Gets the number of dirty nodes.
    pub fn len(&self) -> usize
    {
        self.dirty.len()
    }

    /// Checks if an entity is currently marked dirty.
    pub fn contains(&mut self, entity: Entity) -> bool
    {
        self.dirty.contains(&entity)
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> bool
    {
        self.dirty.remove(&entity)
    }

    pub(crate) fn take_list(&mut self) -> Vec<Entity>
    {
        let mut temp = Vec::default();
        std::mem::swap(&mut self.list, &mut temp);
        temp
    }

    pub(crate) fn return_list(&mut self, list: Vec<Entity>)
    {
        debug_assert_eq!(self.list.len(), 0);
        self.list = list;
    }

    pub(crate) fn clear(&mut self)
    {
        self.dirty.clear();
        self.list.clear();
    }
}

//todo: Frame dirty on member insert, member dirty on insert

//todo: track insertions/removals/mutations of the following (ignore removals caused by despawns)
// - MinDims, MaxDims, NodeSizeAdjuster, Frame, InFrame, InFrameDerived

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct TrackDirtyPlugin;

impl Plugin for TrackDirtyPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<DirtyNodeTracker>()
            .add_reactor_with(DetectNewNodes, any_entity_event::<NodeBuilt>())
            .add_systems(PostUpdate,
                (
                    track_hierarchy_changes,
                ).in_set(LayoutSetPrep)
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
