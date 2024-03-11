//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates [`Position`] whenever [`Justified`] changes on the same entity.
fn justified_reactor(
    event         : MutationEvent<Justified>,
    mut rc        : ReactCommands,
    mut justified : Query<(&mut React<Position>, &React<Justified>)>
){
    let Some(justified_entity) = event.read()
    else { tracing::error!("justified layout mutation event missing"); return; };
    let Ok((mut layout, justified)) = justified.get_mut(justified_entity)
    else { tracing::debug!("layout entity {:?} missing on justified layout mutation", justified_entity); return; };
    layout.set_if_not_eq(&mut rc, Position::from(**justified));
}

struct JustifiedReactor;
impl WorldReactor for JustifiedReactor
{
    type StartingTriggers = MutationTrigger<Justified>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(justified_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates a node's transform on parent update or if the layout changes.
fn position_reactor(
    ref_event : MutationEvent<SizeRef>,
    lay_event : MutationEvent<Position>,
    mut rc    : ReactCommands,
    mut nodes : Query<(&mut Transform, &mut React<NodeSize>, &React<Position>, &React<SizeRef>)>
){
    let Some(node) = ref_event.read().or_else(|| lay_event.read())
    else { tracing::error!("failed running layout reactor, event is missing"); return; };
    let Ok((mut transform, mut size, layout, layout_ref)) = nodes.get_mut(node)
    else { tracing::debug!(?node, "node missing on layout update"); return; };

    // Get the offset between our node's anchor and the parent node's anchor.
    let parent_size = *layout_ref.parent_size;
    let mut offset = layout.offset(parent_size);
    let dims = layout.dims(parent_size);

    // Convert the offset to a translation between the parent and node origins.
    // - Offset = [vector to parent upper left corner]
    //          + [anchor offset vector (convert y)]
    //          + [node corner to node origin (convert y)]
    offset.x = (-parent_size.x / 2.) + offset.x + (dims.x / 2.);
    offset.y = (parent_size.y / 2.) + -offset.y + (-dims.y / 2.);

    // Update this node's transform.
    *transform = Transform::from_translation(offset.extend(*layout_ref.offset));
    transform.rotation = Quat::from_rotation_z(layout.rotation);

    // Update our node's size if it changed.
    size.set_if_not_eq(&mut rc, NodeSize(dims));
}

struct PositionReactor;
impl WorldReactor for PositionReactor
{
    type StartingTriggers = ();
    type Triggers = (EntityMutationTrigger<SizeRef>, EntityMutationTrigger<Position>);
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(position_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// The 2D dimensions of the node as a rectangle on the plane of the node's parent.
///
/// Translate by 1/2 of each dimension to reach the edges of the node parent.
#[derive(ReactComponent, Default, Debug, PartialEq, Copy, Clone, Deref, DerefMut)]
pub struct NodeSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

/// The z-offset applied between this node and its parent.
#[derive(ReactComponent, Debug, PartialEq, Copy, Clone, Deref, DerefMut, Default)]
pub struct NodeOffset(pub f32);

//-------------------------------------------------------------------------------------------------------------------

/// Reference data for use in defining the layout of a node.
#[derive(ReactComponent, Debug, PartialEq, Copy, Clone, Default)]
pub struct SizeRef
{
    /// The parent's node size.
    pub parent_size: NodeSize,
    /// The z-offset this node should use relative to its parent.
    pub offset: NodeOffset,
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

/// Represents a transformation between two rectangles.
///
/// When `Dims` is used as a [`UiInstruction`], this is used to transform the node's [`DimsRef`] into a
/// [`NodeSizeEstimate`].
///
/// `Dims` can also be wrapped in [`MinDims`] and [`MaxDims`] instructions, which will constrain the node's
/// [`NodeSizeEstimate`] and also its final [`NodeSize`] if it has a [`NodeSizeAdjuster`].
#[derive(Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Dims
{
    /// The node's width and height are absolute values in UI coordinates.
    Absolute(Vec2),
    /// The node's width and height are relative to the parents' width and height.
    ///
    /// Relative values are recorded in percentages.
    Relative(Vec2),
    /// The node's width and height equal the parent's width and height minus absolute padding values.
    ///
    /// Padding values are in UI coordinates. Positive padding will reduce the node size, while negative padding will
    /// increase it.
    ///
    /// Note that if padding is too large, your node may completely disappear.
    Padded(Vec2),
    /// The node's dimensions are fixed to a certain (x/y) ratio, and both dimensions are <= the parent's dimensions
    /// (with at least one dimension equal to the parent's corresponding dimension).
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidInFill((u32, u32)),
    /// The node's dimensions are fixed to a certain (x/y) ratio, and both dimensions are >= the parent's dimensions
    /// (with at least one dimension equal to the parent's corresponding dimension).
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidOutFill((u32, u32)),
    /// The parent's dimensions are adjusted by padding, then the node's width and height are computed from absolute
    /// values plus values relative to the adjusted parent.
    ///
    /// Relative values are recorded in percentages.
    Combined{
        #[reflect(default)]
        pad: Vec2,
        #[reflect(default)]
        abs: Vec2,
        #[reflect(default)]
        rel: Vec2
    },
    /// Equivalent to [`Self::SolidInFill`] applied to parent dimensions adjusted by [`Self::Combined`].
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidIn{
        ratio: (u32, u32),
        #[reflect(default)]
        pad: Vec2,
        #[reflect(default)]
        abs: Vec2,
        #[reflect(default)]
        rel: Vec2
    },
    /// Equivalent to [`Self::SolidOutFill`] applied to parent dimensions adjusted by [`Self::Combined`].
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidOut{
        ratio: (u32, u32),
        #[reflect(default)]
        pad: Vec2,
        #[reflect(default)]
        abs: Vec2,
        #[reflect(default)]
        rel: Vec2
    },
}

impl Dims
{
    /// Computes the dimensions of the node in 2D UI coordinates.
    pub fn compute(&self, parent_size: Vec2) -> Vec2
    {
        match *self
        {
            Self::Absolute(abs) =>
            {
                Vec2{
                    x: abs.x.max(0.),
                    y: abs.y.max(0.),
                }
            }
            Self::Relative(rel) =>
            {
                Vec2{
                    x: parent_size.x.max(0.) * rel.x.max(0.) / 100.,
                    y: parent_size.y.max(0.) * rel.y.max(0.) / 100.,
                }
            }
            Self::Padded(padding) =>
            {
                Vec2{
                    x: (parent_size.x - padding.x).max(0.),
                    y: (parent_size.y - padding.y).max(0.),
                }
            }
            Self::SolidInFill((ratio_x, ratio_y)) =>
            {
                let ratio_x = ratio_x.max(1) as f32;
                let ratio_y = ratio_y.max(1) as f32;
                let parent_x = parent_size.x.max(0.);
                let parent_y = parent_size.y.max(0.);

                // Case: this node is flatter than its parent.
                if (ratio_x * parent_y) >= (ratio_y * parent_x)
                {
                    Vec2{
                        x: parent_x,
                        y: parent_x * (ratio_y / ratio_x),
                    }
                }
                // Case: this node is thinner than its parent.
                else
                {
                    Vec2{
                        x: parent_y * (ratio_x / ratio_y),
                        y: parent_y,
                    }
                }
            }
            Self::SolidOutFill((ratio_x, ratio_y)) =>
            {
                let ratio_x = ratio_x.max(1) as f32;
                let ratio_y = ratio_y.max(1) as f32;
                let parent_x = parent_size.x.max(0.);
                let parent_y = parent_size.y.max(0.);

                // Case: this node is flatter than its parent.
                if (ratio_x * parent_y) >= (ratio_y * parent_x)
                {
                    Vec2{
                        x: parent_y * (ratio_x / ratio_y),
                        y: parent_y,
                    }
                }
                // Case: this node is thinner than its parent.
                else
                {
                    Vec2{
                        x: parent_x,
                        y: parent_x * (ratio_y / ratio_x),
                    }
                }
            }
            Self::Combined{ pad, abs, rel } =>
            {
                let parent_size = Self::Padded(pad).compute(parent_size);
                Self::Absolute(abs).compute(parent_size) + Self::Relative(rel).compute(parent_size)
            }
            Self::SolidIn{ ratio, pad, abs, rel } =>
            {
                let parent_size = Self::Combined{ pad, abs, rel }.compute(parent_size);
                Self::SolidInFill(ratio).compute(parent_size)
            }
            Self::SolidOut{ ratio, pad, abs, rel } =>
            {
                let parent_size = Self::Combined{ pad, abs, rel }.compute(parent_size);
                Self::SolidOutFill(ratio).compute(parent_size)
            }
        }
    }
}

impl Default for Dims
{
    fn default() -> Self
    {
        Self::Relative(Vec2::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Represents the position of a rectangle within a another rectangle.
///
/// When added as a [`UiInstruction`] to a node, this will be used to control the node's [`Transform`] using the
/// node's size and automatically-computed [`SizeRef`].
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position
{
    pub x_justify: Justify,
    pub y_justify: Justify,
    pub abs_offset: Vec2,
    pub rel_offset: Vec2,
    pub size: Dims,
    /// The node's rotation around its z-axis in radians.
    ///
    /// Note that rotation is applied after other layout calculations, and that the center of rotation is the node origin
    /// not the node anchor (i.e. the node's centerpoint, not its upper-left corner).
    pub rotation: f32,
}

impl Position
{
    fn new_justified(x_justify: Justify, y_justify: Justify, size: Dims) -> Self
    {
        Self {
            x_justify,
            y_justify,
            size,
            ..Default::default()
        }
    }

    /// Creates a node that perfectly overlaps its parent.
    pub fn overlay() -> Self
    {
        let mut overlay = Self::default();
        overlay.size = Dims::Padded(Vec2::default());
        overlay
    }

    /// Creates a centered node, whose midpoint will be directly on top of the parent's midpoint.
    pub fn centered(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Center, Justify::Center, size.into())
    }

    /// Creates a node justified to center-left.
    pub fn centerleft(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Min, Justify::Center, size.into())
    }

    /// Creates a node justified to center-right.
    pub fn centerright(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Max, Justify::Center, size.into())
    }

    /// Creates a node justified to upper-left.
    pub fn upperleft(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Min, Justify::Min, size.into())
    }

    /// Creates a node justified to upper-center.
    pub fn uppercenter(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Center, Justify::Min, size.into())
    }

    /// Creates a node justified to upper-right.
    pub fn upperright(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Max, Justify::Min, size.into())
    }

    /// Creates a node justified to lower-left.
    pub fn lowerleft(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Min, Justify::Max, size.into())
    }

    /// Creates a node justified to lower-center.
    pub fn lowercenter(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Center, Justify::Max, size.into())
    }

