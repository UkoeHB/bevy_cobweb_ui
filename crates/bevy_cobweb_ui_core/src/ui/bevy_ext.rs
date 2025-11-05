use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy_slow_text_outline::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for BackgroundColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for BorderColor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

impl Instruction for FocusPolicy
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let policy: FocusPolicy = self.into();
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(policy);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<FocusPolicy>();
        });
    }
}

impl Splattable for BorderColor
{
    type Splat = Color;

    fn splat(splat: Self::Splat) -> Self
    {
        splat.into()
    }

    fn splat_value(self) -> Option<Self::Splat>
    {
        Some(self.top)
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for ZIndex
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<ZIndex>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for GlobalZIndex
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<GlobalZIndex>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for Visibility
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let visibility: Visibility = self.into();
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(visibility);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Visibility>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl Instruction for TextOutline
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut e| {
            e.remove::<Self>();
        });
    }
}

//-------------------------------------------------------------------------------------------------------------------
