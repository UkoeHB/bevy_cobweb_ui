//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates a node's size.
fn dims_reactor(
    ref_event : MutationEvent<SizeRef>,
    lay_event : MutationEvent<Dims>,
    mut rc    : ReactCommands,
    mut nodes : Query<(&mut React<NodeSize>, &React<Dims>, &React<SizeRef>)>
){
    let Some(node) = ref_event.read().or_else(|| lay_event.read())
    else { tracing::error!("failed running dims reactor, event is missing"); return; };
    let Ok((mut size, dims, size_ref)) = nodes.get_mut(node)
    else { tracing::debug!(?node, "node missing on dims update"); return; };

    // Update our node's size if it changed.
    let parent_size = *size_ref.parent_size;
    let dims = dims.compute(parent_size);
    size.set_if_not_eq(&mut rc, NodeSize(dims));
}

struct DimsReactor;
impl WorldReactor for DimsReactor
{
    type StartingTriggers = ();
    type Triggers = (EntityMutationTrigger<SizeRef>, EntityMutationTrigger<Dims>);
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(dims_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Represents a transformation between two rectangles.
///
/// When `Dims` is used as a [`UiInstruction`], this is used to transform the node's [`DimsRef`] into a
/// [`NodeSizeEstimate`].
///
/// `Dims` can also be wrapped in [`MinDims`] and [`MaxDims`] instructions, which will constrain the node's
/// [`NodeSizeEstimate`] and also its final [`NodeSize`] if it has a [`NodeSizeAdjuster`].
#[derive(ReactComponent, Reflect, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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
    /// The node's width and height equal `(parent_dims - pad) * rel + abs`.
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
    /// Creates a node that perfectly overlaps its parent.
    pub fn overlay() -> Self
    {
        Self::Padded(Vec2::default())
    }

    /// Transforms `parent_size` into a child size.
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

impl CobwebStyle for Dims
{
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity)
    {
        rc.commands().syscall(node,
            |
                In(node)    : In<Entity>,
                mut rc      : ReactCommands,
                mut reactor : Reactor<DimsReactor>,
            |
            {
                reactor.add_triggers(&mut rc, (entity_mutation::<SizeRef>(node), entity_mutation::<Dims>(node)));
            }
        );
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

pub(crate) struct DimsPlugin;

impl Plugin for DimsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Dims>()
            .register_type::<(u32, u32)>()
            .add_reactor(DimsReactor);
    }
}

//-------------------------------------------------------------------------------------------------------------------
