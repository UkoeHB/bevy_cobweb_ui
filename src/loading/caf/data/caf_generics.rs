
//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafRustPrimitive
{
    pub fill: CafFill,
    pub primitive: SmolStr,
}

impl CafRustPrimitive
{
    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str) -> Result<(), std::io::Error>
    {
        self.fill.write_to_or_else(writer, space)?;
        writer.write(self.primitive.as_bytes())?;
        Ok(())
    }

    pub fn write_canonical(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write(self.primitive.as_bytes())?;
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.fill.recover(other.fill);
    }
}


// Parsing:
// - Primitive must match one of the known primitives.
// f32|f64|i128|i16|i32|i64|i8|isize|u128|u16|u32|u64|u8|usize|bool|char

//-------------------------------------------------------------------------------------------------------------------

/// Any item that can appear in a generic.
#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafGenericItem
{
    Struct{
        fill: CafFill,
        id: SmolStr,
        generics: Option<CafGenerics>
    },
    Tuple{
        /// Fill before opening `(`.
        fill: CafFill,
        values: Vec<CafGenericValue>,
        /// Fill before closing `)`.
        close_fill: CafFill,
    },
    RustPrimitive(CafRustPrimitive)
}

impl CafGenericItem
{
    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Struct{fill, id, generics} => {
                fill.write_to_or_else(writer, space)?;
                writer.write(id.as_bytes())?;
                if let Some(generics) = generics {
                    generics.write_to(writer)?;
                }
            }
            Self::Tuple{fill, values, close_fill} => {
                fill.write_to_or_else(writer, space)?;
                writer.write('('.as_bytes())?;
                for value in values.iter() {
                    value.write_to(writer)?;
                }
                close_fill.write_to(writer)?;
                writer.write(')'.as_bytes())?;
            }
            Self::RustPrimitive(primitive) => {
                primitive.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn write_canonical(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Struct{id, generics, ..} => {
                writer.write(id.as_bytes())?;
                if let Some(generics) = generics {
                    generics.write_canonical(writer)?;
                }
            }
            Self::Tuple{values, ..} => {
                writer.write('('.as_bytes())?;
                let num_values = values.len();
                for (idx, value) in values.iter().enumerate() {
                    value.write_canonical(writer)?;
                    if idx + 1 < num_values {
                        writer.write(", ".as_bytes())?;
                    }
                }
                writer.write(')'.as_bytes())?;
            }
            Self::RustPrimitive(primitive) => {
                primitive.write_canonical(writer)?;
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Struct{fill, generics, ..}, Self::Struct{fill: other_fill, generics: other_generics, ..}) => {
                fill.recover(other_fill);
                if let (Some(generics), Some(other_generics)) = (generics, other_generics) {
                    generics.recover_fill(other_generics);
                }
            }
            (Self::Tuple{fill, values, close_fill}, Self::Tuple{fill: other_fill, values: other_values, close_fill: other_close_fill}) => {
                fill.recover(other_fill);
                for (value, other_value) in values.iter_mut().zip(other_values.iter()) {
                    value.recover_fill(other_value);
                }
                close_fill.recover(other_close_fill);
            }
            (Self::RustPrimitive(primitive), Self::RustPrimitive(primitive: other_primitive)) => {
                primitive.recover_fill(other_primitive);
            }
        }
    }
}


// Parsing:
// - Primitive must match one of the known primitives (ints, floats, bool).

//-------------------------------------------------------------------------------------------------------------------

/// Note that constants and macros are unavailable inside generics. Only macro type params can be used.
#[derive(Debug, Clone, PartialEq, Deref)]
pub enum CafGenericValue
{
    Item(CafGenericItem),
    MacroParam(CafMacroParam),
}

impl CafGenericValue
{
    pub fn write_to_with_space(&self, writer: &mut impl std::io::Write, space: &str) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Item(val) => {
                val.write_to_with_space(writer, space)?;
            }
            Self::MacroParam(val) => {
                val.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn write_canonical(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        match *self {
            Self::Item(item) => {
                item.write_canonical(writer)?;
            }
            Self::MacroParam(param) => {
                // This error probably indicates a bug. Please report it so it can be fixed!
                return Err(std::io::Error::other(format!("generic contains unresolved macro param {:?}", param)));
            }
        }
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Item(val), Self::Item(other_val)) => {
                val.recover_fill(other_val);
            }
            (Self::MacroParam(val), Self::MacroParam(other_val)) => {
                val.recover_fill(other_val);
            }
            _ => ()
        }
    }

    //todo: resolve_macro
    // - Only macro params tied to a macro type param def can be used.
}

/*
Parsing:
- Macro param should be non-optional.
*/

//-------------------------------------------------------------------------------------------------------------------

/// Note that constants and macros are unavailable inside generics. Only macro type params can be used.
#[derive(Debug, Clone, PartialEq, Deref)]
pub struct CafGenerics
{
    /// Fill before opening `<`.
    pub open_fill: CafFill,
    /// Each of these values is expected to take care of its own fill.
    pub values: Vec<CafGenericValue>,
    /// Fill before closing `>`.
    pub close_fill: CafFill,
}

impl CafGenerics
{
    pub fn write_to(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        self.open_fill.write_to(writer)?;
        writer.write('<'.as_bytes())?;
        for (idx, generic) in self.values.iter().enumerate() {
            if idx == 0 {
                generic.write_to_with_space(writer, "")?;
            } else {
                generic.write_to_with_space(writer, ", ")?;
            }
        }
        self.close_fill.write_to(writer)?;
        writer.write('>'.as_bytes())?;
        Ok(())
    }

    /// Assembles generic into a clean sequence of items separated by `, `.
    pub fn write_canonical(&self, writer: &mut impl std::io::Write) -> Result<(), std::io::Error>
    {
        writer.write('<'.as_bytes())?;
        let num_values = self.values.len();
        for (idx, value) in self.values.iter().enumerate() {
            value.write_canonical(writer)?;
            if idx + 1 < num_values {
                writer.write(", ".as_bytes())?;
            }
        }
        writer.write('>'.as_bytes())?;
        Ok(())
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.open_fill.recover(&other.open_fill);
        // TODO: search for equal pairing instead?
        for (value, other_value) in self.values.iter_mut().zip(other.values.iter()) {
            value.recover(other_value);
        }
        self.close_fill.recover(&other.close_fill);
    }
}

/*
Parsing: lowercase identifiers, can be a sequence separated by '.' and not ending or starting with '.'
*/

//-------------------------------------------------------------------------------------------------------------------
