
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafFlattenGroupValueEntry
{
    Value(CafValue),
    KeyValue(CafMapKeyValue)
}

impl CafFlattenGroupValueEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Value(value) => {
                value.write_to(writer)?;
            }
            Self::KeyValue(key_value) => {
                key_value.write_to(key_value)?;
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Value(value), Self::Value(other_value)) => {
                value.recover_fill(other_value);
            }
            (Self::KeyValue(key_value), Self::KeyValue(other_key_value)) => {
                key_value.recover_fill(other_key_value);
            }
            _ => ()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafFlattenGroupVariant
{
    Values(Vec<CafFlattenGroupValueEntry>),
    SceneContext(Vec<CafSceneLayerEntry>)
}

impl CafFlattenGroupVariant
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Values(values) => {
                for value in values.iter() {
                    value.write_to(writer)?;
                }
            }
            Self::SceneContext(layer_entries) => {
                for entry in layer_entries.iter() {
                    entry.write_to(writer)?;
                }
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Values(values), Self::Values(other_values)) => {
                for (value, other_value) in values.iter_mut().zip(other_values.iter()) {
                    value.recover_fill(other_value);
                }
            }
            (Self::SceneContext(entries), Self::SceneContext(other_entries)) => {
                for (entry, other_entry) in entries.iter_mut().zip(other_entries.iter()) {
                    entry.recover_fill(other_entry);
                }
            }
            _ => ()
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafFlattenGroup
{
    /// Fill before opening `\`.
    pub start_fill: CafFill,
    pub variant: CafFlattenGroupVariant,
    /// Fill before ending `\`.
    pub end_fill: CafFill
}

impl CafFlattenGroup
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to(writer)?;
        writer.write('\\'.as_bytes())?;
        self.variant.write_to(writer)?;
        self.end_fill.write_to(writer)?;
        writer.write('\\'.as_bytes())?;
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        self.variant.recover_fill();
        self.end_fill.recover(&other.end_fill);
    }
}

/*
Parsing:
*/

//-------------------------------------------------------------------------------------------------------------------
