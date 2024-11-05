use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use bevy::ui::widget::text_system;
use smallvec::SmallVec;

use crate::prelude::*;
use crate::sickle_ext::prelude::DynamicStylePostUpdate;

// TODO: consider adding IgnorePropagateOpacity so child nodes can opt-out. This would allow you to for example
// fade in ancestor nodes while keeping a segment of the node tree the same opacity.

//-------------------------------------------------------------------------------------------------------------------

const ALPHA_ROUNDING_ERROR: f32 = 0.0000001;

//-------------------------------------------------------------------------------------------------------------------

fn color_alpha(color: &Color) -> f32
{
    match color {
        Color::Srgba(Srgba { alpha, .. })
        | Color::LinearRgba(LinearRgba { alpha, .. })
        | Color::Hsla(Hsla { alpha, .. })
        | Color::Hsva(Hsva { alpha, .. })
        | Color::Hwba(Hwba { alpha, .. })
        | Color::Laba(Laba { alpha, .. })
        | Color::Lcha(Lcha { alpha, .. })
        | Color::Oklaba(Oklaba { alpha, .. })
        | Color::Oklcha(Oklcha { alpha, .. })
        | Color::Xyza(Xyza { alpha, .. }) => *alpha,
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn set_color_alpha(color: &mut Color, new_alpha: f32)
{
    match color {
        Color::Srgba(Srgba { alpha, .. })
        | Color::LinearRgba(LinearRgba { alpha, .. })
        | Color::Hsla(Hsla { alpha, .. })
        | Color::Hsva(Hsva { alpha, .. })
        | Color::Hwba(Hwba { alpha, .. })
        | Color::Laba(Laba { alpha, .. })
        | Color::Lcha(Lcha { alpha, .. })
        | Color::Oklaba(Oklaba { alpha, .. })
        | Color::Oklcha(Oklcha { alpha, .. })
        | Color::Xyza(Xyza { alpha, .. }) => {
            *alpha = new_alpha;
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores the opacity of UI components *without* propagated modifiers.
///
/// Used to restore alpha values after rendering.
// TODO: consider a better design that's more flexible for user-defined components?
#[derive(Component, Clone, Debug, Default)]
struct RestorableOpacity
{
    ui_image: f32,
    // Record for each section.
    text: SmallVec<[f32; 1]>,
    border_color: f32,
    background_color: f32,
}

//-------------------------------------------------------------------------------------------------------------------

fn recursively_propagate_opacity_value(
    mut first_traversal: bool,
    mut accumulated_opacity: f32,
    seen_propagators: &mut EntityHashSet,
    insertion_first_traversal_vals: &mut EntityHashMap<RestorableOpacity>,
    c: &mut Commands,
    children_query: &Query<&Children>,
    nodes: &mut Query<
        (
            Option<&PropagateOpacity>,
            Option<&mut RestorableOpacity>,
            Option<&mut UiImage>,
            Option<&mut Text>,
            Option<&mut BorderColor>,
            Option<&mut BackgroundColor>,
        ),
        With<Node>,
    >,
    entity: Entity,
)
{
    let Ok((maybe_propagator, maybe_restorable, maybe_img, maybe_text, maybe_br_color, maybe_bg_color)) =
        nodes.get_mut(entity)
    else {
        return;
    };

    // Handle the case that this node has `PropagateOpacity`.
    if let Some(PropagateOpacity(value)) = maybe_propagator {
        // Track seen.
        if !seen_propagators.insert(entity) {
            // If we've already seen this propagator, then this node and its children must have already
            // been updated once, so we don't want to overwrite the restoration values.
            first_traversal = false;
        }

        // Accumulate this value.
        // - Ignoring 1.0 hopefully avoids weird floating point issues that would invalidate the 1.0 check down
        //   below.
        if !value.is_nan() && *value != 1.0 {
            accumulated_opacity *= *value;
        }
    }

    // No need to continue if opacity won't be changed.
    // - Pass through if not the first traversal in case somehow we went from non-1.0 accumulated to 1.0
    //   accumulated by adding in ancestor opacities (e.g. if this node has 0.5 and an ancestor has 2.0).
    if first_traversal && (accumulated_opacity - 1.0).abs() <= ALPHA_ROUNDING_ERROR {
        return;
    }

    // Update restorable value.
    if maybe_img.is_some() || maybe_text.is_some() || maybe_br_color.is_some() || maybe_bg_color.is_some() {
        let update_restorable = |restorable: &mut RestorableOpacity| {
            if let Some(mut img) = maybe_img {
                if first_traversal {
                    restorable.ui_image = color_alpha(&img.color);
                }
                let computed = restorable.ui_image * accumulated_opacity;
                if (color_alpha(&img.color) - computed).abs() > ALPHA_ROUNDING_ERROR {
                    set_color_alpha(&mut img.color, computed);
                }
            }
            if let Some(mut text) = maybe_text {
                if first_traversal {
                    restorable.text.clear();
                }
                text.sections
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, section)| {
                        if first_traversal {
                            restorable.text.push(color_alpha(&section.style.color));
                        }
                        let original = restorable.text[idx];
                        let computed = original * accumulated_opacity;
                        if (color_alpha(&section.style.color) - computed).abs() > ALPHA_ROUNDING_ERROR {
                            set_color_alpha(&mut section.style.color, computed);
                        }
                    });
            }
            if let Some(mut br_color) = maybe_br_color {
                if first_traversal {
                    restorable.border_color = color_alpha(&br_color.0);
                }
                let computed = restorable.border_color * accumulated_opacity;
                if (color_alpha(&br_color.0) - computed).abs() > ALPHA_ROUNDING_ERROR {
                    set_color_alpha(&mut br_color.0, computed);
                }
            }
            if let Some(mut bg_color) = maybe_bg_color {
                if first_traversal {
                    restorable.background_color = color_alpha(&bg_color.0);
                }
                let computed = restorable.background_color * accumulated_opacity;
                if (color_alpha(&bg_color.0) - computed).abs() > ALPHA_ROUNDING_ERROR {
                    set_color_alpha(&mut bg_color.0, computed);
                }
            }
        };

        if let Some(restorable) = maybe_restorable {
            // Try to reuse the existing component, which is potentially allocated.
            update_restorable(restorable.into_inner());
        } else {
            let mut restorable = insertion_first_traversal_vals
                .get(&entity)
                .cloned()
                .unwrap_or_default();
            update_restorable(&mut restorable);
            if first_traversal {
                insertion_first_traversal_vals.insert(entity, restorable.clone());
                c.entity(entity).insert(restorable);
            }
        }
    }

    // Iterate into children.
    let Ok(children) = children_query.get(entity) else { return };
    for child in children.iter() {
        recursively_propagate_opacity_value(
            first_traversal,
            accumulated_opacity,
            seen_propagators,
            insertion_first_traversal_vals,
            c,
            children_query,
            nodes,
            *child,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Applies all opacity modifiers throughout the hierarchy, and caches the original opacity values for
/// restoration after rendering.
fn propagate_opacity_values(
    // Optimization to reduce reduntant traversals by 50%.
    mut seen_propagators: Local<EntityHashSet>,
    mut insertion_first_traversal_vals: Local<EntityHashMap<RestorableOpacity>>,
    mut c: Commands,
    propagators: Query<Entity, With<PropagateOpacity>>,
    children: Query<&Children>,
    mut nodes: Query<
        (
            // Include this in case we need to merge modifiers.
            Option<&PropagateOpacity>,
            Option<&mut RestorableOpacity>,
            Option<&mut UiImage>,
            Option<&mut Text>,
            Option<&mut BorderColor>,
            Option<&mut BackgroundColor>,
        ),
        With<Node>,
    >,
)
{
    seen_propagators.clear();
    insertion_first_traversal_vals.clear();

    for propagator in propagators.iter() {
        // Only do this in the base level so ancestor opacities properly reach all children.
        if seen_propagators.contains(&propagator) {
            continue;
        }

        recursively_propagate_opacity_value(
            true,
            1.0,
            &mut *seen_propagators,
            &mut *insertion_first_traversal_vals,
            &mut c,
            &children,
            &mut nodes,
            propagator,
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Returns cached opacity values to components after modified values were rendered.
///
/// Note: For simpliciy we filter for `Changed<RestorableOpacity>`, which indicates values need to be fixed
/// in this system. If `RestorableOpacity` doesn't get modified, then it will simply be an inert component.
fn restore_opacity(
    mut nodes: Query<
        (
            &RestorableOpacity,
            Option<&mut UiImage>,
            Option<&mut Text>,
            Option<&mut BorderColor>,
            Option<&mut BackgroundColor>,
        ),
        Changed<RestorableOpacity>,
    >,
)
{
    // Restore alphas while avoiding excess change detection.
    for (restorable, maybe_img, maybe_text, maybe_br_color, maybe_bg_color) in nodes.iter_mut() {
        if let Some(mut img) = maybe_img {
            if color_alpha(&img.color) != restorable.ui_image {
                set_color_alpha(&mut img.color, restorable.ui_image);
            }
        }
        if let Some(mut text) = maybe_text {
            if text.sections.iter().enumerate().any(|(idx, section)| {
                let Some(restorable) = restorable.text.get(idx) else { return false };
                color_alpha(&section.style.color) != *restorable
            }) {
                text.sections
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, section)| {
                        let Some(restorable) = restorable.text.get(idx) else { return };
                        set_color_alpha(&mut section.style.color, *restorable);
                    });
            }
        }
        if let Some(mut br_color) = maybe_br_color {
            if color_alpha(&br_color.0) != restorable.border_color {
                set_color_alpha(&mut br_color.0, restorable.border_color);
            }
        }
        if let Some(mut bg_color) = maybe_bg_color {
            if color_alpha(&bg_color.0) != restorable.background_color {
                set_color_alpha(&mut bg_color.0, restorable.background_color);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component for setting an opacity multiplier on a hierarchy of nodes.
///
/// The propagated value will stack with other opacity multipliers in the same hierarchy.
///
/// ## Limitations
///
/// The current implementation applies the opacity modifier to all child node components separately. This means
/// you won't get a *composited* fading effect. For example, if you have a window with an icon on it, and fade
/// out that window, then the window's background color will bleed through the icon when the icon's alpha is
/// reduced.
///
/// ## Performance
///
/// This is a convenient tool for fading in/fading out pop-ups like on-hover help text. However, it may not be
/// efficient to *hide* those popups using inherited opacity, because it does require hierarchy traversal.
/// If perf becomes an issue, you should use [`Visibility::Hidden`] to hide popups, and only insert
/// this component when animating a transition to full opacity.
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct PropagateOpacity(pub f32);

impl Instruction for PropagateOpacity
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut ec) = world.get_entity_mut(entity) else { return };
        ec.insert(self);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

impl ThemedAttribute for PropagateOpacity
{
    type Value = f32;
    fn construct(value: Self::Value) -> Self
    {
        Self(value)
    }
}
impl ResponsiveAttribute for PropagateOpacity {}
impl AnimatableAttribute for PropagateOpacity {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiOpacityPlugin;

impl Plugin for UiOpacityPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_animatable::<PropagateOpacity>()
            .add_systems(
                PostUpdate,
                propagate_opacity_values
                    .after(ControlSet)
                    .after(DynamicStylePostUpdate)
                    // Before text is converted to glyphs for rendering.
                    .before(text_system),
            )
            // After rendering.
            .add_systems(First, restore_opacity);
    }
}

//-------------------------------------------------------------------------------------------------------------------