    /// Creates a node justified to lower-right.
    pub fn lowerright(size: impl Into<Dims>) -> Self
    {
        Self::new_justified(Justify::Max, Justify::Max, size.into())
    }

    /// Sets the relative offset.
    pub fn rel_offset(mut self, offset: Vec2) -> Self
    {
        self.rel_offset = offset;
        self
    }

    /// Sets the absolute offset.
    pub fn abs_offset(mut self, offset: Vec2) -> Self
    {
        self.abs_offset = offset;
        self
    }

    /// Sets the z-rotation in radians.
    pub fn rotation(mut self, rotation: f32) -> Self
    {
        self.rotation = rotation;
        self
    }

    /// Gets the dimensions of the node.
    pub fn dims(&self, parent_size: Vec2) -> Vec2
    {
        self.size.compute(parent_size)
    }

    /// Gets the offset between our node and the parent in 2D UI coordinates.
    pub fn offset(&self, parent_size: Vec2) -> Vec2
    {
        let dims = self.dims(parent_size);

        let mut x_offset = match self.x_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_size.x / 2.) - (dims.x / 2.),
            Justify::Max    => parent_size.x - dims.x,
        };
        x_offset += self.abs_offset.x;
        x_offset += self.rel_offset.x * parent_size.x.max(0.) / 100.;

        let mut y_offset = match self.y_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_size.y / 2.) - (dims.y / 2.),
            Justify::Max    => parent_size.y - dims.y,
        };
        y_offset += self.abs_offset.y;
        y_offset += self.rel_offset.y * parent_size.y.max(0.) / 100.;

        Vec2{ x: x_offset, y: y_offset }
    }
}

