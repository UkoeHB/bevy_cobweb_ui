
//-------------------------------------------------------------------------------------------------------------------

impl CafEnumVariant
{
    #[cold]
    pub(crate) fn unexpected(&self) -> Unexpected {
        match self {
            CafEnumVariant::Unit{..} => Unexpected::UnitVariant,
            CafEnumVariant::Tuple{entries, ..} => {
                if entries.len() == 1 {
                    Unexpected::NewtypeVariant
                } else {
                    Unexpected::TupleVariant
                }
            }
            CafEnumVariant::Array{..} => Unexpected::NewtypeVariant,
            CafEnumVariant::Map{..} => Unexpected::StructVariant
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct EnumRefDeserializer<'de>
{
    variant: &'de CafEnumVariant,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de>
{
    type Error = CafError;
    type Variant = VariantRefDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.id().into_deserializer();
        let visitor = VariantRefDeserializer { variant: self.variant };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct VariantRefDeserializer<'de>
{
    variant: &'de CafEnumVariant,
}

impl<'de> VariantAccess<'de> for VariantRefDeserializer<'de>
{
    type Error = CafError;

    fn unit_variant(self) -> CafResult<()> {
        match self.variant {
            CafEnumVariant::Unit{ .. } => Ok(()),
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
        match variant {
            CafEnumVariant::Tuple{ .., tuple } => {
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
            CafEnumVariant::Array{ .., array } => visit_array_ref(array, visitor),
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
        match variant {
            CafEnumVariant::Tuple{ .., tuple } => visit_tuple_ref(tuple, visitor),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> CafResult<V::Value>
    where
        V: Visitor<'de>,
    {
        match variant {
            CafEnumVariant::Map{ .., map } => visit_map_ref(map, visitor),
            _ => Err(serde::de::Error::invalid_type(
                self.variant.unexpected(),
                &"struct variant",
            )),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
