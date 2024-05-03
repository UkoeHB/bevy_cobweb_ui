use crate::*;

use bevy::prelude::*;
use bevy::ecs::entity::Entities;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates [`Position2d`] whenever [`Justified`] changes on the same entity.
fn detect_justified(
    event         : MutationEvent<Justified>,
    mut c         : Commands,
    mut justified : Query<(&mut React<Position2d>, &React<Justified>)>
){
    let justified_entity = event.read().unwrap();
    let Ok((mut position, justified)) = justified.get_mut(justified_entity)
    else { tracing::debug!("position entity {:?} missing on justified position mutation", justified_entity); return; };

    position.set_if_neq(&mut c, Position2d::from(**justified));
}

struct DetectJustified;
impl WorldReactor for DetectJustified
{
    type StartingTriggers = MutationTrigger<Justified>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_justified) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn detect_position(
    mutation    : MutationEvent<Position2d>,
    entities    : &Entities,
    mut tracker : ResMut<DirtyNodeTracker>
){
    let entity = mutation.read().unwrap();
    if entities.get(entity).is_none() { return; }
    tracker.insert(entity);
}

struct DetectPosition2d;
impl WorldReactor for DetectPosition2d
{
    type StartingTriggers = MutationTrigger::<Position2d>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_position) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn compute_new_transform(
    sizeref   : SizeRef,
    nodesize  : NodeSize,
    position  : &Position2d,
    transform : &mut Mut<Transform>,
){
    // Get the offset between our node's anchor and the parent node's anchor.
    let parent_size = *sizeref;
    let size = *nodesize;
    let mut offset = position.offset(size, parent_size);

    // Convert the offset to a translation between the parent and node origins.
    // - Offset = [vector to parent top left corner]
    //          + [anchor offset vector (invert y)]
    //          + [node corner to node origin (invert y)]
    offset.x = (-parent_size.x / 2.) + offset.x + (size.x / 2.);
    offset.y = (parent_size.y / 2.) + -offset.y + (-size.y / 2.);

    // Update this node's transform.
    // - Avoid triggering change detection needlessly.
    let rotation = Quat::from_rotation_z(position.rotation);
    if transform.translation.x != offset.x { transform.translation.x = offset.x; }
    if transform.translation.y != offset.y { transform.translation.y = offset.y; }
    if transform.rotation      != rotation { transform.rotation      = rotation; }
}

//-------------------------------------------------------------------------------------------------------------------

