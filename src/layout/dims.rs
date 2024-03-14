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

fn detect_dims_reactor(
    mutation    : MutationEvent<Dims>,
    entities    : &Entities,
    mut tracker : ResMut<DirtyNodeTracker>
){
    let entity = mutation.read().unwrap();
    if entities.get(entity).is_none() { return; }
    tracker.insert(entity);
}

struct DetectDimsReactor;
impl WorldReactor for DetectDimsReactor
{
    type StartingTriggers = MutationTrigger::<Dims>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_dims_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Represents a transformation between two rectangles.
///
/// When `Dims` is used as a [`UiInstruction`], this is used to transform the node's [`SizeRef`] into a
/// [`NodeSizeEstimate`].
///
/// Mutating `Dims` on a node will automatically mark it [dirty](DirtyNodeTracker) (but not inserting/removing it).
///
/// `Dims` can also be wrapped in [`MinDims`] and [`MaxDims`] instructions, which will constrain the node's
/// [`NodeSizeEstimate`] and also its final [`NodeSize`] if it has a [`NodeSizeAdjuster`].
///
/// Defaults to [`Self::Overlay`].
#[derive(ReactComponent, Reflect, Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Dims
{
    /// The node has zero size.
    None,
    /// The node's width and height equal the parent's width and height.
    #[default]
    Overlay,
    /// The node's width and height are absolute values in UI coordinates.
    Pixels(Vec2),
    /// The node's width and height are percentages of the parents' width and height.
    Percent(Vec2),
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
    /// The node's width and height equal `(parent_dims - pad) * percent + pixels`.
    ///
    /// Relative values are recorded in percentages.
    Combined{
        #[reflect(default)]
        pad: Vec2,
        #[reflect(default)]
        pixels: Vec2,
        #[reflect(default)]
        percent: Vec2
    },
    /// Equivalent to [`Self::SolidInFill`] applied to parent dimensions adjusted by [`Self::Combined`].
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidIn{
        ratio: (u32, u32),
        #[reflect(default)]
        pad: Vec2,
        #[reflect(default)]
        pixels: Vec2,
        #[reflect(default)]
        percent: Vec2
    },
    /// Equivalent to [`Self::SolidOutFill`] applied to parent dimensions adjusted by [`Self::Combined`].
    ///
    /// Ratio parameters are clamped to >= 1.
    SolidOut{
        ratio: (u32, u32),
        #[reflect(default)]
        pad: Vec2,
        #[reflect(default)]
        pixels: Vec2,
        #[reflect(default)]
        percent: Vec2
    },
}

impl Dims
{
    /// Transforms `parent_size` into a child size.
    pub fn compute(&self, parent_size: Vec2) -> Vec2
    {
        match *self
        {
            Self::None => Vec2::default(),
            Self::Overlay => parent_size,
            Self::Pixels(pixels) =>
            {
                Vec2{
                    x: pixels.x.max(0.),
                    y: pixels.y.max(0.),
                }
            }
            Self::Percent(rel) =>
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
            Self::Combined{ pad, pixels, percent } =>
            {
                let parent_size = Self::Padded(pad).compute(parent_size);
                Self::Pixels(pixels).compute(parent_size) + Self::Percent(percent).compute(parent_size)
            }
            Self::SolidIn{ ratio, pad, pixels, percent } =>
            {
                let parent_size = Self::Combined{ pad, pixels, percent }.compute(parent_size);
                Self::SolidInFill(ratio).compute(parent_size)
            }
            Self::SolidOut{ ratio, pad, pixels, percent } =>
            {
                let parent_size = Self::Combined{ pad, pixels, percent }.compute(parent_size);
                Self::SolidOutFill(ratio).compute(parent_size)
            }
        }
    }
}

impl CobwebStyle for Dims
{
    fn apply_style(&self, _rc: &mut ReactCommands, _node: Entity) { }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct DimsPlugin;

impl Plugin for DimsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Dims>()
            .register_type::<(u32, u32)>()
            .add_reactor_with(DetectDimsReactor, mutation::<Dims>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
