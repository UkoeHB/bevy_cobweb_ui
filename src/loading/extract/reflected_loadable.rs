use std::any::{type_name, TypeId};
use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::TypeRegistry;

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
    Value(Arc<Box<dyn PartialReflect + 'static>>),
    DeserializationFailed(Arc<CobError>),
}

impl ReflectedLoadable
{
    pub(crate) fn equals(&self, other: &ReflectedLoadable) -> Option<bool>
    {
        let (Self::Value(this), Self::Value(other)) = (self, other) else {
            return Some(false);
        };

        this.reflect_partial_eq(other.as_partial_reflect())
    }

    pub(crate) fn get_value<T: Loadable>(&self, scene_ref: &SceneRef, registry: &TypeRegistry) -> Option<T>
    {
        match self {
            ReflectedLoadable::Value(loadable) => {
                let Some(new_value) = T::from_reflect(loadable.as_partial_reflect()) else {
                    let hint = Self::make_hint::<T>(registry);
                    tracing::error!("failed reflecting loadable {:?} at path {:?} in file {:?}\n\
                        serialization hint: {}",
                        type_name::<T>(), scene_ref.path.path, scene_ref.file, hint.as_str());
                    return None;
                };
                Some(new_value)
            }
            ReflectedLoadable::DeserializationFailed(err) => {
                let hint = Self::make_hint::<T>(registry);
                tracing::error!("failed deserializing loadable {:?} at path {:?} in file {:?}, {:?}\n\
                    serialization hint: {}",
                    type_name::<T>(), scene_ref.path.path, scene_ref.file, **err, hint.as_str());
                None
            }
        }
    }

    fn make_hint<T: Loadable>(registry: &TypeRegistry) -> String
    {
        let temp = T::default();
        match CobLoadable::extract_reflect(&temp, registry) {
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
