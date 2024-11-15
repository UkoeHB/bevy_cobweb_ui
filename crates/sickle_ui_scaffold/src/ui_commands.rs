use bevy::core::Name;
use bevy::ecs::component::ComponentInfo;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, EntityCommand, EntityCommands};
use bevy::ecs::world::World;
use bevy::hierarchy::Children;
use bevy::log::{info, warn};
use bevy::prelude::{Color, Component, Mut, Text, TextColor, TextFont};
use bevy::state::state::{FreelyMutableState, NextState, States};

use crate::attributes::prelude::*;
use crate::flux_interaction::{FluxInteractionStopwatchLock, StopwatchLock};
use crate::prelude::UiUtils;

struct SetText
{
    text: String,
    font: TextFont,
    color: Color,
}

fn get_component_or_warn<T: Component>(entity: Entity, world: &mut World) -> Option<Mut<T>>
{
    let Some(comp) = world.get_mut::<T>(entity) else {
        warn!("Expected component not found on entity {}!", entity);
        return None;
    };

    Some(comp)
}

impl EntityCommand for SetText
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut text) = get_component_or_warn::<Text>(entity, world) else {
            return;
        };
        text.0 = self.text;
        let Some(mut font) = get_component_or_warn::<TextFont>(entity, world) else {
            return;
        };
        *font = self.font;
        let Some(mut color) = get_component_or_warn::<TextColor>(entity, world) else {
            return;
        };
        *color = TextColor(self.color);
    }
}

pub trait SetTextExt
{
    /// Set text for a UI entity with the given [`TextStyle`]
    ///
    /// The [`Text`] component must already exist on the target entity.
    fn set_text(&mut self, text: impl Into<String>, font: Option<TextFont>, color: Option<Color>) -> &mut Self;
}

impl SetTextExt for EntityCommands<'_>
{
    fn set_text(&mut self, text: impl Into<String>, font: Option<TextFont>, color: Option<Color>) -> &mut Self
    {
        self.queue(SetText {
            text: text.into(),
            font: font.unwrap_or_default(),
            color: color.unwrap_or_default(),
        });

        self
    }
}

struct UpdateText
{
    text: String,
}

impl EntityCommand for UpdateText
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!("Failed to set text on entity {}: No Text component found!", entity);
            return;
        };

        text.0 = self.text;
    }
}

pub trait UpdateTextExt
{
    /// Update an entity's [`Text`]
    ///
    /// The [`Text`] component must already exist.
    fn update_text(&mut self, text: impl Into<String>) -> &mut Self;
}

impl UpdateTextExt for EntityCommands<'_>
{
    fn update_text(&mut self, text: impl Into<String>) -> &mut Self
    {
        self.queue(UpdateText { text: text.into() });

        self
    }
}

struct LogHierarchy
{
    level: usize,
    is_last: bool,
    trace_levels: Vec<usize>,
    component_filter: Option<fn(ComponentInfo) -> bool>,
}

impl EntityCommand for LogHierarchy
{
    fn apply(self, id: Entity, world: &mut World)
    {
        let mut children_ids: Vec<Entity> = Vec::new();
        if let Some(children) = world.get::<Children>(id) {
            children_ids = children.iter().map(|child| *child).collect();
        }

        let filter = self.component_filter;
        let debug_infos: Vec<_> = world
            .inspect_entity(id)
            .into_iter()
            .filter(|component_info| {
                if let Some(filter) = filter {
                    filter((*component_info).clone())
                } else {
                    true
                }
            })
            .map(UiUtils::simplify_component_name)
            .collect();

        let prefix = if self.is_last { "╚" } else { "╠" };
        let mut padding_parts: Vec<&str> = Vec::with_capacity(self.level);
        for i in 0..self.level {
            let should_trace = i > 0 && self.trace_levels.contains(&(i - 1));

            padding_parts.push(match should_trace {
                true => "  ║ ",
                false => "    ",
            });
        }

        let padding = padding_parts.join("");
        let name = match world.get::<Name>(id) {
            Some(name) => format!("[{}] {}", id, name),
            None => format!("Entity {}", id),
        };
        let entity_text = format!("{}  {}══ {} ", padding, prefix, name);
        let has_children = children_ids.len() > 0;

        info!("{}", entity_text);
        for i in 0..debug_infos.len() {
            let is_last = i == (debug_infos.len() - 1);
            let component_pipe = if is_last { "└" } else { "├" };
            let child_pipe = if self.is_last {
                if has_children {
                    "      ║      "
                } else {
                    "             "
                }
            } else {
                if has_children {
                    "  ║   ║      "
                } else {
                    "  ║          "
                }
            };
            info!("{}{}{}── {}", padding, child_pipe, component_pipe, debug_infos[i]);
        }

        if children_ids.len() > 0 {
            let next_level = self.level + 1;

            for i in 0..children_ids.len() {
                let child = children_ids[i];
                let is_last = i == (children_ids.len() - 1);
                let mut trace_levels = self.trace_levels.clone();
                if !is_last {
                    trace_levels.push(self.level);
                }

                LogHierarchy {
                    level: next_level,
                    is_last,
                    trace_levels,
                    component_filter: self.component_filter,
                }
                .apply(child, world);
            }
        }
    }
}

