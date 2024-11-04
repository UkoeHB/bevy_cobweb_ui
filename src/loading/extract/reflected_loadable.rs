use std::any::{type_name, TypeId};
use std::sync::Arc;

use bevy::prelude::*;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) struct ErasedLoadable
{
    pub(crate) type_id: TypeId,
    pub(crate) loadable: ReflectedLoadable,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) enum ReflectedLoadable
{
    Value(Arc<Box<dyn Reflect + 'static>>),
    DeserializationFailed(Arc<CafError>),
}

impl ReflectedLoadable
{
    pub(crate) fn equals(&self, other: &ReflectedLoadable) -> Option<bool>
    {
        let (Self::Value(this), Self::Value(other)) = (self, other) else {
            return Some(false);
        };

        this.reflect_partial_eq(other.as_reflect())
    }

    pub(crate) fn get_value<T: Loadable>(&self, loadable_ref: &SceneRef) -> Option<T>
    {
        match self {
            ReflectedLoadable::Value(loadable) => {
                let Some(new_value) = T::from_reflect(loadable.as_reflect()) else {
                    let hint = Self::make_hint::<T>();
                    tracing::error!("failed reflecting loadable {:?} at path {:?} in file {:?}\n\
                        serialization hint: {}",
                        type_name::<T>(), loadable_ref.path.path, loadable_ref.file, hint.as_str());
                    return None;
                };
                Some(new_value)
            }
            ReflectedLoadable::DeserializationFailed(err) => {
                let hint = Self::make_hint::<T>();
                tracing::error!("failed deserializing loadable {:?} at path {:?} in file {:?}, {:?}\n\
                    serialization hint: {}",
                    type_name::<T>(), loadable_ref.path.path, loadable_ref.file, **err, hint.as_str());
                None
            }
        }
    }

    fn make_hint<T: Loadable>() -> String
    {
        let temp = T::default();
        match CafValue::extract(&temp) {
            Ok(value) => {
                let mut buff = Vec::<u8>::default();
                let mut serializer = DefaultRawSerializer::new(&mut buff);
                value.write_to(&mut serializer).unwrap();
                String::from_utf8(buff).unwrap()
            }
            Err(err) => format!("! hint serialization failed: {:?}", err),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
