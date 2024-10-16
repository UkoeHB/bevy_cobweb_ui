use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, Unexpected, VariantAccess, Visitor};

use super::{visit_map_ref, visit_tuple_ref};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl CafEnumVariant
{
    #[cold]
    pub(super) fn unexpected(&self) -> Unexpected
    {
        match self {
            CafEnumVariant::Unit { .. } => Unexpected::UnitVariant,
            CafEnumVariant::Tuple { tuple, .. } => {
                if tuple.entries.len() == 1 {
                    Unexpected::NewtypeVariant
                } else {
                    Unexpected::TupleVariant
                }
            }
            CafEnumVariant::Array { .. } => Unexpected::NewtypeVariant,
            CafEnumVariant::Map { .. } => Unexpected::StructVariant,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct EnumRefDeserializer<'de>
{
    pub(super) variant: &'de CafEnumVariant,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de>
{
    type Error = CafError;
    type Variant = VariantRefAccess<'de>;

    fn variant_seed<V>(self, seed: V) -> CafResult<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.id().into_deserializer();
        let visitor = VariantRefAccess { variant: self.variant };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct VariantRefAccess<'de>
{
    variant: &'de CafEnumVariant,
}

impl<'de> VariantAccess<'de> for VariantRefAccess<'de>
{
    type Error = CafError;

    fn unit_variant(self) -> CafResult<()>
    {
        match self.variant {
            CafEnumVariant::Unit { .. } => Ok(()),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> CafResult<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        match self.variant {
            CafEnumVariant::Tuple { tuple, .. } => {
                if tuple.entries.len() != 1 {
                    Err(serde::de::Error::invalid_type(
                        self.variant.unexpected(),
                        &"newtype variant",
                    ))
                } else {
                    seed.deserialize(&tuple.entries[0])
                }
            }
            // Enum variant array is special case of tuple-variant-of-sequence.
            CafEnumVariant::Array { array, .. } => seed.deserialize(array),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.variant {
            CafEnumVariant::Tuple { tuple, .. } => visit_tuple_ref(tuple, visitor),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.variant {
            CafEnumVariant::Map { map, .. } => visit_map_ref(map, visitor),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"struct variant",
            )),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
