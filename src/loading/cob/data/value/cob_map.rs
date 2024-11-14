use nom::character::complete::char;
use nom::Parser;
use smol_str::SmolStr;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobMapKey
{
    Value(CobValue),
    FieldName
    {
        fill: CobFill,
        name: SmolStr,
    },
}

impl CobMapKey
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        match self {
            Self::Value(value) => {
                value.write_to_with_space(writer, space)?;
            }
            Self::FieldName { fill, name } => {
                fill.write_to_or_else(writer, space)?;
                writer.write_bytes(name.as_bytes())?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        // Try to parse value first in case it's a field-name-like value such as 'true' or 'none'.
        let fill = match CobValue::try_parse(fill, content)? {
            (Some(value), next_fill, remaining) => return Ok((Some(Self::Value(value)), next_fill, remaining)),
            (None, fill, _) => fill,
        };
        match snake_identifier(content) {
            Ok((remaining, id)) => {
                let (next_fill, remaining) = CobFill::parse(remaining);
                Ok((
                    Some(Self::FieldName { fill, name: SmolStr::from(*id.fragment()) }),
                    next_fill,
                    remaining,
                ))
            }
            _ => Ok((None, fill, content)),
        }
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::Value(value), Self::Value(other_value)) => {
                value.recover_fill(other_value);
            }
            (Self::FieldName { fill, .. }, Self::FieldName { fill: other_fill, .. }) => {
                fill.recover(other_fill);
            }
            _ => (),
        }
    }

    pub fn resolve(&mut self, constants: &ConstantsBuffer) -> Result<(), String>
    {
        match self {
            Self::Value(value) => {
                if let Some(_) = value.resolve(constants)? {
                    let err_msg = match value {
                        CobValue::Constant(constant) => {
                            format!("constant ${} in a map entry's key points to value group \
                            but only plain values are allowed", constant.path.as_str())
                        }
                        _ => format!("{{unknown source}} in a map entry's key points to value group \
                            but only plain values are allowed"),
                    };
                    return Err(err_msg);
                }
            }
            Self::FieldName { .. } => (),
        }

        Ok(())
    }

    pub fn value(value: CobValue) -> Self
    {
        Self::Value(value)
    }

    pub fn field_name(name: impl AsRef<str>) -> Self
    {
        Self::FieldName { fill: CobFill::default(), name: SmolStr::from(name.as_ref()) }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobMapKeyValue
{
    pub key: CobMapKey,
    pub semicolon_fill: CobFill,
    pub value: CobValue,
}

impl CobMapKeyValue
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.key.write_to_with_space(writer, space)?;
        self.semicolon_fill.write_to(writer)?;
        writer.write_bytes(":".as_bytes())?;
        self.value.write_to(writer)?;
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let (maybe_key, semicolon_fill, remaining) = CobMapKey::try_parse(fill, content)?;
        let Some(key) = maybe_key else { return Ok((None, semicolon_fill, content)) };
        // Allow failure on missing `:` in case we are inside a value group where there can be either single values
        // or map entries.
        let remaining = match char::<_, ()>(':').parse(remaining) {
            Ok((remaining, _)) => remaining,
            Err(_) => return Ok((None, semicolon_fill, content)),
        };
        let (value_fill, remaining) = CobFill::parse(remaining);
        let (Some(value), next_fill, remaining) = CobValue::try_parse(value_fill, remaining)? else {
            tracing::warn!("failed parsing value for map entry at {}; no valid value found", get_location(remaining));
            return Err(span_verify_error(content));
        };
        Ok((Some(Self { key, semicolon_fill, value }), next_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.key.recover_fill(&other.key);
        self.semicolon_fill.recover(&other.semicolon_fill);
        self.value.recover_fill(&other.value);
    }

    pub fn resolve(&mut self, constants: &ConstantsBuffer) -> Result<(), String>
    {
        self.key.resolve(constants)?;
        if let Some(_) = self.value.resolve(constants)? {
            let err_msg = match &self.value {
                CobValue::Constant(constant) => {
                    format!("constant ${} in a map entry's value points to value group \
                    but only plain values are allowed", constant.path.as_str())
                }
                _ => format!("{{unknown source}} in a map entry's value points to value group \
                    but only plain values are allowed"),
            };
            return Err(err_msg);
        }

        Ok(())
    }

    pub fn struct_field(key: &str, value: CobValue) -> Self
    {
        Self {
            key: CobMapKey::field_name(key),
            semicolon_fill: CobFill::default(),
            value,
        }
    }

    pub fn map_entry(key: CobValue, value: CobValue) -> Self
    {
        Self {
            key: CobMapKey::value(key),
            semicolon_fill: CobFill::default(),
            value,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CobMapEntry
{
    KeyValue(CobMapKeyValue),
    Constant(CobConstant),
    /// Only catch-all params are allowed.
    MacroParam(CobMacroParam),
}

impl CobMapEntry
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        match self {
            Self::KeyValue(keyvalue) => {
                keyvalue.write_to_with_space(writer, space)?;
            }
            Self::Constant(constant) => {
                constant.write_to_with_space(writer, space)?;
            }
            Self::MacroParam(param) => {
                param.write_to_with_space(writer, space)?;
            }
        }
        Ok(())
    }

    pub fn try_parse(fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let fill = match rc(content, move |c| CobMapKeyValue::try_parse(fill, c))? {
            (Some(kv), next_fill, remaining) => return Ok((Some(Self::KeyValue(kv)), next_fill, remaining)),
            (None, next_fill, _) => next_fill,
        };
        let fill = match rc(content, move |c| CobConstant::try_parse(fill, c))? {
            (Some(constant), next_fill, remaining) => {
                return Ok((Some(Self::Constant(constant)), next_fill, remaining))
            }
            (None, next_fill, _) => next_fill,
        };
        let fill = match rc(content, move |c| CobMacroParam::try_parse(fill, c))? {
            (Some(param), next_fill, remaining) => {
                if !param.is_catch_all() {
                    tracing::warn!("failed parsing map entry at {}; found macro param that isn't a 'catch all'",
                        get_location(content));
                    return Err(span_verify_error(content));
                }
                return Ok((Some(Self::MacroParam(param)), next_fill, remaining));
            }
            (None, next_fill, _) => next_fill,
        };

        Ok((None, fill, content))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        match (self, other) {
            (Self::KeyValue(keyvalue), Self::KeyValue(other_keyvalue)) => {
                keyvalue.recover_fill(other_keyvalue);
            }
            (Self::Constant(constant), Self::Constant(other_constant)) => {
                constant.recover_fill(other_constant);
            }
            (Self::MacroParam(param), Self::MacroParam(other_param)) => {
                param.recover_fill(other_param);
            }
            _ => (),
        }
    }

    pub fn resolve<'a>(
        &mut self,
        constants: &'a ConstantsBuffer,
    ) -> Result<Option<&'a [CobValueGroupEntry]>, String>
    {
        match self {
            Self::KeyValue(kv) => kv.resolve(constants)?,
            Self::Constant(constant) => {
                let Some(const_val) = constants.get(constant.path.as_str()) else {
                    return Err(format!("constant lookup failed for ${}", constant.path.as_str()));
                };
                match const_val {
                    CobConstantValue::Value(_) => {
                        return Err(
                            format!("constant ${} points to a value but is found in a map where only \
                            value groups of key-value pairs are allowed", constant.path.as_str()),
                        );
                    }
                    CobConstantValue::ValueGroup(group) => {
                        return Ok(Some(&group.entries));
                    }
                }
            }
            Self::MacroParam(param) => {
                // TODO: need to warn if encountered a param while not resolving a macro call
                return Err(format!("encountered macro parameter {param:?} in map"));
            }
        }
        Ok(None)
    }

    pub fn struct_field(key: &str, value: CobValue) -> Self
    {
        Self::KeyValue(CobMapKeyValue::struct_field(key, value))
    }

    pub fn map_entry(key: CobValue, value: CobValue) -> Self
    {
        Self::KeyValue(CobMapKeyValue::map_entry(key, value))
    }

    /// Returns `true` if the value is a key-value type.
    pub fn is_keyvalue(&self) -> bool
    {
        matches!(*self, Self::KeyValue(..))
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct CobMap
{
    /// Fill before opening `{`.
    pub start_fill: CobFill,
    pub entries: Vec<CobMapEntry>,
    /// Fill before ending `}`.
    pub end_fill: CobFill,
}

impl CobMap
{
    pub fn write_to(&self, writer: &mut impl RawSerializer) -> Result<(), std::io::Error>
    {
        self.write_to_with_space(writer, "")
    }

    pub fn write_to_with_space(&self, writer: &mut impl RawSerializer, space: &str) -> Result<(), std::io::Error>
    {
        self.start_fill.write_to_or_else(writer, space)?;
        writer.write_bytes("{".as_bytes())?;
        for (idx, entry) in self.entries.iter().enumerate() {
            if idx == 0 {
                entry.write_to(writer)?;
            } else {
                entry.write_to_with_space(writer, " ")?;
            }
        }
        self.end_fill.write_to(writer)?;
        writer.write_bytes("}".as_bytes())?;
        Ok(())
    }

    pub fn try_parse(start_fill: CobFill, content: Span) -> Result<(Option<Self>, CobFill, Span), SpanError>
    {
        let Ok((remaining, _)) = char::<_, ()>('{').parse(content) else { return Ok((None, start_fill, content)) };

        let (mut item_fill, mut remaining) = CobFill::parse(remaining);
        let mut entries = vec![];

        let end_fill = loop {
            let fill_len = item_fill.len();
            match rc(remaining, move |rm| CobMapEntry::try_parse(item_fill, rm))? {
                (Some(entry), next_fill, after_entry) => {
                    if entries.len() > 0 {
                        if fill_len == 0 {
                            tracing::warn!("failed parsing map at {}; entry #{} is not preceded by fill/whitespace",
                                get_location(content), entries.len() + 1);
                            return Err(span_verify_error(content));
                        }
                    }
                    entries.push(entry);
                    item_fill = next_fill;
                    remaining = after_entry;
                }
                (None, end_fill, after_end) => {
                    remaining = after_end;
                    break end_fill;
                }
            }
        };

        let (remaining, _) = char('}').parse(remaining)?;
        let (post_fill, remaining) = CobFill::parse(remaining);
        Ok((Some(Self { start_fill, entries, end_fill }), post_fill, remaining))
    }

    pub fn recover_fill(&mut self, other: &Self)
    {
        self.start_fill.recover(&other.start_fill);
        for (entry, other_entry) in self.entries.iter_mut().zip(other.entries.iter()) {
            entry.recover_fill(other_entry);
        }
        self.end_fill.recover(&other.end_fill);
    }

    pub fn resolve(&mut self, constants: &ConstantsBuffer) -> Result<(), String>
    {
        let mut idx = 0;
        while idx < self.entries.len() {
            // If resolving the entry returns a group of values, they need to be flattened into this map.
            let Some(group) = self.entries[idx].resolve(constants)? else {
                idx += 1;
                continue;
            };

            // Remove the old entry.
            let old = self.entries.remove(idx);

            // Flatten the group into the map.
            for val in group.iter() {
                match val {
                    CobValueGroupEntry::KeyValue(kv) => {
                        self.entries.insert(idx, CobMapEntry::KeyValue(kv.clone()));
                        idx += 1;
                    }
                    CobValueGroupEntry::Value(_) => {
                        let err_msg = match old {
                            CobMapEntry::Constant(constant) => {
                                format!("failed flattening constant ${}'s value group into \
                                a map, the group contains a plain value which is incompatible with maps",
                                constant.path.as_str())
                            }
                            _ => format!("failed flattening {{source unknown}} value group into \
                                a map, the group contains a plain value which is incompatible with maps"),
                        };
                        return Err(err_msg);
                    }
                }
            }
        }

        Ok(())
    }
}

impl From<Vec<CobMapEntry>> for CobMap
{
    fn from(entries: Vec<CobMapEntry>) -> Self
    {
        Self {
            start_fill: CobFill::default(),
            entries,
            end_fill: CobFill::default(),
        }
    }
}

/*
Parsing:
*/

//-------------------------------------------------------------------------------------------------------------------
