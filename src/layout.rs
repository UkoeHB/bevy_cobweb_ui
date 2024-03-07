//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates [`Layout`] whenever [`JustifiedLayout`] changes on the same entity.
fn justified_layout_reactor(
    event         : MutationEvent<JustifiedLayout>,
    mut rc        : ReactCommands,
    mut justified : Query<(&mut React<Layout>, &React<JustifiedLayout>)>
){
    let Some(justified_entity) = event.read()
    else { tracing::error!("justified layout mutation event missing"); return; };
    let Ok((mut layout, justified)) = justified.get_mut(justified_entity)
    else { tracing::debug!("layout entity {:?} missing on justified layout mutation", justified_entity); return; };
    layout.set_if_not_eq(&mut rc, Layout::from(**justified));
}

struct JustifiedLayoutReactor;
impl WorldReactor for JustifiedLayoutReactor
{
    type StartingTriggers = MutationTrigger<JustifiedLayout>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(justified_layout_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates a node's transform on parent update or if the layout changes.
fn layout_reactor(
    ref_event : MutationEvent<LayoutRef>,
    lay_event : MutationEvent<Layout>,
    mut rc    : ReactCommands,
    mut nodes : Query<(&mut Transform, &mut React<NodeSize>, &React<Layout>, &React<LayoutRef>)>
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

struct LayoutReactor;
impl WorldReactor for LayoutReactor
{
    type StartingTriggers = ();
    type Triggers = (EntityMutationTrigger<LayoutRef>, EntityMutationTrigger<Layout>);
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(layout_reactor) }
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
pub struct LayoutRef
{
    /// The parent's node size.
    pub parent_size: NodeSize,
    /// The z-offset this node should use relative to its parent.
    pub offset: NodeOffset,
}

//-------------------------------------------------------------------------------------------------------------------

/// Expresses the positioning reference of one axis of a node within another node.
#[derive(Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Justify
{
    /// The node's minimum edge will align with the parent's minimum edge.
    /// - X-axis: left side
    /// - Y-axis: top side
    Min,
    /// The node's midpoint will align with the parent's midpoint.
    #[default]
    Center,
    /// The node's maximum edge will align with the parent's maximum edge.
    /// - X-axis: right side
    /// - Y-axis: bottom side
    Max,
}

//-------------------------------------------------------------------------------------------------------------------

/// The size of a node relative to its parent.
#[derive(Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Size
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
    /// The node's width and height are computed from absolute values plus values relative to the parent.
    ///
    /// Relative values are recorded in percentages.
    Combined{ abs: Vec2, rel: Vec2 },
    /// The node's dimensions are fixed to a certain (x/y) ratio, and both dimensions are <= the parent's dimensions
    /// (with at least one dimension equal to the parent's corresponding dimension).
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidIn((u32, u32)),
    /// The node's dimensions are fixed to a certain (x/y) ratio, and both dimensions are >= the parent's dimensions
    /// (with at least one dimension equal to the parent's corresponding dimension).
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidOut((u32, u32)),
    /// The same as [`Self::SolidIn`] except parent dimensions are adusted by `abs` and `rel` before computing the size.
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidInCombined{ ratio: (u32, u32), abs: Vec2, rel: Vec2 },
    /// The same as [`Self::SolidOut`] except parent dimensions are adusted by `abs` and `rel` before computing the size.
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidOutCombined{ ratio: (u32, u32), abs: Vec2, rel: Vec2 },
}

impl Size
{
    /// Computes the dimensions of the node in 2D UI coordinates.
    pub fn compute(&self, parent_dims: Vec2) -> Vec2
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
                    x: parent_dims.x.max(0.) * rel.x.max(0.) / 100.,
                    y: parent_dims.y.max(0.) * rel.y.max(0.) / 100.,
                }
            }
            Self::Padded(padding) =>
            {
                Vec2{
                    x: (parent_dims.x - padding.x).max(0.),
                    y: (parent_dims.y - padding.y).max(0.),
                }
            }
            Self::Combined{ abs, rel } =>
            {
                Self::Absolute(abs).compute(parent_dims) + Self::Relative(rel).compute(parent_dims)
            }
            Self::SolidIn((ratio_x, ratio_y)) =>
            {
                let ratio_x = ratio_x.max(1) as f32;
                let ratio_y = ratio_y.max(1) as f32;
                let parent_x = parent_dims.x.max(0.);
                let parent_y = parent_dims.y.max(0.);

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
            Self::SolidOut((ratio_x, ratio_y)) =>
            {
                let ratio_x = ratio_x.max(1) as f32;
                let ratio_y = ratio_y.max(1) as f32;
                let parent_x = parent_dims.x.max(0.);
                let parent_y = parent_dims.y.max(0.);

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
            Self::SolidInCombined{ ratio, abs, rel } =>
            {
                let parent_dims = Self::Absolute(abs).compute(parent_dims) + Self::Relative(rel).compute(parent_dims);
                Self::SolidIn(ratio).compute(parent_dims)
            }
            Self::SolidOutCombined{ ratio, abs, rel } =>
            {
                let parent_dims = Self::Absolute(abs).compute(parent_dims) + Self::Relative(rel).compute(parent_dims);
                Self::SolidOut(ratio).compute(parent_dims)
            }
        }
    }
}

