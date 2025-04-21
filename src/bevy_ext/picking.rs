use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Instruction that inserts a [`Pickable`] component to the entity.
///
/// Defaults to [`Self::Pass`], which matches the default behavior when there is no `Pickable` component.
#[derive(Reflect, Default, Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum Picking
{
    /// Pointers will 'pass through' the entity, and won't send events to the entity.
    Ignore,
    /// Pointers will 'pass through' the entity, and will send events to the entity.
    #[default]
    Pass,
    /// Pointers will get stuck on the entity, but won't send events to the entity.
    Block,
    /// Pointers will get stuck on the entity, and will send events to the entity.
    Sink,
}

impl Into<Pickable> for Picking
{
    fn into(self) -> Pickable
    {
        match self {
            Self::Ignore => Pickable { should_block_lower: false, is_hoverable: false },
            Self::Pass => Pickable { should_block_lower: false, is_hoverable: true },
            Self::Block => Pickable { should_block_lower: true, is_hoverable: false },
            Self::Sink => Pickable { should_block_lower: true, is_hoverable: true },
        }
    }
}

impl From<Pickable> for Picking
{
    fn from(behavior: Pickable) -> Self
    {
        match (behavior.should_block_lower, behavior.is_hoverable) {
            (false, false) => Self::Ignore,
            (false, true) => Self::Pass,
            (true, false) => Self::Block,
            (true, true) => Self::Sink,
        }
    }
}

impl Instruction for Picking
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        let behavior: Pickable = self.into();
        emut.insert(behavior);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<Pickable>();
    }
}

impl StaticAttribute for Picking
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

impl ResponsiveAttribute for Picking {}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct PickingPlugin;

impl Plugin for PickingPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_responsive::<Picking>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
