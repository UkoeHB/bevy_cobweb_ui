use bevy::ecs::bundle::Bundle;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::{Commands, EntityCommands, IntoObserverSystem};
use bevy::prelude::*;

use crate::*;

/// Ghost struct to use as a type filler for root UI nodes.
///
/// i.e. `commands.ui_builder(UiRoot)` to start building without a parent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UiRoot;

/// Used to find a root node where nodes are safe to spawn
///
/// i.e. context menus or floating panels torn off from tab containers look for this to mount.
/// This can be injected manually by the developer to indicate mount points for trees.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct UiContextRoot;

/// The heart of `sickle_ui`
///
/// Holds a number of extension traits that map to widget creation and styling commands.
/// Acquire a builder from commands via `commands.ui_builder(UiRoot)` or
/// `commands.ui_builder(entity)`, where the entity is the UI parent node.
pub struct UiBuilder<'a, T>
{
    commands: Commands<'a, 'a>,
    context: T,
}

impl<'a, T> UiBuilder<'a, T>
{
    /// The current build context
    ///
    /// Actual value depends on the type of the builder, usually it is an Entity.
    /// Widgets interally can use it to pass around their main component or other data.
    pub fn context(&self) -> &T
    {
        &self.context
    }

    /// Return the commands used by the builder.
    pub fn commands(&mut self) -> &mut Commands<'a, 'a>
    {
        &mut self.commands
    }
}

impl<'a, T> UiBuilder<'a, T>
where
    T: Copy,
{
    /// Reborrows self with a shorter lifetime.
    pub fn reborrow(&mut self) -> UiBuilder<T>
    {
        UiBuilder { commands: self.commands.reborrow(), context: self.context }
    }
}

impl UiBuilder<'_, UiRoot>
{
    /// Spawn a bundle as a root node (without parent)
    ///
    /// The returned builder can be used to add children to the newly root node.
    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity>
    {
        let new_entity = self.commands().spawn(bundle).id();

        self.commands().ui_builder(new_entity)
    }
}

/// Trait to reduce duplication of code on UiBuilder Implementation through composition
/// basically, as long as the type has a way of returning its own entity Id, all methods implemented on the
/// UiBuilder becomes available
pub trait UiBuilderGetId
{
    /// The ID (Entity) of the current builder
    fn get_id(&self) -> Entity;
}

impl UiBuilderGetId for Entity
{
    fn get_id(&self) -> Entity
    {
        *self
    }
}

impl<T> UiBuilderGetId for (Entity, T)
{
    fn get_id(&self) -> Entity
    {
        self.0
    }
}

impl<T: UiBuilderGetId> UiBuilder<'_, T>
{
    pub fn id(&self) -> Entity
    {
        self.context().get_id()
    }

    /// The `EntityCommands` of the builder
    ///
    /// Points to the entity currently being built upon (see [`UiBuilder<'_, Entity>::id()`]).
    ///
    /// Panics if the entity doesn't exist.
    pub fn entity_commands(&mut self) -> EntityCommands
    {
        let entity = self.id();
        self.commands().entity(entity)
    }

    /// This allows for using the `EntityCommands` of the builder, and also returning the UiBuilder with context
    /// intact for further processing
    pub fn entity_commands_inplace(&mut self, entity_commands_fn: impl FnOnce(&mut EntityCommands)) -> &mut Self
    {
        let mut ec = self.entity_commands();
        entity_commands_fn(&mut ec);
        self
    }

    /// Styling commands for UI Nodes
    ///
    /// `sickle_ui` exposes functions for all standard bevy styleable attributes.
    /// Manual extension can be done for custom styling needs via extension traits:
    ///
    /// ```rust
    /// pub trait SetMyPropExt {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self;
    /// }
    ///
    /// impl SetMyPropExt for UiStyle<'_> {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self {
    ///         // SetMyProp is assumed to be an EntityCommand
    ///         // Alternatively a closure can be supplied as per a standard bevy command
    ///         // NOTE: All built-in commands structs are public and can be re-used in extensions
    ///         self.entity_commands().add(SetMyProp {
    ///             value
    ///         });
    ///         self
    ///     }
    /// }
    /// ```
    pub fn style(&mut self) -> UiStyle
    {
        let entity = self.id();
        self.commands().style(entity)
    }

    /// This allows for modification of style, and also returning the UiBuilder with context
    /// intact for further processing
    pub fn style_inplace(&mut self, style_fn: impl FnOnce(&mut UiStyle)) -> &mut Self
    {
        let entity = self.id();
        let mut style = self.commands().style(entity);
        style_fn(&mut style);
        self
    }

    /// Same as [`UiBuilder<'_, Entity>::style()`], except style commands bypass possible attribute locks.
    pub fn style_unchecked(&mut self) -> UiStyleUnchecked
    {
        let entity = self.id();
        self.commands().style_unchecked(entity)
    }

    /// Spawn a child node as a child of the current entity identified by [`UiBuilder<'_, Entity>::id()`]
    ///
    /// Does nothing if the entity does not exist.
    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity>
    {
        let mut new_entity = Entity::PLACEHOLDER;

        let entity = self.id();
        if let Ok(mut ec) = self.commands().get_entity(entity) {
            ec.with_children(|parent| {
                new_entity = parent.spawn(bundle).id();
            });
        } else {
            tracing::debug!("ignoring spawn with UiBuilder<Entity>; parent entity {entity:?} does not exist");
        }

        self.commands().ui_builder(new_entity)
    }

    /// Inserts a [`Bundle`] of components to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    ///
    /// Does nothing if the entity does not exist.
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self
    {
        let entity = self.id();
        if let Ok(mut ec) = self.commands().get_entity(entity) {
            ec.insert(bundle);
        }
        self
    }

    /// Insert a [`Name`] component to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    ///
    /// Does nothing if the entity does not exist.
    pub fn named(&mut self, name: impl Into<String>) -> &mut Self
    {
        let entity = self.id();
        if let Ok(mut ec) = self.commands().get_entity(entity) {
            ec.named(name);
        }
        self
    }

    /// Mount an observer to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    ///
    /// Does nothing if the entity does not exist.
    pub fn observe<E: Event, B: Bundle, M>(&mut self, system: impl IntoObserverSystem<E, B, M>) -> &mut Self
    {
        let entity = self.id();
        if let Ok(mut ec) = self.commands().get_entity(entity) {
            ec.observe(system);
        }
        self
    }
}

/// Implementations that are useful for creating nested widgets
impl<T> UiBuilder<'_, (Entity, T)>
{
    /// The extension content of the UiBuilder
    pub fn context_data(&self) -> &T
    {
        &self.context().1
    }
}

pub trait UiBuilderExt
{
    /// A contextual UI Builder, see [`UiBuilder<'a, T>`]
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T>;

    /// A UI Builder for the root of a UI tree.
    fn ui_root(&mut self) -> UiBuilder<UiRoot>
    {
        self.ui_builder(UiRoot)
    }
}

impl UiBuilderExt for Commands<'_, '_>
{
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T>
    {
        UiBuilder { commands: self.reborrow(), context }
    }
}

impl UiBuilderExt for EntityCommands<'_>
{
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T>
    {
        UiBuilder { commands: self.commands(), context }
    }
}
