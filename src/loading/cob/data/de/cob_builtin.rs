use std::sync::OnceLock;

use bevy::color::Srgba;
use bevy::ui::Val;
use serde::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn deserialize_builtin<'de, V>(builtin: &'de CobBuiltin, visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    match builtin {
        CobBuiltin::Color(CobHexColor { color, .. }) => visitor.visit_enum(ColorSrgbaAccess { color: *color }),
        CobBuiltin::Val { val, .. } => visitor.visit_enum(ValAccess { val: *val }),
        CobBuiltin::GridValFraction { fraction, .. } => {
            visitor.visit_enum(GridValFractionAccess { fraction: *fraction })
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn visit_raw_repeated_grid_val<'de, V>(builtin_val: &'de CobValue, visitor: V) -> CobResult<V::Value>
where
    V: Visitor<'de>,
{
    let mut deserializer = RawRepeatedGridValSeqDeserializer::new(builtin_val);
    let seq = visitor.visit_seq(&mut deserializer)?;
    if deserializer.remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            deserializer.remaining,
            &"two elements in tuple",
        ))
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: is there an easier way to do this by leveraging the Serialize implementation of Srgba?
struct ColorSrgbaAccess
{
    color: Srgba,
}

impl<'de> EnumAccess<'de> for ColorSrgbaAccess
{
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

struct ColorSrgbaVariantAccess
{
    color: Srgba,
}

impl<'de> VariantAccess<'de> for ColorSrgbaVariantAccess
{
    type Error = CobError;

    fn unit_variant(self) -> CobResult<()>
    {
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

struct SrgbaDeserializer
{
    color: Srgba,
}

impl<'de> serde::Deserializer<'de> for SrgbaDeserializer
{
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

struct ValVariantAccess
{
    val: Val,
}

impl<'de> VariantAccess<'de> for ValVariantAccess
{
    type Error = CobError;

    fn unit_variant(self) -> CobResult<()>
    {
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

struct GridValFractionAccess
{
    fraction: f32,
}

impl<'de> EnumAccess<'de> for GridValFractionAccess
{
    type Error = CobError;
    type Variant = GridValFractionVariantAccess;

    fn variant_seed<V>(self, seed: V) -> CobResult<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = "Fraction";
        let variant = variant.into_deserializer();
        let visitor = GridValFractionVariantAccess { fraction: self.fraction };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}
//-------------------------------------------------------------------------------------------------------------------

struct GridValFractionVariantAccess
{
    fraction: f32,
}

impl<'de> VariantAccess<'de> for GridValFractionVariantAccess
{
    type Error = CobError;

    fn unit_variant(self) -> CobResult<()>
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::NewtypeVariant,
            &"unit variant",
        ))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> CobResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.fraction.into_deserializer())
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::NewtypeVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> CobResult<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::NewtypeVariant,
            &"struct variant",
        ))
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct RawRepeatedGridValSeqDeserializer<'de>
{
    builtin_val: &'de CobValue,
    remaining: usize,
}

impl<'de> RawRepeatedGridValSeqDeserializer<'de>
{
    fn new(builtin_val: &'de CobValue) -> Self
    {
        RawRepeatedGridValSeqDeserializer { builtin_val, remaining: 2 }
    }
}

impl<'de> SeqAccess<'de> for RawRepeatedGridValSeqDeserializer<'de>
{
    type Error = CobError;

    fn next_element_seed<T>(&mut self, seed: T) -> CobResult<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Helper constant since `GridTrackRepetition` doesn't implement `Deserializer`.
        static GRID_TRACK_REPETITION_COUNT_1: OnceLock<CobValue> = OnceLock::new();
        let count_1 = GRID_TRACK_REPETITION_COUNT_1.get_or_init(|| {
            CobValue::Enum(CobEnum {
                fill: CobFill::default(),
                id: CobEnumVariantIdentifier(SmolStr::new_static("Count")),
                variant: CobEnumVariant::Tuple(CobTuple {
                    start_fill: CobFill::default(),
                    entries: vec![CobValue::Number(CobNumber::from(1u64))],
                    end_fill: CobFill::default(),
                }),
            })
        });

        if self.remaining == 2 {
            self.remaining = 1;
            seed.deserialize(count_1).map(Some)
        } else if self.remaining == 1 {
            self.remaining = 0;
            seed.deserialize(self.builtin_val).map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize>
    {
        Some(self.remaining)
    }
}

//-------------------------------------------------------------------------------------------------------------------
