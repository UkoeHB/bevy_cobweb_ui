use bevy::core::Name;
use bevy::ecs::component::ComponentInfo;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, EntityCommand, EntityCommands};
use bevy::ecs::world::{Command, World};
use bevy::hierarchy::Children;
use bevy::log::{info, warn};
use bevy::state::state::{FreelyMutableState, NextState, States};
use bevy::text::{Text, TextSection, TextStyle};
use bevy::window::{CursorIcon, PrimaryWindow, Window};

use crate::flux_interaction::{FluxInteractionStopwatchLock, StopwatchLock};
use crate::prelude::UiUtils;
use crate::theme::prelude::*;

struct SetTextSections
{
    sections: Vec<TextSection>,
}

impl EntityCommand for SetTextSections
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set text sections on entity {}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = self.sections;
    }
}

pub trait SetTextSectionsExt
{
    /// Set text sections for a UI entity
    ///
    /// The [`Text`] component must already exist on the target entity.
    fn set_text_sections(&mut self, sections: Vec<TextSection>) -> &mut Self;
}

impl SetTextSectionsExt for EntityCommands<'_>
{
    fn set_text_sections(&mut self, sections: Vec<TextSection>) -> &mut Self
    {
        self.add(SetTextSections { sections });
        self
    }
}

struct SetText
{
    text: String,
    style: TextStyle,
}

impl EntityCommand for SetText
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Some(mut text) = world.get_mut::<Text>(entity) else {
            warn!(
                "Failed to set text on entity {}: No Text component found!",
                entity
            );
            return;
        };

        text.sections = vec![TextSection::new(self.text, self.style)];
    }
}

pub trait SetTextExt
{
    /// Set text for a UI entity with the given [`TextStyle`]
    ///
    /// The [`Text`] component must already exist on the target entity.
    fn set_text(&mut self, text: impl Into<String>, style: Option<TextStyle>) -> &mut Self;
}

impl SetTextExt for EntityCommands<'_>
{
    fn set_text(&mut self, text: impl Into<String>, style: Option<TextStyle>) -> &mut Self
    {
        self.add(SetText { text: text.into(), style: style.unwrap_or_default() });

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
            warn!(
                "Failed to set text on entity {}: No Text component found!",
                entity
            );
            return;
        };

        let first_section = match text.sections.get(0) {
            Some(section) => TextSection::new(self.text, section.style.clone()),
            None => TextSection::new(self.text, TextStyle::default()),
        };

        text.sections = vec![first_section];
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
        self.add(UpdateText { text: text.into() });

        self
    }
}

// TODO: Move to style and apply to Node's window
struct SetCursor
{
    cursor: CursorIcon,
}

impl Command for SetCursor
{
    fn apply(self, world: &mut World)
    {
        let mut q_window = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let Ok(mut window) = q_window.get_single_mut(world) else {
            return;
        };

        if window.cursor.icon != self.cursor {
            window.cursor.icon = self.cursor;
        }
    }
}

pub trait SetCursorExt<'w, 's, 'a>
{
    /// Set the [`PrimaryWindow`]'s cursor
    fn set_cursor(&mut self, cursor: CursorIcon);
}

impl<'w, 's, 'a> SetCursorExt<'w, 's, 'a> for Commands<'w, 's>
{
    fn set_cursor(&mut self, cursor: CursorIcon)
    {
        self.add(SetCursor { cursor });
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
            info!(
                "{}{}{}── {}",
                padding, child_pipe, component_pipe, debug_infos[i]
            );
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
        self.add(LogHierarchy {
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
        self.add(move |entity, world: &mut World| {
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
        self.add(move |entity, world: &mut World| {
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
        self.add(move |entity, world: &mut World| {
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
        self.add(move |entity, world: &mut World| {
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
        self.add(|world: &mut World| {
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