pub trait LogHierarchyExt
{
    /// Logs the hierarchy of the entity along with the component of each entity in the tree.
    /// Components listed can be optionally filtered by supplying a `component_filter`
    ///
    /// ## Example
    /// ``` rust
    /// commands.entity(parent_id).log_hierarchy(Some(|info| {
    ///     info.name().contains("Node")
    /// }));
    /// ```
    /// ## Output Example
    /// <pre>
    /// ╚══ Entity 254v2:
    ///     ║      └── Node
    ///     ╠══ Entity 252v2:
    ///     ║   ║      └── Node
    ///     ║   ╚══ Entity 158v2:
    ///     ║       ║      └── Node
    ///     ║       ╠══ Entity 159v2:
    ///     ║       ║   ║      └── Node
    ///     ║       ║   ╚══ Entity 286v1:
    ///     ║       ║              └── Node
    ///     ║       ╚══ Entity 287v1:
    ///     ║                  └── Node
    ///     ╚══ Entity 292v1:
    ///                └── Node
    /// </pre>
    fn log_hierarchy(&mut self, component_filter: Option<fn(ComponentInfo) -> bool>) -> &mut Self;
}

impl LogHierarchyExt for EntityCommands<'_>
{
    fn log_hierarchy(&mut self, component_filter: Option<fn(ComponentInfo) -> bool>) -> &mut Self
    {
        self.queue(LogHierarchy {
            level: 0,
            is_last: true,
            trace_levels: vec![],
            component_filter,
        });
        self
    }
}

// Adopted from @brandonreinhart
pub trait EntityCommandsNamedExt
{
    /// Name the entity by inserting a [`Name`] component with the given string
    fn named(&mut self, name: impl Into<String>) -> &mut Self;
}

impl EntityCommandsNamedExt for EntityCommands<'_>
{
    fn named(&mut self, name: impl Into<String>) -> &mut Self
    {
        self.insert(Name::new(name.into()))
    }
}

pub trait ManageFluxInteractionStopwatchLockExt
{
    /// Set the [`FluxInteractionStopwatchLock`] of the entity to the given duration
    ///
    /// This in turn will keep the [`crate::flux_interaction::FluxInteractionStopwatch`] for the given period.
    fn lock_stopwatch(&mut self, owner: &'static str, duration: StopwatchLock) -> &mut Self;

    /// Release the [`FluxInteractionStopwatchLock`] of the entity
    ///
    /// The [`crate::flux_interaction::FluxInteractionStopwatch`] will cleaned up normally
    fn try_release_stopwatch_lock(&mut self, lock_of: &'static str) -> &mut Self;
}

impl ManageFluxInteractionStopwatchLockExt for EntityCommands<'_>
{
    fn lock_stopwatch(&mut self, owner: &'static str, duration: StopwatchLock) -> &mut Self
    {
        self.queue(move |entity, world: &mut World| {
            if let Some(mut lock) = world.get_mut::<FluxInteractionStopwatchLock>(entity) {
                lock.lock(owner, duration);
            } else {
                let mut lock = FluxInteractionStopwatchLock::new();
                lock.lock(owner, duration);
                world.entity_mut(entity).insert(lock);
            }
        });
        self
    }

    fn try_release_stopwatch_lock(&mut self, lock_of: &'static str) -> &mut Self
    {
        self.queue(move |entity, world: &mut World| {
            if let Some(mut lock) = world.get_mut::<FluxInteractionStopwatchLock>(entity) {
                lock.release(lock_of);
            }
        });
        self
    }
}

pub trait ManagePseudoStateExt
{
    /// Add a [`PseudoState`] to the entity if it doesn't have it already
    ///
    /// Will insert the carrier [`PseudoStates`] component if necessary.
    /// Will not trigger change detection if the state is already in the collection.
    fn add_pseudo_state(&mut self, state: PseudoState) -> &mut Self;

    /// Removes a [`PseudoState`] from the entity
    ///
    /// Does not remove the [`PseudoStates`] even if it is empty as a result of the removal.
    fn remove_pseudo_state(&mut self, state: PseudoState) -> &mut Self;
}

impl ManagePseudoStateExt for EntityCommands<'_>
{
    fn add_pseudo_state(&mut self, state: PseudoState) -> &mut Self
    {
        self.queue(move |entity, world: &mut World| {
            let pseudo_states = world.get_mut::<PseudoStates>(entity);

            if let Some(mut pseudo_states) = pseudo_states {
                // NOTE: we must check here, as calling the add fn will trigger change detection
                if !pseudo_states.has(&state) {
                    pseudo_states.add(state);
                }
            } else {
                let mut pseudo_states = PseudoStates::new();
                pseudo_states.add(state);

                world.entity_mut(entity).insert(pseudo_states);
            }
        });
        self
    }

    fn remove_pseudo_state(&mut self, state: PseudoState) -> &mut Self
    {
        self.queue(move |entity, world: &mut World| {
            let Some(mut pseudo_states) = world.get_mut::<PseudoStates>(entity) else {
                return;
            };

            // NOTE: we must check here, as calling the remove fn will trigger change detection
            if pseudo_states.has(&state) {
                pseudo_states.remove(state);
            }
        });

        self
    }
}

pub trait UpdateStatesExt<'w, 's, 'a>
{
    // TODO: deprecate in favor of bevy's own
    // #[deprecated(
    //     since = "0.3.0",
    //     note = "please use bevy's `commands.set_state` instead"
    // )]
    /// Update a state to a new value via [`NextState`]
    fn next_state<C: States + FreelyMutableState>(&mut self, state: C);
}

impl<'w, 's, 'a> UpdateStatesExt<'w, 's, 'a> for Commands<'w, 's>
{
    fn next_state<C: States + FreelyMutableState>(&mut self, state: C)
    {
        self.queue(|world: &mut World| {
            if let Some(mut old_state) = world.get_resource_mut::<NextState<C>>() {
                old_state.set(state);
            } else {
                warn!(
                    "Failed to set state: {}, state not initialized!",
                    std::any::type_name::<C>()
                );
            }
        });
    }
}
