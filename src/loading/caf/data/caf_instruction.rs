
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafInstructionIdentifier
{
    pub start_fill: CafFill,
    pub name: SmolStr,
    pub generics: Option<CafGenerics>
}

impl CafInstructionIdentifier
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(self.as_bytes())?;
        Ok(())
    }

    /// The canonical string can be used to access the type in the reflection type registry.
    ///
    /// You can pass a scratch string as input to reuse a string buffer for querying multiple identifiers.
    pub fn to_canonical(&self, scratch: Option<String>) -> Result<String, std::io::Error>
    {
        let mut string = scratch.unwrap_or_default();
        string.clear();
        let mut cursor = Cursor::new(&mut string);
        let error = |e| {
            std::io::Error::other(
                format!("failed writing canonical generics to caf instruction {:?}: {e:?}", *self.name)
            )
        };

        cursor.write(self.name.as_bytes()).map_err(error)?;
        if let Some(generics) = &self.generics {
            generics.write_canonical(&mut cursor).map_err(error)?;
        }

        Ok(string)
    }

    //todo: resolve_constants
    //todo: resolve_macro
}

/*
Parsing:
- identifier is camelcase
*/

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafInstruction
{
    /// Corresponds to a unit struct.
    Unit{
        id: CafInstructionIdentifier
    },
    /// Corresponds to a tuple struct.
    Tuple{
        id: CafInstructionIdentifier,
        tuple: CafTuple,
    },
    /// This is a shorthand and equivalent to a tuple struct of an array.
    Array{
        id: CafInstructionIdentifier,
        array: CafArray,
    },
    /// Corresponds to a plain struct.
    Map{
        id: CafInstructionIdentifier,
        map: CafMap,
    },
    /// Corresponds to an enum.
    Enum{
        id: CafInstructionIdentifier,
        variant: CafEnumVariant
    }
}

impl CafInstruction
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Unit{id} => {
                id.write_to(writer)?;
            }
            Self::Tuple{id, tuple} => {
                id.write_to(writer)?;
                tuple.write_to(writer)?;
            }
            Self::Array{id, array} => {
                id.write_to(writer)?;
                array.write_to(writer)?;
            }
            Self::Map{id, map} => {
                id.write_to(writer)?;
                map.write_to(writer)?;
            }
            Self::Enum{id, variant} => {
                id.write_to(writer)?;
                writer.write("::".as_bytes())?;
                variant.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<serde_json::Value, std::io::Error>
    {
        match *self {
            Self::Unit{..} => {
                // []
                Ok(serde_json::Value::Array(vec![]))
            }
            Self::Tuple{tuple, ..} => {
                // [..tuple items..]
                tuple.to_json()
            }
            Self::Array{array, ..} => {
                // [[..array items..]]
                Ok(serde_json::Value::Array(vec![array.to_json()?]))
            }
            Self::Map{map, ..} => {
                // {..map items..}
                map.to_json()
            }
            Self::Enum{variant, ..} => {
                // .. enum variant ..
                variant.to_json()
            }
        }
    }

    pub fn id(&self) -> &CafInstructionIdentifier
    {
        match self {
            Self::Unit{id} |
            Self::Tuple{id, ..}  |
            Self::Array{id, ..}  |
            Self::Map{id, ..}  |
            Self::Enum{id, ..} => id
        }
    }
}

/*
Parsing:
- no whitespace allowed between identifier and value
*/

//-------------------------------------------------------------------------------------------------------------------