impl Default for Size
{
    fn default() -> Self
    {
        Self::Relative(Vec2::default())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A layout component for UI nodes.
///
/// This should be added to nodes as a [`UiInstruction`].
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout
{
    pub x_justify: Justify,
    pub y_justify: Justify,
    pub abs_offset: Vec2,
    pub rel_offset: Vec2,
    pub size: Size,
    /// The node's rotation around its z-axis in radians.
    ///
    /// Note that rotation is applied after other layout calculations, and that the center of rotation is the node origin
    /// not the node anchor (i.e. the node's centerpoint, not its upper-left corner).
    pub rotation: f32,
}

impl Layout
{
    fn new_justified(x_justify: Justify, y_justify: Justify, size: Size) -> Self
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
        overlay.size = Size::Padded(Vec2::default());
        overlay
    }

    /// Creates a centered node, whose midpoint will be directly on top of the parent's midpoint.
    pub fn centered(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Center, Justify::Center, size.into())
    }

    /// Creates a node justified to center-left.
    pub fn centerleft(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Min, Justify::Center, size.into())
    }

    /// Creates a node justified to center-right.
    pub fn centerright(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Max, Justify::Center, size.into())
    }

    /// Creates a node justified to upper-left.
    pub fn upperleft(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Min, Justify::Min, size.into())
    }

    /// Creates a node justified to upper-center.
    pub fn uppercenter(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Center, Justify::Min, size.into())
    }

    /// Creates a node justified to upper-right.
    pub fn upperright(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Max, Justify::Min, size.into())
    }

    /// Creates a node justified to lower-left.
    pub fn lowerleft(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Min, Justify::Max, size.into())
    }

    /// Creates a node justified to lower-center.
    pub fn lowercenter(size: impl Into<Size>) -> Self
    {
        Self::new_justified(Justify::Center, Justify::Max, size.into())
    }

    /// Creates a node justified to lower-right.
    pub fn lowerright(size: impl Into<Size>) -> Self
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
    pub fn dims(&self, parent_dims: Vec2) -> Vec2
    {
        self.size.compute(parent_dims)
    }

    /// Gets the offset between our node and the parent in 2D UI coordinates.
    pub fn offset(&self, parent_dims: Vec2) -> Vec2
    {
        let dims = self.dims(parent_dims);

        let mut x_offset = match self.x_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_dims.x / 2.) - (dims.x / 2.),
            Justify::Max    => parent_dims.x - dims.x,
        };
        x_offset += self.abs_offset.x;
        x_offset += self.rel_offset.x * parent_dims.x.max(0.) / 100.;

        let mut y_offset = match self.y_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_dims.y / 2.) - (dims.y / 2.),
            Justify::Max    => parent_dims.y - dims.y,
        };
        y_offset += self.abs_offset.y;
        y_offset += self.rel_offset.y * parent_dims.y.max(0.) / 100.;

        Vec2{ x: x_offset, y: y_offset }
    }
}

impl CobwebStyle for Layout
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        rc.commands().syscall(node,
            |
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut reactor : Reactor<LayoutReactor>,
            |
            {
                reactor.add_triggers(&mut rc, (entity_mutation::<LayoutRef>(node), entity_mutation::<Layout>(node)));
            }
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`CobwebStyle`] that wraps [`Layout`] with simple justification-based settings.
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum JustifiedLayout
{
    UpperLeft(Size),
    UpperCenter(Size),
    UpperRight(Size),
    CenterLeft(Size),
    Center(Size),
    CenterRight(Size),
    LowerLeft(Size),
    LowerCenter(Size),
    LowerRight(Size),
}

impl From<JustifiedLayout> for Layout
{
    fn from(justified: JustifiedLayout) -> Self
    {
        match justified
        {
            JustifiedLayout::UpperLeft(size)      => Layout::upperleft(size),
            JustifiedLayout::UpperCenter(size)    => Layout::uppercenter(size),
            JustifiedLayout::UpperRight(size)     => Layout::upperright(size),
            JustifiedLayout::CenterLeft(size)   => Layout::centerleft(size),
            JustifiedLayout::Center(size)       => Layout::centered(size),
            JustifiedLayout::CenterRight(size)  => Layout::centerright(size),
            JustifiedLayout::LowerLeft(size)   => Layout::lowerleft(size),
            JustifiedLayout::LowerCenter(size) => Layout::lowercenter(size),
            JustifiedLayout::LowerRight(size)  => Layout::lowerright(size),
        }
    }
}

impl Default for JustifiedLayout
{
    fn default() -> Self
    {
        Self::Center(Size::default())
    }
}

impl CobwebStyle for JustifiedLayout
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        Layout::from(*self).apply(rc, node);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LayoutPlugin;

impl Plugin for LayoutPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Size>()
            .register_type::<(u32, u32)>()
            .register_type::<Layout>()
            .register_type::<JustifiedLayout>()
            .add_reactor(LayoutReactor)
            .add_reactor_with(JustifiedLayoutReactor, mutation::<JustifiedLayout>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
