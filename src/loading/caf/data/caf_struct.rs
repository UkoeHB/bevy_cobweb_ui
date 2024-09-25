
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafStruct
{
    Unit{
        id: CafTypeIdentifier
    },
    Array{
        id: CafTypeIdentifier,
        array: CafValueArray,
    },
    Tuple{
        id: CafTypeIdentifier,
        tuple: CafValueTuple,
    },
    Map{
        id: CafTypeIdentifier,
        map: CafValueMap,
    }
}

impl CafStruct
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Unit{id} => {
                id.write_to(writer)?;
            }
            Self::Array{id, array} => {
                id.write_to(writer)?;
                array.write_to(writer)?;
            }
            Self::Tuple{id, tuple} => {
                id.write_to(writer)?;
                tuple.write_to(writer)?;
            }
            Self::Map{id, map} => {
                id.write_to(writer)?;
                map.write_to(writer)?;
            }
        }
        Ok(())
    }
}

/*
Parsing:
- no whitespace allowed betwen type id and value
*/

//-------------------------------------------------------------------------------------------------------------------
