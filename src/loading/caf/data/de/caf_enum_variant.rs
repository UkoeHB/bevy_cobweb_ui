use serde::de::{DeserializeSeed, EnumAccess, IntoDeserializer, Unexpected, VariantAccess, Visitor};

use super::{visit_map_ref, visit_tuple_ref, visit_wrapped_erased_ref, ErasedNewtypeStruct};
use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

impl CafEnum
{
    #[cold]
    pub(super) fn unexpected(&self) -> Unexpected
    {
        match &self.variant {
            CafEnumVariant::Unit => Unexpected::UnitVariant,
            CafEnumVariant::Tuple(tuple) => {
                if tuple.entries.len() == 1 {
                    Unexpected::NewtypeVariant
                } else {
                    Unexpected::TupleVariant
                }
            }
            CafEnumVariant::Array(..) => Unexpected::NewtypeVariant,
            CafEnumVariant::Map(..) => Unexpected::StructVariant,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct EnumRefDeserializer<'de>
{
    pub(super) variant: &'de CafEnum,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de>
{
    type Error = CafError;
    type Variant = VariantRefAccess<'de>;

    fn variant_seed<V>(self, seed: V) -> CafResult<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.id.as_str().into_deserializer();
        let visitor = VariantRefAccess { variant: self.variant };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub struct VariantRefAccess<'de>
{
    variant: &'de CafEnum,
}

impl<'de> VariantAccess<'de> for VariantRefAccess<'de>
{
    type Error = CafError;

    fn unit_variant(self) -> CafResult<()>
    {
        match &self.variant.variant {
            CafEnumVariant::Unit => Ok(()),
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
        match &self.variant.variant {
            CafEnumVariant::Tuple(tuple) => {
                if tuple.entries.len() == 1 {
                    seed.deserialize(&tuple.entries[0])
                } else {
                    seed.deserialize(tuple)
                }
            }
            // Special cases: enum variant flattening
            CafEnumVariant::Array(array) => seed.deserialize(array),
            CafEnumVariant::Map(map) => seed.deserialize(map),
            CafEnumVariant::Unit => seed.deserialize(ErasedNewtypeStruct),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match &self.variant.variant {
            CafEnumVariant::Tuple(tuple) => visit_tuple_ref(&tuple.entries, visitor),
            // Special case: flattened
            CafEnumVariant::Unit => {
                if len == 0 {
                    visit_tuple_ref(&[], visitor)
                } else if len == 1 {
                    visit_wrapped_erased_ref(visitor)
                } else {
                    Err(serde::de::Error::invalid_type(
                        self.variant.unexpected(),
                        &"tuple variant",
                    ))
                }
            }
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
        match &self.variant.variant {
            CafEnumVariant::Map(map) => visit_map_ref(&map.entries, visitor),
            // Special case: flattened
            CafEnumVariant::Unit => visit_map_ref(&[], visitor),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"struct variant",
            )),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
