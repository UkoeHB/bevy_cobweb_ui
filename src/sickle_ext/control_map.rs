use attributes::prelude::DynamicStylePostUpdate;
use attributes::pseudo_state::RefreshPseudoStates;
use attributes::ui_context::UiContext;
use bevy::ecs::entity::Entities;
use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_cobweb::prelude::*;
use smallvec::SmallVec;
use smol_str::SmolStr;

use super::*;
use crate::prelude::*;
use crate::sickle_ext::attributes::dynamic_style::DynamicStyleStopwatch;
use crate::sickle_ext::attributes::dynamic_style_attribute::DynamicStyleAttribute;
use crate::sickle_ext::attributes::pseudo_state::PseudoState;
use crate::sickle_ext::prelude::{
    ContextStyleAttribute, DynamicStyle, FluxInteraction, PseudoStates, TrackedInteraction,
};
use crate::sickle_ext::ui_style::builder::StyleBuilder;
use crate::sickle_ext::ui_style::LogicalEq;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ControlRefreshCache
{
    collected_styles: Vec<(Option<Entity>, DynamicStyle)>,
    unstyled_entities: Vec<Entity>,
    style_builder: StyleBuilder,
    dynamic_style_buffers: Vec<Vec<ContextStyleAttribute>>,
    recovered_dynamic_styles: Vec<DynamicStyle>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct CachedContextualAttribute
{
    source: Option<SmolStr>,
    target: Option<SmolStr>,
    attribute: DynamicStyleAttribute,
}

impl LogicalEq for CachedContextualAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.source == other.source && self.target == other.target && self.attribute.logical_eq(&other.attribute)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct EditablePseudoTheme
{
    state: Option<SmallVec<[PseudoState; 3]>>,
    /// [ (origin entity, attribute )]
    /// - Origin entity is used for cleanup when an attribute is reverted.
    style: SmallVec<[(Entity, CachedContextualAttribute); 3]>,
}

impl EditablePseudoTheme
{
    fn new(origin: Entity, state: Option<SmallVec<[PseudoState; 3]>>, attribute: CachedContextualAttribute)
        -> Self
    {
        let mut style = SmallVec::new();
        style.push((origin, attribute));
        Self { state, style }
    }

    fn remove_origin(&mut self, origin: Entity)
    {
        self.style.retain(|(e, _)| *e != origin);
    }

    fn matches(&self, state: &Option<SmallVec<[PseudoState; 3]>>) -> bool
    {
        self.state == *state
    }

    fn set_attribute(&mut self, origin: Entity, attribute: CachedContextualAttribute)
    {
        // Merge attribute with existing list.
        if let Some(index) = self
            .style
            .iter()
            .position(|(_, attr)| attr.logical_eq(&attribute))
        {
            self.style[index] = (origin, attribute);
        } else {
            self.style.push((origin, attribute));
        }
    }

    fn is_subset(&self, node_states: &[PseudoState]) -> Option<usize>
    {
        match &self.state {
            // Only consider pseudo themes that are specific to an inclusive substet of the themed element's pseudo
            // states. A theme for [Checked, Disabled] will apply to elements with [Checked, Disabled,
            // FirstChild], but will not apply to elements with [Checked] (because the theme targets
            // more specific elements) or [Checked, FirstChild] (because they are disjoint)
            Some(theme_states) => match theme_states.iter().all(|state| node_states.contains(state)) {
                true => Some(theme_states.len()),
                false => None,
            },
            None => Some(0),
        }
    }

    /// Adds all attributes to the style builder.
    fn build(&self, style_builder: &mut StyleBuilder)
    {
        for (_, CachedContextualAttribute { source, target, attribute }) in self.style.iter() {
            // Set the placement.
            if let Some(source) = source {
                style_builder.switch_placement_with(source.clone());
            } else {
                style_builder.reset_placement();
            }

            // Set the target.
            if let Some(target) = target {
                style_builder.switch_target_with(target.clone());
            } else {
                style_builder.reset_target();
            }

            // Insert attribute.
            style_builder.add(attribute.clone());
        }
    }

    fn cleanup_references(&mut self, entities: &Entities)
    {
        self.style.retain(|(e, _)| entities.contains(*e));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that coordinates dynamic attributes for multi-entity widgets.
///
/// Control maps should be placed on the root entity of a widget. See [`ControlRoot`] and [`ControlLabel`].
#[derive(Component, Debug, Default)]
pub(crate) struct ControlMap
{
    entities: SmallVec<[(SmolStr, Entity); 5]>,
    pseudo_themes: SmallVec<[EditablePseudoTheme; 1]>,
}

impl ControlMap
{
    pub(crate) fn insert(&mut self, label: impl AsRef<str>, entity: Entity)
    {
        let label = SmolStr::new(label.as_ref());
        let Some(pos) = self.entities.iter().position(|(name, _)| *name == label) else {
            self.entities.push((label, entity));
            return;
        };
        self.entities[pos] = (label, entity);
    }

    pub(crate) fn remove(&mut self, entity: Entity)
    {
        if let Some(pos) = self.entities.iter().position(|(_, e)| *e == entity) {
            self.entities.remove(pos);
        }

        for pt in self.pseudo_themes.iter_mut() {
            pt.remove_origin(entity);
        }
    }

    pub(crate) fn set_attribute(
        &mut self,
        origin: Entity,
        mut state: Option<SmallVec<[PseudoState; 3]>>,
        source: Option<SmolStr>,
        target: Option<SmolStr>,
        attribute: DynamicStyleAttribute,
    )
    {
        if let Some(states) = state.as_deref_mut() {
            states.sort_unstable();
        }

        let attribute = CachedContextualAttribute { source, target, attribute };
        match self.pseudo_themes.iter_mut().find(|t| t.matches(&state)) {
            Some(pseudo_theme) => pseudo_theme.set_attribute(origin, attribute),
            None => self
                .pseudo_themes
                .push(EditablePseudoTheme::new(origin, state, attribute)),
        }
    }

    pub(crate) fn get_entity(&self, target: impl AsRef<str>) -> Option<Entity>
    {
        let target = target.as_ref();
        self.entities
            .iter()
            .find(|(name, _)| *name == target)
            .map(|(_, entity)| *entity)
    }

    pub(crate) fn remove_all_labels(&mut self) -> impl Iterator<Item = (SmolStr, Entity)> + '_
    {
        self.entities.drain(..)
    }

    pub(crate) fn remove_all_attrs(
        &mut self,
    ) -> Vec<(
        Entity,
        Option<SmolStr>,
        Option<SmolStr>,
        Option<SmallVec<[PseudoState; 3]>>,
        DynamicStyleAttribute,
    )>
    {
        let res = self
            .pseudo_themes
            .iter_mut()
            .flat_map(|pt| {
                let pstates = pt.state.take();
                pt.style.drain(..).map(move |(origin, ctx_attr)| {
                    (
                        origin,
                        ctx_attr.source,
                        ctx_attr.target,
                        pstates.clone(),
                        ctx_attr.attribute,
                    )
                })
            })
            .collect();
        self.pseudo_themes.clear();
        res
    }

    fn cleanup_references(&mut self, entities: &Entities)
    {
        self.entities.retain(|(_, e)| entities.contains(*e));
        for pt in self.pseudo_themes.iter_mut() {
            pt.cleanup_references(entities);
        }
    }

    fn iter_entities(&self) -> impl Iterator<Item = &(SmolStr, Entity)> + '_
    {
        self.entities.iter()
    }
}

impl UiContext for ControlMap
{
    fn get(&self, target: &str) -> Result<Entity, String>
    {
        let Some(entity) = self.get_entity(target) else {
            return Err(format!(
                "unknown UI context {target} requested for ControlMap, available are {:?}",
                Vec::from_iter(self.contexts())
            ));
        };
        Ok(entity)
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_
    {
        self.iter_entities().map(|(name, _)| name.as_str())
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for cleaning up control maps after the `ControlRoot` instruction is reverted.
#[derive(Component)]
pub(crate) struct ControlMapDying;

//-------------------------------------------------------------------------------------------------------------------

fn cleanup_control_maps(mut c: Commands, dying: Query<Entity, With<ControlMapDying>>)
{
    // If any control map is dead, then remove it and reapply its contents.
    // - Reapplied content should only 'move' to a lower control map in the hierarchy. If a higher control
    // map was added that would steal attributes from the dead control map, then that map will auto-steal
    // attributes on insert (which synchronizes with attribute loads from children). There should be no cases
    // were stale attributes from this map overwrite correct attributes on other control maps.
    // - We know if a map is 'dying' here then it's actually dead, because the only time a 'dying' flag can
    // be unset is immediately after it is set (i.e. ControlRoot instruction reverted -> ControlRoot instruction
    // re-applied).
    for entity in dying.iter() {
        c.entity(entity)
            .queue(RemoveDeadControlMap)
            .remove::<ControlMapDying>();
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn refresh_controlled_styles(
    entities: &Entities,
    mut c: Commands,
    mut changed_with_states: Query<
        (Entity, &mut ControlMap),
        (With<PseudoStates>, Or<(Changed<ControlMap>, Changed<PseudoStates>)>),
    >,
    mut changed_without_states: Query<(Entity, &mut ControlMap), (Changed<ControlMap>, Without<PseudoStates>)>,
)
{
    for (entity, mut map) in changed_with_states
        .iter_mut()
        .chain(changed_without_states.iter_mut())
    {
        // Cleanup dead references.
        // - We do this here instead of as an OnRemove hook for ControlLabel to avoid excess hierarchy traversals
        // when despawning large scenes.
        map.cleanup_references(&entities);

        // Queue styles update.
        c.entity(entity).queue(RefreshControlledStyles);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Removes a dead [`ControlMap`] and reapplies all its labels and attributes so they can be relocated to another
/// control map if possible.
struct RemoveDeadControlMap;

impl EntityCommand for RemoveDeadControlMap
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut old_control_map) = world
            .get_entity_mut(entity)
            .ok()
            .and_then(|mut emut| emut.take::<ControlMap>())
        else {
            tracing::error!("failed removing ControlMap");
            return;
        };
        if world.entity(entity).contains::<ControlMap>() {
            tracing::error!("still has ControlMap");
        }

        for (label, label_entity) in old_control_map.remove_all_labels() {
            // Clean up dynamic style on all label entities in case they were targets for placement.
            // - We expect there aren't any stale labels in this map, so removing DynamicStyle is safe here.
            let _ = world.get_entity_mut(label_entity).map(|mut e| {
                e.remove::<(DynamicStyle, DynamicStyleStopwatch)>();
            });

            ControlLabel(label).apply(label_entity, world);
        }

        for (origin, source, target, state, attribute) in old_control_map.remove_all_attrs() {
            world.syscall((origin, source, target, state, attribute), add_attribute);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct RefreshControlledStyles;

impl EntityCommand for RefreshControlledStyles
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let mut collected_styles =
            std::mem::take(&mut world.resource_mut::<ControlRefreshCache>().collected_styles);
        let mut unstyled_entities = std::mem::take(
            &mut world
                .resource_mut::<ControlRefreshCache>()
                .unstyled_entities,
        );
        let mut style_builder = std::mem::take(&mut world.resource_mut::<ControlRefreshCache>().style_builder);
        let mut dynamic_style_buffers = std::mem::take(
            &mut world
                .resource_mut::<ControlRefreshCache>()
                .dynamic_style_buffers,
        );
        let mut recovered_dynamic_styles = std::mem::take(
            &mut world
                .resource_mut::<ControlRefreshCache>()
                .recovered_dynamic_styles,
        );
        let control_map = world.get::<ControlMap>(entity).unwrap();

        // Get the entity's PseudoStates.
        let empty_pseudo_state = Vec::default();
        let pseudo_states = world
            .get::<PseudoStates>(entity)
            .map(|p| p.get())
            .unwrap_or(&empty_pseudo_state);

        // Collect eligible pseudo themes.
        //todo: this has excessive complexity ~ O(S^2*P*T); It would be easier to collect, sort, and dedup
        // pseudo_themes but unstable sorting may cause hard-to-debug order inconsistencies between pseudo themes
        // with the same number of pseudo states.
        let mut pseudo_themes: SmallVec<[&EditablePseudoTheme; 10]> = SmallVec::default();

        for i in 0..=pseudo_states.len() {
            control_map
                .pseudo_themes
                .iter()
                .filter(|pt| match pt.is_subset(pseudo_states) {
                    Some(count) => count == i,
                    None => false,
                })
                .for_each(|pt| pseudo_themes.push(pt));
        }

        // Merge attributes, overwriting per-attribute as more specific pseudo themes are encountered.
        collected_styles.clear();

        for pseudo_theme in pseudo_themes.iter() {
            style_builder.clear();
            pseudo_theme.build(&mut style_builder);
            let styles_iter = style_builder
                .convert_to_iter_with_buffers(control_map, || dynamic_style_buffers.pop().unwrap_or_default());
            collected_styles = styles_iter.fold(collected_styles, |mut collected, (placement, mut style)| {
                if let Some(index) = collected
                    .iter()
                    .position(|(c_placement, _)| *c_placement == placement)
                {
                    collected[index].1.merge_in_place(&mut style);
                    recovered_dynamic_styles.push(style);
                } else {
                    collected.push((placement, style));
                }

                collected
            });
            // Need this in a separate step due to mutable reference constraints.
            for recovered in recovered_dynamic_styles.drain(..) {
                dynamic_style_buffers.push(recovered.take_inner());
            }
        }

        // Save the dynamic styles to entities.
        let mut cleanup_main_style = true;
        unstyled_entities.clear();
        unstyled_entities.extend(control_map.iter_entities().map(|(_, entity)| *entity));
        std::mem::drop(pseudo_themes); // borrow checker needs some help...

        for (placement, mut style) in collected_styles.drain(..) {
            let placement_entity = match placement {
                Some(placement_entity) => placement_entity,
                None => {
                    cleanup_main_style = false;
                    entity
                }
            };

            unstyled_entities.retain(|e| *e != placement_entity);

            if let Some(current_style) = world.get_mut::<DynamicStyle>(placement_entity) {
                style.copy_controllers(&current_style);

                // The current style will be overwritten, so we can take its inner buffer.
                let current_style = std::mem::take(current_style.into_inner());
                dynamic_style_buffers.push(current_style.take_inner());
            }

            if style.is_interactive() || style.is_animated() {
                if world.get::<Interaction>(placement_entity).is_none() {
                    world
                        .entity_mut(placement_entity)
                        .insert(Interaction::default());
                }

                if world.get_mut::<FluxInteraction>(placement_entity).is_none() {
                    world
                        .entity_mut(placement_entity)
                        .insert(TrackedInteraction::default());
                }
            }

            world.entity_mut(placement_entity).insert(style);
        }

        // Cleanup unused styles.
        for unstyled_context in unstyled_entities.iter() {
            if let Some(old) = world.entity_mut(*unstyled_context).take::<DynamicStyle>() {
                dynamic_style_buffers.push(old.take_inner());
            }
        }

        if cleanup_main_style {
            if let Some(old) = world.entity_mut(entity).take::<DynamicStyle>() {
                dynamic_style_buffers.push(old.take_inner());
            }
        }

        // Cache buffers.
        world.resource_mut::<ControlRefreshCache>().collected_styles = collected_styles;
        world
            .resource_mut::<ControlRefreshCache>()
            .unstyled_entities = unstyled_entities;
        world.resource_mut::<ControlRefreshCache>().style_builder = style_builder;
        world
            .resource_mut::<ControlRefreshCache>()
            .dynamic_style_buffers = dynamic_style_buffers;
        world
            .resource_mut::<ControlRefreshCache>()
            .recovered_dynamic_styles = recovered_dynamic_styles;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// System set where responsive and animatable attributes are refreshed on entities after pseudo state changes.
#[derive(SystemSet, Hash, Debug, Eq, PartialEq, Copy, Clone)]
pub struct ControlSet;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ControlMapPlugin;

impl Plugin for ControlMapPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<ControlRefreshCache>()
            .configure_sets(
                PostUpdate,
                ControlSet
                    .after(RefreshPseudoStates)
                    .before(DynamicStylePostUpdate)
                    .before(UiSystem::Prepare),
            )
            .add_systems(
                PostUpdate,
                (cleanup_control_maps, refresh_controlled_styles)
                    .chain()
                    .in_set(ControlSet),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
