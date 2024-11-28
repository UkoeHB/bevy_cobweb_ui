//! WIP

//-------------------------------------------------------------------------------------------------------------------

/// Anchor point on a tooltip's parent where the tooltip should be positioned.
pub enum TooltipAnchor
{
    /// Top-right corner of the parent node.
    TopRight,
    /// Top-center point on the parent node.
    TopCenter,
    /// Top-left corner of the parent node.
    TopLeft,
    /// Left-center point on the parent node.
    LeftCenter,
    /// Bottom-left corner of the parent node.
    BottomLeft,
    /// Bottom-center point on the parent node.
    BottomCenter,
    /// Bottom-right corner of the parent node.
    BottomRight,
    /// Right-center point on the parent node.
    RightCenter,
    /// The center of the parent node. The tooltip is aligned on its top edge.
    CenterAlignTop,
    /// The center of the parent node. The tooltip is aligned on its left edge.
    CenterAlignLeft,
    /// The center of the parent node. The tooltip is aligned on its bottom edge.
    CenterAlignBottom,
    /// The center of the parent node. The tooltip is aligned on its right edge.
    CenterAlignRight,
}

//-------------------------------------------------------------------------------------------------------------------

/// A tooltip's alignment on its edge facing the parent node. Without any offset, the anchor point and alignment
/// point will overlap.
pub enum TooltipAlignment
{
    /// `Top*`/`Bottom*` anchor: left bottom/top corner of the tooltip.
    /// `LeftCenter`/`RightCenter` anchor: top right/left corner of the tooltip.
    Start,
    /// `Top*`/`Bottom*` anchor: center of bottom/top edge of the tooltip.
    /// `LeftCenter`/`RightCenter` anchor: center of right/left edge of the tooltip.
    Center,
    /// `Top*`/`Bottom*` anchor: right bottom/top corner of the tooltip.
    /// `LeftCenter`/`RightCenter` anchor: bottom right/left corner of the tooltip.
    End,
}

//-------------------------------------------------------------------------------------------------------------------

/// Component/instruction for setting up a tooltip associated with the current entity.
///
/// The tooltip will spawn as a fresh UI scene when hovering the entity.
pub struct WithTooltip
{
    /// A reference to the scene of the tooltip to spawn. Format: `(file path, scene name)`.
    pub tooltip: (SmolStr, SmolStr),

    /// The anchor point on the reference node for the tooltip.
    ///
    /// Defaults to [`TooltipAnchor::TopLeft`].
    #[reflect(default)]
    pub anchor: TooltipAnchor,
    /// The alignment of the tooltip relative to the anchor point.
    ///
    /// Defaults to [`TooltipAlignment::End`].
    #[reflect(default)]
    pub alignment: TooltipAlignment,
    /// Offset from the anchor point to the alignment point.
    ///
    /// Defaults to no offset.
    #[reflect(default)]
    pub offset: Vec2,

    /// Fade-in settings for when the tooltip should appear.
    ///
    /// Includes [`AnimationConfig::delay`], which lets you delay when the tooltip appears after a hover on the
    /// entity starts.
    ///
    /// Defaults to instantaneous.
    #[reflect(default)]
    pub fade_in: Option<AnimationConfig>,
    /// Fade-out settings for when the tooltip should despawn.
    ///
    /// Defaults to instantaneous.
    #[reflect(default)]
    pub fade_out: Option<AnimationConfig>,
    /// If set, then the tooltip will fade-out when the entity is pressed.
    ///
    /// Defaults to `false`.
    #[reflect(default)]
    pub remove_on_press: bool,

