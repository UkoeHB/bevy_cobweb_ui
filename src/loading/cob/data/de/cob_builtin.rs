use bevy::color::Srgba;
use bevy::ui::Val;
use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, Unexpected, VariantAccess, Visitor};
use serde::forward_to_deserialize_any;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn deserialize_builtin<'de, V>(builtin: &'de CobBuiltin, visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    match builtin {
        CobBuiltin::Color(CobHexColor { color, .. }) => visitor.visit_enum(ColorSrgbaAccess { color: *color }),
        CobBuiltin::Val { val, .. } => visitor.visit_enum(ValAccess { val: *val }),
        CobBuiltin::GridVal { val, .. } => visitor.visit_enum(GridValAccess { val: *val }),
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: is there an easier way to do this by leveraging the Serialize implementation of Srgba?
struct ColorSrgbaAccess {
    color: Srgba,
}

impl<'de> EnumAccess<'de> for ColorSrgbaAccess {
    type Error = CobError;
    type Variant = ColorSrgbaVariantAccess;

    fn variant_seed<V>(self, seed: V) -> CobResult<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = "Srgba".into_deserializer();
        let visitor = ColorSrgbaVariantAccess { color: self.color };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct ColorSrgbaVariantAccess {
    color: Srgba,
}

impl<'de> VariantAccess<'de> for ColorSrgbaVariantAccess {
    type Error = CobError;

    fn unit_variant(self) -> CobResult<()> {
        Err(serde::de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant Srgba",
        ))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> CobResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(SrgbaDeserializer { color: self.color })
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::TupleVariant,
            &"newtype variant Srgba",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::StructVariant,
            &"newtype variant Srgba",
        ))
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct SrgbaDeserializer {
    color: Srgba,
}

impl<'de> serde::Deserializer<'de> for SrgbaDeserializer {
    type Error = CobError;

    fn deserialize_any<V>(self, visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(SrgbaAccess::new(self.color))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit seq tuple
        map identifier ignored_any
        enum newtype_struct unit_struct tuple_struct struct
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct SrgbaAccess {
    next: usize,
    color: Srgba,
    value: Option<f32>,
}

impl SrgbaAccess {
    fn new(color: Srgba) -> Self {
        SrgbaAccess { next: 0, color, value: None }
    }
}

impl<'de> MapAccess<'de> for SrgbaAccess {
    type Error = CobError;

    fn next_key_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
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
            _ => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> CobResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value.into_deserializer()),
            None => Err(serde::de::Error::custom("Srgba field is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(4usize.saturating_sub(self.next))
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: is there an easier way to do this by leveraging the Serialize implementation of Val?
struct ValAccess {
    val: Val,
}

impl<'de> EnumAccess<'de> for ValAccess {
    type Error = CobError;
    type Variant = ValVariantAccess;

    fn variant_seed<V>(self, seed: V) -> CobResult<(V::Value, Self::Variant)>
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

struct ValVariantAccess {
    val: Val,
}

impl<'de> VariantAccess<'de> for ValVariantAccess {
    type Error = CobError;

    fn unit_variant(self) -> CobResult<()> {
        match self.val {
            Val::Auto => Ok(()),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::NewtypeVariant,
                &"unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> CobResult<T::Value>
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

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> CobResult<V::Value>
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

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> CobResult<V::Value>
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

struct GridValAccess {
    val: GridVal,
}

impl<'de> EnumAccess<'de> for GridValAccess {
    type Error = CobError;
    type Variant = GridValVariantAccess;

    fn variant_seed<V>(self, seed: V) -> CobResult<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = match self.val {
            GridVal::Auto => "Auto",
            GridVal::Px(_) => "Px",
            GridVal::Percent(_) => "Percent",
            GridVal::Vw(_) => "Vw",
            GridVal::Vh(_) => "Vh",
            GridVal::VMin(_) => "VMin",
            GridVal::VMax(_) => "VMax",
            GridVal::Fr(_) => "Fr",
        };
        let variant = variant.into_deserializer();
        let visitor = GridValVariantAccess { val: self.val };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}
//-------------------------------------------------------------------------------------------------------------------

struct GridValVariantAccess {
    val: GridVal,
}

impl<'de> VariantAccess<'de> for GridValVariantAccess {
    type Error = CobError;

    fn unit_variant(self) -> CobResult<()> {
        match self.val {
            GridVal::Auto => Ok(()),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::NewtypeVariant,
                &"unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> CobResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.val {
            GridVal::Px(f)
            | GridVal::Percent(f)
            | GridVal::Vw(f)
            | GridVal::Vh(f)
            | GridVal::VMin(f)
            | GridVal::VMax(f)
            | GridVal::Fr(f) => seed.deserialize(f.into_deserializer()),
            GridVal::Auto => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.val {
            GridVal::Auto => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
            _ => Err(serde::de::Error::invalid_type(
                Unexpected::NewtypeVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.val {
            GridVal::Auto => Err(serde::de::Error::invalid_type(
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