/// Expresses the positioning reference of one axis of a node within another node.
///
/// Defaults to [`Self::Min`].
#[derive(Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Justify
{
    /// The node's minimum edge will align with the parent's minimum edge.
    /// - X-axis: left side
    /// - Y-axis: top side
    #[default]
    Min,
    /// The node's midpoint will align with the parent's midpoint.
    Center,
    /// The node's maximum edge will align with the parent's maximum edge.
    /// - X-axis: right side
    /// - Y-axis: bottom side
    Max,
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents the position of a rectangle within another rectangle.
///
/// This control's a 2d UI node's [`Transform`] using the node's
/// automatically-computed [`NodeSize`] and [`SizeRef`].
///
/// Mutating `Position2d` on a node will automatically mark it [dirty](DirtyNodeTracker) (but not inserting/removing it).
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position2d
{
    /// Justification of the node on the parent's x-axis.
    ///
    /// Defaults to the left side (x = 0.0).
    pub x_justify: Justify,
    /// Justification of the node on the parent's y-axis.
    ///
    /// Defaults to the top side (y = 0.0).
    pub y_justify: Justify,
    /// Horizontal offset from the node's anchor-point within its parent.
    pub x_offset: Val2d,
    /// Vertical offset from the node's anchor-point within its parent.
    pub y_offset: Val2d,
    /// The node's rotation around its z-axis in radians.
    ///
    /// Note that rotation is applied after other position calculations, and that the center of rotation is the node origin
    /// not the node anchor (i.e. the node's centerpoint, not the anchor defined by justify constraints).
    pub rotation: f32,
}

impl Position2d
{
    fn new_justified(x_justify: Justify, y_justify: Justify) -> Self
    {
        Self { x_justify, y_justify, ..Default::default() }
    }

    /// Creates a centered node, whose midpoint will be directly on top of the parent's midpoint.
    pub fn center() -> Self
    {
        Self::new_justified(Justify::Center, Justify::Center)
    }

    /// Creates a node justified to center-left.
    pub fn centerleft() -> Self
    {
        Self::new_justified(Justify::Min, Justify::Center)
    }

    /// Creates a node justified to center-right.
    pub fn centerright() -> Self
    {
        Self::new_justified(Justify::Max, Justify::Center)
    }

    /// Creates a node justified to top-left.
    pub fn topleft() -> Self
    {
        Self::new_justified(Justify::Min, Justify::Min)
    }

    /// Creates a node justified to top-center.
    pub fn topcenter() -> Self
    {
        Self::new_justified(Justify::Center, Justify::Min)
    }

    /// Creates a node justified to top-right.
    pub fn topright() -> Self
    {
        Self::new_justified(Justify::Max, Justify::Min)
    }

    /// Creates a node justified to bottom-left.
    pub fn bottomleft() -> Self
    {
        Self::new_justified(Justify::Min, Justify::Max)
    }

    /// Creates a node justified to bottom-center.
    pub fn bottomcenter() -> Self
    {
        Self::new_justified(Justify::Center, Justify::Max)
    }

    /// Creates a node justified to bottom-right.
    pub fn bottomright() -> Self
    {
        Self::new_justified(Justify::Max, Justify::Max)
    }

    /// Sets the horizontal offset.
    pub fn x_offset(mut self, offset: Val2d) -> Self
    {
        self.x_offset = offset;
        self
    }

    /// Sets the vertical offset.
    pub fn y_offset(mut self, offset: Val2d) -> Self
    {
        self.y_offset = offset;
        self
    }

    /// Sets the z-rotation in radians.
    pub fn rotation(mut self, rotation: f32) -> Self
    {
        self.rotation = rotation;
        self
    }

    /// Gets the offset between our node and the parent in 2D [`Transform`] coordinates.
    pub fn offset(&self, size: Vec2, parent_size: Vec2) -> Vec2
    {
        let size_x = fix_nan(size.x).max(0.);
        let size_y = fix_nan(size.y).max(0.);
        let parent_x = fix_nan(parent.x).max(0.);
        let parent_y = fix_nan(parent.y).max(0.);

        let mut x_offset = match self.x_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_x / 2.) - (size_x / 2.),
            Justify::Max    => parent_x - size_x,
        };
        x_offset += self.x_offset.compute(parent_x).max(0.);

        let mut y_offset = match self.y_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_y / 2.) - (size_y / 2.),
            Justify::Max    => parent_y - size_y,
        };
        y_offset += self.y_offset.compute(parent_y).max(0.);

        Vec2{ x: x_offset, y: y_offset }
    }
}

impl CobwebStyle for Position2d
{
    fn apply_style(&self, _rc: &mut ReactCommands, _node: Entity) { }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`CobwebStyle`] that wraps [`Position2d`] with simple justification-based settings.
///
/// Defaults to [`Self::Center`].
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Justified
{
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    #[default]
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl From<Justified> for Position2d
{
    fn from(justified: Justified) -> Self
    {
        match justified
        {
            Justified::TopLeft      => Position2d::topleft(),
            Justified::TopCenter    => Position2d::topcenter(),
            Justified::TopRight     => Position2d::topright(),
            Justified::CenterLeft   => Position2d::centerleft(),
            Justified::Center       => Position2d::center(),
            Justified::CenterRight  => Position2d::centerright(),
            Justified::BottomLeft   => Position2d::bottomleft(),
            Justified::BottomCenter => Position2d::bottomcenter(),
            Justified::BottomRight  => Position2d::bottomright(),
        }
    }
}

impl CobwebStyle for Justified
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        Position2d::from(*self).apply(rc, node);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct Position2dPlugin;

impl Plugin for Position2dPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<Position2d>()
            .register_type::<Justified>()
            .add_reactor_with(DetectJustified, mutation::<Justified>())
            .add_reactor_with(DetectPosition2d, mutation::<Position2d>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