impl CobwebStyle for Position
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        rc.commands().syscall(node,
            |
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut reactor : Reactor<PositionReactor>,
            |
            {
                reactor.add_triggers(&mut rc, (entity_mutation::<SizeRef>(node), entity_mutation::<Position>(node)));
            }
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`CobwebStyle`] that wraps [`Position`] with simple justification-based settings.
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Justified
{
    UpperLeft(Dims),
    UpperCenter(Dims),
    UpperRight(Dims),
    CenterLeft(Dims),
    Center(Dims),
    CenterRight(Dims),
    LowerLeft(Dims),
    LowerCenter(Dims),
    LowerRight(Dims),
}

impl From<Justified> for Position
{
    fn from(justified: Justified) -> Self
    {
        match justified
        {
            Justified::UpperLeft(dims)   => Position::upperleft(dims),
            Justified::UpperCenter(dims) => Position::uppercenter(dims),
            Justified::UpperRight(dims)  => Position::upperright(dims),
            Justified::CenterLeft(dims)  => Position::centerleft(dims),
            Justified::Center(dims)      => Position::centered(dims),
            Justified::CenterRight(dims) => Position::centerright(dims),
            Justified::LowerLeft(dims)   => Position::lowerleft(dims),
            Justified::LowerCenter(dims) => Position::lowercenter(dims),
            Justified::LowerRight(dims)  => Position::lowerright(dims),
        }
    }
}

impl Default for Justified
{
    fn default() -> Self
    {
        Self::Center(Dims::default())
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

pub(crate) struct LayoutPlugin;

impl Plugin for LayoutPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Dims>()
            .register_type::<(u32, u32)>()
            .register_type::<Position>()
            .register_type::<Justified>()
            .add_reactor(PositionReactor)
            .add_reactor_with(JustifiedReactor, mutation::<Justified>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
