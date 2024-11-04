pub mod attribute;
pub mod builder;
pub mod generated;
pub mod manual;

use bevy::{ecs::system::EntityCommands, prelude::*, utils::HashSet};

use sickle_math::lerp::Lerp;

use attribute::AnimatedVals;
use generated::LockableStyleAttribute;

pub mod prelude {
    pub use super::{
        attribute::{AnimatedVals, InteractiveVals},
        builder::StyleBuilder,
        generated::*,
        manual::*,
        *,
    };
}

pub struct UiStyle<'a> {
    commands: EntityCommands<'a>,
}

impl UiStyle<'_> {
    /// Returns the Entity that is the target of all styling commands
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    /// Returns the underlying EntityCommands via reborrow
    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
    }
}

pub trait UiStyleExt {
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
    fn style(&mut self, entity: Entity) -> UiStyle;
}

impl UiStyleExt for Commands<'_, '_> {
    fn style(&mut self, entity: Entity) -> UiStyle {
        UiStyle {
            commands: self.entity(entity),
        }
    }
}

pub struct UiStyleUnchecked<'a> {
    commands: EntityCommands<'a>,
}

impl UiStyleUnchecked<'_> {
    /// Returns the Entity that is the target of all styling commands
    pub fn id(&self) -> Entity {
        self.commands.id()
    }

    /// Returns the underlying EntityCommands via reborrow
    pub fn entity_commands(&mut self) -> EntityCommands {
        self.commands.reborrow()
    }
}

pub trait UiStyleUncheckedExt {
    /// Same as [`UiStyleExt::style`], except styling calls will bypass attribute locks
    fn style_unchecked(&mut self, entity: Entity) -> UiStyleUnchecked;
}

impl UiStyleUncheckedExt for Commands<'_, '_> {
    fn style_unchecked(&mut self, entity: Entity) -> UiStyleUnchecked {
        UiStyleUnchecked {
            commands: self.entity(entity),
        }
    }
}

pub trait LogicalEq<Rhs: ?Sized = Self> {
    fn logical_eq(&self, other: &Rhs) -> bool;

    fn logical_ne(&self, other: &Rhs) -> bool {
        !self.logical_eq(other)
    }
}

/// A set of attributes that should be protected against styling via [`UiStyleExt::style`] commands.
///
/// Used by widgets to protect attributes that are controlled by logic and should not be styled by end users.
#[derive(Component, Debug, Default, Reflect)]
pub struct LockedStyleAttributes(HashSet<LockableStyleAttribute>);

impl LockedStyleAttributes {
    /// Creates a new empty set
    pub fn new() -> Self {
        Self(HashSet::<LockableStyleAttribute>::new())
    }

    /// Creates a new set from the provided set of [`LockableStyleAttribute`]s
    pub fn lock(attributes: impl Into<HashSet<LockableStyleAttribute>>) -> Self {
        Self(attributes.into())
    }

    /// Creates a new set from the provided list of [`LockableStyleAttribute`]s
    pub fn from_vec(attributes: Vec<LockableStyleAttribute>) -> Self {
        let mut set = HashSet::<LockableStyleAttribute>::with_capacity(attributes.len());
        for attribute in attributes.iter() {
            if !set.contains(attribute) {
                set.insert(*attribute);
            }
        }

        Self(set)
    }

    /// Checks whether the set contains the attribute
    pub fn contains(&self, attr: LockableStyleAttribute) -> bool {
        self.0.contains(&attr)
    }
}

impl From<LockableStyleAttribute> for HashSet<LockableStyleAttribute> {
    fn from(value: LockableStyleAttribute) -> Self {
        let mut set = HashSet::<LockableStyleAttribute>::new();
        set.insert(value);
        set
    }
}

/// Dummy stylable attribute used for tracking state changes
///
/// This can be used in animated themes to provide discretized states to interop with logic
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum TrackedStyleState {
    #[default]
    None,
    Transitioning,
    Enter,
    Idle,
    Hover,
    Pressed,
    Released,
    Canceled,
}

impl Lerp for TrackedStyleState {
    fn lerp(&self, to: Self, t: f32) -> Self {
        if t == 0. {
            *self
        } else if t == 1. {
            to
        } else {
            Self::Transitioning
        }
    }
}

impl TrackedStyleState {
    pub fn default_vals() -> AnimatedVals<Self> {
        AnimatedVals {
            idle: Self::Idle,
            hover: Self::Hover.into(),
            press: Self::Pressed.into(),
            cancel: Self::Canceled.into(),
            enter_from: Self::Enter.into(),
            ..default()
        }
    }
}
