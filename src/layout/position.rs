//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy::ecs::entity::Entities;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates [`Position`] whenever [`Justified`] changes on the same entity.
fn detect_justified(
    event         : MutationEvent<Justified>,
    mut rc        : ReactCommands,
    mut justified : Query<(&mut React<Position>, &React<Justified>)>
){
    let justified_entity = event.read().unwrap();
    let Ok((mut position, justified)) = justified.get_mut(justified_entity)
    else { tracing::debug!("position entity {:?} missing on justified position mutation", justified_entity); return; };

    position.set_if_not_eq(&mut rc, Position::from(**justified));
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
    mutation    : MutationEvent<Position>,
    entities    : &Entities,
    mut tracker : ResMut<DirtyNodeTracker>
){
    let entity = mutation.read().unwrap();
    if entities.get(entity).is_none() { return; }
    tracker.insert(entity);
}

struct DetectPosition;
impl WorldReactor for DetectPosition
{
    type StartingTriggers = MutationTrigger::<Position>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_position) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn compute_new_transform(
    sizeref   : SizeRef,
    nodesize  : NodeSize,
    position  : &Position,
    transform : &mut Mut<Transform>,
){
    // Get the offset between our node's anchor and the parent node's anchor.
    let parent_size = *sizeref;
    let size = *nodesize;
    let mut offset = position.offset(size, parent_size);

    // Convert the offset to a translation between the parent and node origins.
    // - Offset = [vector to parent top left corner]
    //          + [anchor offset vector (convert y)]
    //          + [node corner to node origin (convert y)]
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
/// When added as a [`UiInstruction`] to a node, this will control the node's [`Transform`] using the node's
/// automatically-computed [`NodeSize`] and [`SizeRef`].
///
/// Mutating `Position` on a node will automatically mark it [dirty](DirtyNodeTracker) (but not inserting/removing it).
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position
{
    /// Justification of the node on the parent's x-axis.
    pub x_justify: Justify,
    /// Justification of the node on the parent's y-axis.
    pub y_justify: Justify,
    /// Offset from the node's anchor-point within its parent, in absolute UI coordinates.
    pub pixels: Vec2,
    /// Offset from the node's anchor-point within its parent, as percentages of the parent dimensions.
    pub percent: Vec2,
    /// The node's rotation around its z-axis in radians.
    ///
    /// Note that rotation is applied after other position calculations, and that the center of rotation is the node origin
    /// not the node anchor (i.e. the node's centerpoint, not the anchor defined by justify constraints).
    pub rotation: f32,
}

impl Position
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

    /// Sets the absolute offset.
    pub fn pixels(mut self, offset: Vec2) -> Self
    {
        self.pixels = offset;
        self
    }

    /// Sets the percentage offset.
    pub fn percent(mut self, offset: Vec2) -> Self
    {
        self.percent = offset;
        self
    }

    /// Sets the z-rotation in radians.
    pub fn rotation(mut self, rotation: f32) -> Self
    {
        self.rotation = rotation;
        self
    }

    /// Gets the offset between our node and the parent in 2D UI coordinates.
    pub fn offset(&self, size: Vec2, parent_size: Vec2) -> Vec2
    {
        let mut x_offset = match self.x_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_size.x / 2.) - (size.x / 2.),
            Justify::Max    => parent_size.x - size.x,
        };
        x_offset += self.pixels.x;
        x_offset += self.percent.x * parent_size.x.max(0.) / 100.;

        let mut y_offset = match self.y_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_size.y / 2.) - (size.y / 2.),
            Justify::Max    => parent_size.y - size.y,
        };
        y_offset += self.pixels.y;
        y_offset += self.percent.y * parent_size.y.max(0.) / 100.;

        Vec2{ x: x_offset, y: y_offset }
    }
}

impl CobwebStyle for Position
{
    fn apply_style(&self, _rc: &mut ReactCommands, _node: Entity) { }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`CobwebStyle`] that wraps [`Position`] with simple justification-based settings.
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

impl From<Justified> for Position
{
    fn from(justified: Justified) -> Self
    {
        match justified
        {
            Justified::TopLeft      => Position::topleft(),
            Justified::TopCenter    => Position::topcenter(),
            Justified::TopRight     => Position::topright(),
            Justified::CenterLeft   => Position::centerleft(),
            Justified::Center       => Position::center(),
            Justified::CenterRight  => Position::centerright(),
            Justified::BottomLeft   => Position::bottomleft(),
            Justified::BottomCenter => Position::bottomcenter(),
            Justified::BottomRight  => Position::bottomright(),
        }
    }
}

impl CobwebStyle for Justified
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        Position::from(*self).apply(rc, node);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct PositionPlugin;

impl Plugin for PositionPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .register_type::<Position>()
            .register_type::<Justified>()
            .add_reactor_with(DetectJustified, mutation::<Justified>())
            .add_reactor_with(DetectPosition, mutation::<Position>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