    /// If set, then the tooltip will adjust its position to avoid overlap with the cursor if there is no room
    /// on the reference node to move the cursor so that the tooltip will be fully visible.
    ///
    /// Cursor avoidance currently only works for custom cursors where the cursor size and hotspot are known. See
    /// [`CursorIcon`].
    ///
    /// Defaults to `true`.
    #[reflect(default = "WithTooltip::avoid_cursor_default")]
    pub avoid_cursor: bool,
    /// If set, then the tooltip will be repositioned to stay inside the primary window.
    ///
    /// Respositioning is done by first trying to 'push' the tooltip away from the window. If pushing results
    /// in the tooltip overlapping with the 'offset box' around the edge of the reference node, then
    /// the anchor point will be flipped away from the overlap.
    ///
    /// If the window is too small to fit the tooltip, then it will be adjusted so its top and left edges stay within
    /// the window.
    ///
    /// Defaults to `true`.
    #[reflect(default = "WithTooltip::stay_in_window_default")]
    pub stay_in_window: bool,
    /// Minimum distance allowed between the tooltip and the window edges. Only takes effect if `stay_in_window`
    /// is set.
    ///
    /// Will shrink to zero if the window is too small to include both the tooltip and the padding.
    ///
    /// Defaults to no padding.
    #[reflect(default)]
    pub window_padding: f32,
}

impl WithTooltip
{
    fn avoid_cursor_default() -> bool
    {
        true
    }

    fn stay_in_window_default() -> bool
    {
        true
    }
}

// Need WithTooltipReactor component to insert on reactor entities so they can be cleaned up on WithTooltip revert
// Reactor entities should be child-ed to the WithTooltip entity

// On tooltip spawn, make Animated<PropagateOpacity> to control the entry, and another tied to the Dying pseudostate
// for fade-out.

/*
- target entity: WithTooltip instruction
    - on_pointer_enter, on_pressed: spawn tooltip, add HasTooltip component to hovered entity
    - on_pointer_leave, on_released, on_press_canceled: remove HasTooltip component, add "Dying" pseudostate
    (activates Animated<PropagateOpacity>), also add DespawnOnOpacityOut component which waits for PropagateOpacity to
    reach zero then despawns
        - DespawnOnOpacityOut checker system should be ordered in Last to make sure PropagateOpacity has a chance to be
        inserted by the flux framework
- tooltip
    - has TooltipParent component
    - has WindowClamp component
    - has CenterPosition component
        - includes absolute offset
        - includes left/right and top/bottom  
    - has Animated<PropagateOpacity> with delay for on_enter, and a second entry tied to "Dying" pseudostate
    - system: update CenterPosition from TooltipParent, if parent is missing then despawn self
    - system: 
- demo
    - make mock-draggable object with tooltip
    - on trigger DragStart, insert ComputedCenteringDrag component with initial center position
    - on trigger Drag, update ComputedCenteringDrag with distance traveled
    - no need to remove ComputedCenteringDrag for the demo
    - system: convert ComputedCenteringDrag to Transform

- use TargetCamera on current node to detect which window we are in? camera -> render target -> window entity
*/

/*
// Reference solution from discord user:

use bevy::{math::Vec3A, prelude::*, ui::UiSystem, window::PrimaryWindow};

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        window_clamp.after(TransformSystem::TransformPropagate),
    );
    app.add_systems(
        PostUpdate,
        center_position
            .after(UiSystem::Layout)
            .before(TransformSystem::TransformPropagate),
    );
}

/// UI nodes with this component will position their center at the specified position.
#[derive(Component)]
pub struct CenterPosition {
    pub position: Vec2,
}

/// UI nodes with this component will be moved to fit within the window size.
#[derive(Component)]
pub struct WindowClamp;

pub fn center_position(mut nodes: Query<(&mut Transform, &CenterPosition)>) {
    for (mut transform, center) in &mut nodes {
        transform.translation.x = center.position.x;
        transform.translation.y = center.position.y;
    }
}

pub fn window_clamp(
    mut nodes: Query<(&mut GlobalTransform, &ComputedNode), With<WindowClamp>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let size = window.size();
    for (mut transform, node) in &mut nodes {
        let mut affine = transform.affine();
        let half_size = node.size() / 2.0;
        let min = (affine.translation.xy() - half_size).min(Vec2::ZERO);
        let max = size - (affine.translation.xy() + half_size).max(size);
        affine.translation += Vec3A::from((min + max).extend(0.0));
        *transform = GlobalTransform::from(affine);
    }
}
*/

//-------------------------------------------------------------------------------------------------------------------
