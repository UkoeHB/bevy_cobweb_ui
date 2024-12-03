use bevy::ecs::entity::Entities;
use bevy::ecs::system::EntityCommand;
use bevy::prelude::*;
use bevy::ui::UiSystem;
use smallvec::SmallVec;
use smol_str::SmolStr;

use super::*;
use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct PseudoThemeBuffer
{
    stack: Vec<(usize, usize, &'static SmolStr, &'static PseudoTheme)>,
}

impl PseudoThemeBuffer
{
    fn take<'a>(&mut self) -> Vec<(usize, usize, &'a SmolStr, &'a PseudoTheme)>
    {
        core::mem::take(&mut self.stack)
            .into_iter()
            .map(|_| -> (usize, usize, &SmolStr, &PseudoTheme) { unreachable!() })
            .collect()
    }

    fn recover(&mut self, mut stack: Vec<(usize, usize, &SmolStr, &PseudoTheme)>)
    {
        stack.clear();
        self.stack = stack
            .into_iter()
            .map(|_| -> (usize, usize, &'static SmolStr, &'static PseudoTheme) { unreachable!() })
            .collect();
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ControlRefreshCache
{
    collected_styles: Vec<(Option<Entity>, DynamicStyle)>,
    unstyled_entities: Vec<Entity>,
    style_builder: StyleBuilder,
    dynamic_style_buffers: Vec<Vec<ContextStyleAttribute>>,
    recovered_dynamic_styles: Vec<DynamicStyle>,
    ps_buffer: PseudoThemeBuffer,
}

impl ControlRefreshCache
{
    fn recover(
        &mut self,
        collected_styles: Vec<(Option<Entity>, DynamicStyle)>,
        unstyled_entities: Vec<Entity>,
        style_builder: StyleBuilder,
        dynamic_style_buffers: Vec<Vec<ContextStyleAttribute>>,
        recovered_dynamic_styles: Vec<DynamicStyle>,
        ps_buffer: PseudoThemeBuffer,
    )
    {
        self.collected_styles = collected_styles;
        self.unstyled_entities = unstyled_entities;
        self.style_builder = style_builder;
        self.dynamic_style_buffers = dynamic_style_buffers;
        self.recovered_dynamic_styles = recovered_dynamic_styles;
        self.ps_buffer = ps_buffer;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that coordinates dynamic attributes for multi-entity widgets (or single entities).
///
/// Control maps should be placed on the root entity of a widget. Use [`ControlRoot`] and [`ControlMember`] if you
/// want a multi-entity control group. Otherwise the control group will be 'anonymous' and only work on the current
/// entity.
#[derive(Component, Debug, Default)]
pub(crate) struct ControlMap
{
    /// Anonymous maps only manage attributes for the entity with the map component.
    ///
    /// We allow this so entities can use pseudo states without needing to set up a control group.
    anonymous: bool,
    entities: SmallVec<[(SmolStr, Entity); 5]>,
}

impl ControlMap
{
    pub(crate) fn new_anonymous(entity: Entity) -> Self
    {
        let mut map = Self { anonymous: true, ..default() };
        map.insert(SmolStr::new_static("__anon"), entity);
        map
    }

    pub(crate) fn set_anonymous(&mut self, anonymous: bool)
    {
        self.anonymous = anonymous;
    }

    pub(crate) fn make_anonymous(&mut self, c: &mut Commands, entity: Entity)
    {
        if self.is_anonymous() {
            return;
        }
        self.set_anonymous(true);
        self.remove(entity);
        self.reapply_labels(c);
        self.insert(SmolStr::new_static("__anon"), entity);
    }

    pub(crate) fn make_not_anonymous(&mut self, entity: Entity, label: SmolStr)
    {
        if !self.is_anonymous() {
            return;
        }
        self.set_anonymous(false);
        self.entities.clear();
        self.insert(label, entity);
    }

    pub(crate) fn is_anonymous(&self) -> bool
    {
        self.anonymous
    }

    pub(crate) fn reapply_labels(&mut self, c: &mut Commands)
    {
        for (label, entity) in self.entities.drain(..) {
            let Some(mut ec) = c.get_entity(entity) else { continue };
            ec.apply(ControlMember::from(label));
        }
    }

    pub(crate) fn insert(&mut self, label: SmolStr, entity: Entity)
    {
        let Some(pos) = self.entities.iter().position(|(name, _)| *name == label) else {
            self.entities.push((label, entity));
            return;
        };

        let (prev_label, prev_entity) = &self.entities[pos];
        if *prev_entity == entity {
            return;
        }

        tracing::warn!("overwriting entity for control group label \"{}\"; old={prev_entity:?}, new={entity:?}",
            prev_label.as_str());
        self.entities[pos].1 = entity;
    }

    pub(crate) fn remove(&mut self, entity: Entity)
    {
        if let Some(pos) = self.entities.iter().position(|(_, e)| *e == entity) {
            self.entities.remove(pos);
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

    fn cleanup_references(&mut self, entities: &Entities)
    {
        self.entities.retain(|(_, e)| entities.contains(*e));
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

fn handle_node_attr_changes(
    mut c: Commands,
    mut maps: Query<(&mut ControlMap, Has<NodeAttributes>)>,
    parents: Query<&Parent>,
    mut removed_attrs: RemovedComponents<NodeAttributes>,
    node_attributes: Query<(Entity, Option<&ControlMember>), Changed<NodeAttributes>>,
)
{
    // Cleanup anonymous control maps on attrs removal.
    for removed in removed_attrs.read() {
        let Ok((map, has_attrs)) = maps.get(removed) else { continue };
        if has_attrs {
            continue;
        }
        if !map.is_anonymous() {
            continue;
        }
        c.entity(removed).remove::<ControlMap>();
    }

    // Mark maps as changed when node attrs changed on any member of the control group.
    for (entity, maybe_label) in node_attributes.iter() {
        if let Some(label) = maybe_label {
            // Case: control group
            let mut current_entity = entity;

            loop {
                if let Ok((mut map, _)) = maps.get_mut(current_entity) {
                    if !map.is_anonymous() {
                        map.set_changed();
                        break;
                    } else if current_entity == entity {
                        map.make_not_anonymous(entity, label.id.clone());
                        break;
                    }
                }

                let Ok(parent) = parents.get(current_entity) else {
                    tracing::warn!("failed finding control root for entity {:?} with {:?}; use ControlRoot",
                        entity, label);
                    break;
                };
                current_entity = **parent;
            }
        } else {
            // Case: no control group
            if let Ok((mut map, _)) = maps.get_mut(entity) {
                map.set_changed();

                if !map.is_anonymous() {
                    map.make_anonymous(&mut c, entity);
                }
            } else {
                let map = ControlMap::new_anonymous(entity);
                c.entity(entity).insert(map);
            }
        }
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
        // - We do this here instead of as an OnRemove hook for ControlMember to avoid excess hierarchy traversals
        // when despawning large scenes.
        map.cleanup_references(&entities);

        // Queue styles update.
        c.entity(entity).queue(RefreshControlledStyles);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Removes a dead [`ControlMap`] and reapplies all its labels so they can be relocated to another
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
            return;
        };

        for (label, label_entity) in old_control_map.remove_all_labels() {
            // Clean up dynamic style on all label entities in case they were targets for placement.
            // - We expect there aren't any stale labels in this map, so removing DynamicStyle is safe here.
            let _ = world.get_entity_mut(label_entity).map(|mut e| {
                e.remove::<(DynamicStyle, DynamicStyleStopwatch)>();
            });

            ControlMember::from(label).apply(label_entity, world);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct RefreshControlledStyles;

impl EntityCommand for RefreshControlledStyles
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Need to check this first due to borrow checker limitations (>.<).
        if world.get::<ControlMap>(entity).is_none() {
            return;
        }

        let mut cache = world.resource_mut::<ControlRefreshCache>();
        let mut collected_styles = std::mem::take(&mut cache.collected_styles);
        let mut unstyled_entities = std::mem::take(&mut cache.unstyled_entities);
        let mut style_builder = std::mem::take(&mut cache.style_builder);
        let mut dynamic_style_buffers = std::mem::take(&mut cache.dynamic_style_buffers);
        let mut recovered_dynamic_styles = std::mem::take(&mut cache.recovered_dynamic_styles);
        let mut ps_buffer = std::mem::take(&mut cache.ps_buffer);
        let mut pseudo_themes = ps_buffer.take();

        let control_map = world.get::<ControlMap>(entity).unwrap();

        // Get the entity's PseudoStates.
        let empty_pseudo_state = Vec::default();
        let pseudo_states = world
            .get::<PseudoStates>(entity)
            .map(|p| p.get())
            .unwrap_or(&empty_pseudo_state);

        // Collect eligible pseudo themes.
        // - We store an index for use during sorting to avoid hard-to-debug order inconsistencies.
        let mut idx = 0;
        for (label, entity) in control_map.iter_entities() {
            // Failure is not an error if the entity doesn't have any attributes.
            let Some(attrs) = world.get::<NodeAttributes>(*entity) else { continue };

            for pt in attrs.iter_themes() {
                let Some(count) = pt.is_subset(pseudo_states) else { continue };
                pseudo_themes.push((count, idx, label, pt));
                idx += 1;
            }
        }

        // Sort themes by how well they match the pseudo states.
        pseudo_themes.sort_unstable_by(|(count, idx, _, _), (count_b, idx_b, _, _)| {
            count.cmp(count_b).then(idx.cmp(idx_b))
        });

        // Merge attributes, overwriting per-attribute as more specific pseudo themes are encountered.
        collected_styles.clear();

        for (_, _, label, pseudo_theme) in pseudo_themes.iter() {
            style_builder.clear();
            pseudo_theme.build(label, &mut style_builder);
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

        ps_buffer.recover(pseudo_themes); // borrow checker needs some help...

        // Save the dynamic styles to entities.
        let mut cleanup_main_style = true;
        unstyled_entities.clear();
        unstyled_entities.extend(control_map.iter_entities().map(|(_, entity)| *entity));

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
        let mut cache = world.resource_mut::<ControlRefreshCache>();
        // ps_buffer.recover(pseudo_themes); // Did this above
        cache.recover(
            collected_styles,
            unstyled_entities,
            style_builder,
            dynamic_style_buffers,
            recovered_dynamic_styles,
            ps_buffer,
        );
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
                (
                    cleanup_control_maps,
                    handle_node_attr_changes,
                    refresh_controlled_styles,
                )
                    .chain()
                    .in_set(ControlSet),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
