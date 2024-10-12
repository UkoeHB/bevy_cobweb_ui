use bevy::color::Srgba;
use bevy::ui::Val;
use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, Unexpected, VariantAccess, Visitor};

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn deserialize_builtin<'de, V>(builtin: &'de CafBuiltin, visitor: V) -> CafResult<V::Value>
where
    V: Visitor<'de>,
{
    match builtin {
        CafBuiltin::Color(CafHexColor { color, .. }) => visitor.visit_map(SrgbaAccess::new(*color)),
        CafBuiltin::Val { val, .. } => visitor.visit_enum(ValAccess { val: *val }),
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: is there an easier way to do this by leveraging the Serialize implementation of Srgba?
struct SrgbaAccess
{
    next: usize,
    color: Srgba,
    value: Option<f32>,
}

impl SrgbaAccess
{
    fn new(color: Srgba) -> Self
    {
        SrgbaAccess { next: 0, color, value: None }
    }
}

impl<'de> MapAccess<'de> for SrgbaAccess
{
    type Error = CafError;

    fn next_key_seed<T>(&mut self, seed: T) -> CafResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.next {
            0 => {
                self.value = Some(self.color.red);
                self.next += 1;
                let name = "red".into_deserializer();
                seed.deserialize(name).map(Some)
            }
            1 => {
                self.value = Some(self.color.green);
                self.next += 1;
                let name = "green".into_deserializer();
                seed.deserialize(name).map(Some)
            }
            2 => {
                self.value = Some(self.color.blue);
                self.next += 1;
                let name = "blue".into_deserializer();
                seed.deserialize(name).map(Some)
            }
            3 => {
                self.value = Some(self.color.alpha);
                self.next += 1;
                let name = "alpha".into_deserializer();
                seed.deserialize(name).map(Some)
            }
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Other(&"no remaining Srgba fields"),
                &"Srgba fully constructed",
            )),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> CafResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value.into_deserializer()),
            None => Err(serde::de::Error::custom("Srgba field is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        Some(4usize.saturating_sub(self.next))
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: is there an easier way to do this by leveraging the Serialize implementation of Val?
struct ValAccess
{
    val: Val,
}

impl<'de> EnumAccess<'de> for ValAccess
{
    type Error = CafError;
    type Variant = ValVariantAccess;

    fn variant_seed<V>(self, seed: V) -> CafResult<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = match self.val {
            Val::Auto => "Auto",
            Val::Px(_) => "Px",
            Val::Percent(_) => "Percent",
            Val::Vw(_) => "Vw",
            Val::Vh(_) => "Vh",
            Val::VMin(_) => "VMin",
            Val::VMax(_) => "VMax",
        };
        let variant = variant.into_deserializer();
        let visitor = ValVariantAccess { val: self.val };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct ValVariantAccess
{
    val: Val,
}

impl<'de> VariantAccess<'de> for ValVariantAccess
{
    type Error = CafError;

    fn unit_variant(self) -> CafResult<()>
    {
        match self.val {
            Val::Auto => Ok(()),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::NewtypeVariant,
                &"unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> CafResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.val {
            Val::Px(f) | Val::Percent(f) | Val::Vw(f) | Val::Vh(f) | Val::VMin(f) | Val::VMax(f) => {
                seed.deserialize(f.into_deserializer())
            }
            Val::Auto => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.val {
            Val::Auto => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::NewtypeVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.val {
            Val::Auto => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::NewtypeVariant,
                &"struct variant",
            )),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
