use bevy::prelude::Deref;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafSceneNodeName(pub SmolStr);

impl CafSceneNodeName
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write("\"".as_bytes())?;
        writer.write(self.as_bytes())?;
        writer.write("\"".as_bytes())?;
        Ok(())
    }
}

// Parsing:
// - String should be snake-case only, starting with a lowercase letter and numbers are optional.

//-------------------------------------------------------------------------------------------------------------------

/// Full instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum CafSceneLayerEntry
{
    Instruction(CafInstruction),
    InstructionMacroCall(CafInstructionMacroCall),
    SceneMacroCall(CafSceneMacroCall),
    /// This is the `..'node_name'` and `..*` syntax.
    SceneMacroParam(CafSceneMacroParam),
    Layer(CafSceneLayer),
}

impl CafSceneLayerEntry
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match self {
            Self::Instruction(entry) => {
                entry.write_to(writer)?;
            }
            Self::InstructionMacroCall(entry) => {
                entry.write_to(writer)?;
            }
            Self::Layer(entry) => {
                entry.write_to(writer)?;
            }
            Self::SceneMacroCall(entry) => {
                entry.write_to(writer)?;
            }
            Self::SceneMacroParam(entry) => {
                entry.write_to(writer)?;
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Instruction(entry), Self::Instruction(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::InstructionMacroCall(entry), Self::InstructionMacroCall(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::Layer(entry), Self::Layer(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::SceneMacroCall(entry), Self::SceneMacroCall(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            (Self::SceneMacroParam(entry), Self::SceneMacroParam(other_entry)) => {
                entry.recover_fill(other_entry);
            }
            _ => (),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafSceneLayer
{
    /// Fill before the layer name.
    ///
    /// Whatespace between the name and most recent newline is used to control scene layer depth.
    pub name_fill: CafFill,
    pub name: CafSceneNodeName,
    pub entries: Vec<CafSceneLayerEntry>,
}

impl CafSceneLayer
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.name_fill.write_to_or_else(writer, "\n")?;
        self.name.write_to(writer)?;
        for entry in self.entries.iter() {
            entry.write_to(writer)?;
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.name_fill.recover(&other.name_fill);
        // name has no fill
        for (entry, other) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other);
        }
    }
}

// Parsing: layer name must have preceding spaces with a newline separating anything else (such as comments).
// - Parsing context must keep track of the layer depth increment in order to place layers in the right positions.
//   - Note that in scene macro defs, the first layer is anonymous so depth tracking needs to be relative to the
//   first child layer encountered.
// - Layer entries should not have the same names unless anonymous.

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CafScenes
{
    pub start_fill: CafFill,
    pub scenes: Vec<CafSceneLayer>,
}

impl CafScenes
{
    pub fn write_to(&self, first_section: bool, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        let space = if first_section { "" } else { "\n\n" };
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write("#scenes".as_bytes())?;
        for scene in self.scenes.iter() {
            scene.write_to(writer)?;
        }
        Ok(())
    }
}

// Parsing: layers cannot contain scene macro params, and layer entries cannot contain macro params.

//-------------------------------------------------------------------------------------------------------------------
