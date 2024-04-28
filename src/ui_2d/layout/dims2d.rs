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

// Note: this does not detect insertions because it is assumed dirtyness will be detected in other ways for that case.
fn detect_dims2d_reactor(
    mutation    : MutationEvent<Dims2d>,
    entities    : &Entities,
    mut tracker : ResMut<DirtyNodeTracker>
){
    let entity = mutation.read().unwrap();
    if entities.get(entity).is_none() { return; }
    tracker.insert(entity);
}

struct DetectDims2dReactor;
impl WorldReactor for DetectDims2dReactor
{
    type StartingTriggers = MutationTrigger::<Dims2d>;
    type Triggers = ();
    fn reactor(self) -> SystemCommandCallback { SystemCommandCallback::new(detect_dims2d_reactor) }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn update_dims2d(
    In((entity, dims)): In<(Entity, Dims2d)>,
    mut c: Commands,
    mut query: Query<Option<&mut React<Dims2d>>>,
){
    // Insert or update.
    if let Ok(mut existing) = query.get_mut(entity) {
        existing.set_if_not_eq(&mut c, dims);
    } else {
        c.react().insert(entity, dims);
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Represents a transformation between two rectangles.
///
/// `Dims2d` is used a node's [`SizeRef`] into a [`NodeSizeEstimate`].
///
/// Mutating `Dims2d` on a node will automatically mark it [dirty](DirtyNodeTracker) (but not inserting/removing it).
///
/// Defaults to [`Self::overlay`].
#[derive(ReactComponent, Reflect, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dims2d
{
    /// Indicates the desired width of the node.
    ///
    /// Does nothing if [`Self::solid_in_fill`] or [`Self::solid_out_fill`] is set.
    ///
    /// Defaults to [`Val2d::Percent(100.)`].
    #[reflect(default = "Dims2d::width_height_default")]
    pub width: Val2d,
    /// Indicates the desired height of the node.
    ///
    /// Does nothing if [`Self::solid_in_fill`] or [`Self::solid_out_fill`] is set.
    ///
    /// Defaults to [`Val2d::Percent(100.)`].
    #[reflect(default = "Dims2d::width_height_default")]
    pub height: Val2d,
    /// Controls the absolute maximum width of the node.
    ///
    /// Defaults to [`Val2d::Inf`].
    #[reflect(default = "Dims2d::max_width_height_default")]
    pub max_width: Val2d,
    /// Controls the absolute maximum height of the node.
    ///
    /// Defaults to [`Val2d::Inf`].
    #[reflect(default = "Dims2d::max_width_height_default")]
    pub max_height: Val2d,
    /// Controls the absolute minimum width of the node.
    ///
    /// Defaults to [`Val2d::Px(0.)`].
    #[reflect(default)]
    pub min_width: Val2d,
    /// Controls the absolute minimum height of the node.
    ///
    /// Defaults to [`Val2d::Px(0.)`].
    #[reflect(default)]
    pub min_height: Val2d,
    /// The node's dimensions are fixed to a certain (x/y) ratio, and both dimensions are inside the parent's dimensions
    /// (with at least one dimension equal to the parent's corresponding dimension).
    ///
    /// Ratio parameters are clamped to `>= 1`.
    ///
    /// If set, then [`Self::width`] and [`Self::height`] will be ignored. Takes precedence over [`Self::sold_out_fill`].
    ///
    /// Defaults to `None`.
    #[reflect(default)]
    pub solid_in_fill: Option<(u32, u32)>,
    /// The node's dimensions are fixed to a certain (x/y) ratio, and both dimensions are outside the parent's dimensions
    /// (with at least one dimension equal to the parent's corresponding dimension).
    ///
    /// Ratio parameters are clamped to `>= 1`.
    ///
    /// If set, then [`Self::width`] and [`Self::height`] will be ignored. Does nothing if [`Self::solid_in_fill`] is set.
    ///
    /// Defaults to `None`.
    #[reflect(default)]
    pub solid_out_fill: Option<(u32, u32)>,
/*
    /// Region between a node's boundary and its padding.
    ///
    /// Defaults to zero border.
    #[reflect(default)]
    pub border: StyleRect,
*/
}

impl Dims2d
{
    fn width_height_default() -> Val2d
    {
        Val2d::Percent(100.)
    }

    fn max_width_height_default() -> Val2d
    {
        Val2d::Inf
    }

    /// Creates dimensions that overlay the parent.
    pub fn overlay() -> Self
    {
        Self::default()
    }

    /// Transforms `parent_size` into a child size.
    ///
    /// Will not return values less than zero.
    pub fn compute(&self, mut parent_size: Vec2) -> Vec2
    {
        parent_size.x = fix_nan(parent_size.x);
        parent_size.y = fix_nan(parent_size.y);

        // Get desired size.
        let desired_size = match (self.solid_in_fill, self.solid_out_fill) {
            // Solid in fill
            (Some((ratio_x, ratio_y)), _) => {
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
            // Solid out fill
            (None, Some((ratio_x, ratio_y))) => {
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
            // Normal width/height
            _ => {
                Vec2{
                    x: self.width.compute(parent_size.x).max(0.),
                    y: self.height.compute(parent_size.y).max(0.),
                }
            }
        }

        // Apply min/max constraints.
        let min_width = self.min_width.compute(parent_size.x).max(0.);
        let max_width = self.max_width.compute(parent_size.x).max(min_width);
        let min_height = self.min_height.compute(parent_size.y).max(0.);
        let max_height = self.max_height.compute(parent_size.y).max(min_height);

        let clamped_size = Vec2{
            x: desired_size.x.clamp(min_width, max_width),
            y: desired_size.y.clamp(min_height, max_height)
        };

        clamped_size
    }
}

impl Default for Dims2d
{
    fn default() -> Self
    {
        Self{
            width: Dims2d::width_height_default(),
            height: Dims2d::width_height_default(),
            max_width: Dims2d::max_width_height_default(),
            max_height: Dims2d::max_width_height_default(),
            min_width: Val2d::Px(0.),
            min_height: Val2d::Px(0.),
            solid_in_fill: None,
            solid_out_fill: None,
        }
    }
}

impl ApplyLoadable for Dims2d
{
    fn apply(self, ec: &mut EntityCommands)
    {
        let id = ec.id();
        ec.commands().syscall((id, self), update_dims2d);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct Dims2dPlugin;

impl Plugin for Dims2dPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_type::<Dims2d>()
            .register_type::<(u32, u32)>()
            .add_reactor_with(DetectDims2dReactor, mutation::<Dims2d>());
    }
}

//-------------------------------------------------------------------------------------------------------------------
