use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Instruction that inserts a [`PickingBehavior`] component to the entity.
///
/// Defaults to [`Self::Pass`], which matches the default behavior when there is no `PickingBehavior` component.
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

impl Into<PickingBehavior> for Picking
{
    fn into(self) -> PickingBehavior
    {
        match self {
            Self::Ignore => PickingBehavior { should_block_lower: false, is_hoverable: false },
            Self::Pass => PickingBehavior { should_block_lower: false, is_hoverable: true },
            Self::Block => PickingBehavior { should_block_lower: true, is_hoverable: false },
            Self::Sink => PickingBehavior { should_block_lower: true, is_hoverable: true },
        }
    }
}

impl From<PickingBehavior> for Picking
{
    fn from(behavior: PickingBehavior) -> Self
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
        let behavior: PickingBehavior = self.into();
        emut.insert(behavior);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let Ok(mut emut) = world.get_entity_mut(entity) else { return };
        emut.remove::<PickingBehavior>();
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
