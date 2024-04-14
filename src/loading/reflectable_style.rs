use bevy::prelude::*;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Trait representing types that can be loaded by [`StyleSheet`].
pub trait ReflectableStyle:
    Reflect + FromReflect + PartialEq + Clone + Default + Serialize + for<'de> Deserialize<'de>
{
}

impl<T> ReflectableStyle for T where
    T: Reflect + FromReflect + PartialEq + Clone + Default + Serialize + for<'de> Deserialize<'de>
{
}

//-------------------------------------------------------------------------------------------------------------------
