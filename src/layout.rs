//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn setup_justified_layout(mut rc: ReactCommands)
{
    // Update Layout whenever JustifiedLayout changes on the same entity.
    rc.on(mutation::<JustifiedLayout>(),
        |
            mut rc        : ReactCommands,
            event         : MutationEvent<JustifiedLayout>,
            mut justified : Query<(&mut React<Layout>, &React<JustifiedLayout>)>
        |
        {
            let Some(justified_entity) = event.read()
            else { tracing::error!("justified layout mutation event missing"); return; };
            let Ok((mut layout, justified)) = justified.get_mut(justified_entity)
            else { tracing::debug!("layout entity {:?} missing on justified layout mutation", justified_entity); return; };
            *layout.get_mut(&mut rc) = Layout::from(**justified);
        }
    );
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// The 2D dimensions of the node as a rectangle on the plane of the node's parent.
///
/// Translate by 1/2 of each dimension to reach the edges of the node parent.
#[derive(ReactComponent, Default, Debug, Copy, Clone, Deref, DerefMut)]
pub struct NodeSize(pub Vec2);

//-------------------------------------------------------------------------------------------------------------------

/// The z-offset applied between this node and its parent.
#[derive(ReactComponent, Debug, Copy, Clone, Deref, DerefMut)]
pub struct NodeOffset(pub f32);

//-------------------------------------------------------------------------------------------------------------------

/// A parent node's updated layout information.
#[derive(Debug, Copy, Clone)]
pub struct ParentUpdate
{
    /// The parent's node size.
    pub size: NodeSize,
    /// The z-offset that children should use relative to the parent.
    pub child_offset: NodeOffset,
}

//-------------------------------------------------------------------------------------------------------------------

/// Expresses the positioning reference of one axis of a node within another node.
#[derive(Reflect, Default, Debug, Copy, Clone, Eq, PartialEq, Deserialize)]
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

/// Relative dimensions.
#[derive(Debug, Copy, Clone, Deref, DerefMut)]
pub struct Relative(pub Vec2);

impl Relative
{
    /// Creates a new relative dimensions from x and y dimensions.
    pub fn new(x: f32, y: f32) -> Self
    {
        Self(Vec2{ x, y})
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// The size of a node relative to its parent.
#[derive(Reflect, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Size
{
    /// The node's width and height are relative to the parents' width and height.
    ///
    /// Relative values are recorded in percentages.
    Relative(Vec2)
}

impl Size
{
    /// Computes the dimensions of the node in 2D UI coordinates.
    pub fn compute(&self, parent_dims: Vec2) -> Vec2
    {
        match self
        {
            Self::Relative(rel) =>
            {
                Vec2{
                    x: parent_dims.x.max(0.) * rel.x.max(0.) / 100.,
                    y: parent_dims.y.max(0.) * rel.y.max(0.) / 100.,
                }
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

impl From<Relative> for Size
{
    fn from(rel: Relative) -> Self
    {
        Self::Relative(rel.0)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A layout component for UI nodes.
///
/// This should be added to nodes as a [`UiInstruction`].
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, Deserialize)]
pub struct Layout
{
    pub x_justify: Justify,
    pub y_justify: Justify,
    pub offset_abs: Vec2,
    pub offset_rel: Vec2,
    pub size: Size,
    /// The node's rotation around its z-axis in radians.
    ///
    /// Note that rotation is applied after other layout calculations, and that the center of rotation is the node origin
    /// not the node anchor (i.e. the node's centerpoint, not its upper-left corner).
    pub z_rotation: f32,
}

impl Layout
{
    /// Creates a centered node, whose midpoint will be directly on top of the parent's midpoint.
    pub fn centered(size: impl Into<Size>) -> Self
    {
        Self {
            x_justify  : Justify::Center,
            y_justify  : Justify::Center,
            offset_abs : Vec2::default(),
            offset_rel : Vec2::default(),
            size       : size.into(),
            z_rotation : 0.,
        }
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
        x_offset += self.offset_abs.x;
        x_offset += self.offset_rel.x * parent_dims.x.max(0.) / 100.;

        let mut y_offset = match self.y_justify
        {
            Justify::Min    => 0.,
            Justify::Center => (parent_dims.y / 2.) - (dims.y / 2.),
            Justify::Max    => parent_dims.y - dims.y,
        };
        y_offset += self.offset_abs.y;
        y_offset += self.offset_rel.y * parent_dims.y.max(0.) / 100.;

        Vec2{ x: x_offset, y: y_offset }
    }
}

impl CobwebStyle for Layout
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        // Update the node's transform on parent update or if the layout changes.
        let token = rc.on((entity_event::<ParentUpdate>(node), entity_mutation::<Layout>(node)),
            move
            |
                mut cache : Local<Option<ParentUpdate>>,
                mut rc    : ReactCommands,
                update    : EntityEvent<ParentUpdate>,
                mut nodes : Query<(&mut Transform, &mut React<NodeSize>, &React<Layout>)>
            |
            {
                if let Some((_, update)) = update.read() { *cache = Some(*update); }
                let Some(parent) = &*cache else { return; };
                let Ok((mut transform, mut size, layout)) = nodes.get_mut(node)
                else { tracing::debug!(?node, "node missing on layout update"); return; };

                // Get the offset between our node's anchor and the parent node's anchor.
                let mut offset = layout.offset(*parent.size);
                let dims = layout.dims(*parent.size);

                // Convert the offset to a translation between the parent and node origins.
                // - Offset = [vector to parent upper left corner]
                //          + [anchor offset vector (convert y)]
                //          + [node corner to node origin (convert y)]
                offset.x = (-parent.size.x / 2.) + offset.x + (dims.x / 2.);
                offset.y = (parent.size.y / 2.) + -offset.y + (-dims.y / 2.);

                // Update this node's transform.
                *transform = Transform::from_translation(offset.extend(*parent.child_offset));
                transform.rotation = Quat::from_rotation_z(layout.z_rotation);

                // Update our node's size.
                //todo: how to use set_if_not_eq? NodeSize contains floats...
                *size.get_mut(&mut rc) = NodeSize(dims);
            }
        );
        cleanup_reactor_on_despawn(rc, node, token);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`CobwebStyle`] that wraps [`Layout`] with simple justification-based settings.
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum JustifiedLayout
{
    TopLeft(Size),
    TopCenter(Size),
    TopRight(Size),
    CenterLeft(Size),
    Center(Size),
    CenterRight(Size),
    BottomLeft(Size),
    BottomCenter(Size),
    BottomRight(Size),
}

impl From<JustifiedLayout> for Layout
{
    fn from(justified: JustifiedLayout) -> Self
    {
        match justified
        {
            JustifiedLayout::Center(size) => Layout::centered(size),
            _ => todo!(),
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
        app.register_type::<Layout>()
            .register_type::<JustifiedLayout>()
            .add_systems(PreStartup, setup_justified_layout);
    }
}

//-------------------------------------------------------------------------------------------------------------------
