use std::io::{Cursor, Write};

use crate::prelude::CobNumberValue;

//-------------------------------------------------------------------------------------------------------------------

/// Serializer for converting [`CobLoadable`](crate::prelude::CobLoadable)/
/// [`CobValue`](crate::prelude::CobValue) to raw bytes.
pub trait RawSerializer: std::io::Write
{
    fn write_u128(&mut self, val: u128) -> Result<(), std::io::Error>;
    fn write_i128(&mut self, val: i128) -> Result<(), std::io::Error>;
    /// Only finite numbers are passed in here.
    fn write_f64(&mut self, val: f64) -> Result<(), std::io::Error>;
    /// Only finite numbers are passed in here.
    fn write_f32(&mut self, val: f32) -> Result<(), std::io::Error>;
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error>;
}

//-------------------------------------------------------------------------------------------------------------------

/// Implementation of [`RawSerializer`] with default options.
pub struct DefaultRawSerializer<'a>
{
    cursor: Cursor<&'a mut Vec<u8>>,
    simplify_fp: bool,
}

impl<'a> DefaultRawSerializer<'a>
{
    /// Makes a new raw serializer with default options.
    pub fn new(bytes: &'a mut Vec<u8>) -> Self
    {
        Self { cursor: Cursor::new(bytes), simplify_fp: true }
    }

    /// Controls whether floating point numbers can be coerced to integers if there is no precision loss.
    ///
    /// On by default.
    pub fn simplify_fp(mut self, simplify: bool) -> Self
    {
        self.simplify_fp = simplify;
        self
    }
}

impl<'a> RawSerializer for DefaultRawSerializer<'a>
{
    fn write_u128(&mut self, val: u128) -> Result<(), std::io::Error>
    {
        write!(self, "{:?}", val)
    }
    fn write_i128(&mut self, val: i128) -> Result<(), std::io::Error>
    {
        write!(self, "{:?}", val)
    }
    fn write_f64(&mut self, val: f64) -> Result<(), std::io::Error>
    {
        debug_assert!(val.is_finite());

        // Try to write as integer.
        if self.simplify_fp {
            if val.is_finite() {
                let num = CobNumberValue::Float64(val);
                // We only convert inside +/- 1e16 because the rust formatter only uses scientific notation for
                // floats with magnitude >= 1e16 (it doesn't use scientific notation for ints).
                if val.is_sign_positive() && val < 1e16 {
                    if let Some(converted) = num.as_u128() {
                        return self.write_u128(converted);
                    }
                } else if val.is_sign_negative() && val > -1e16 {
                    if let Some(converted) = num.as_i128() {
                        return self.write_i128(converted);
                    }
                }
            }
        }

        write!(self, "{:?}", val)
    }
    fn write_f32(&mut self, val: f32) -> Result<(), std::io::Error>
    {
        debug_assert!(val.is_finite());

        // Try to write as integer.
        if self.simplify_fp {
            let num = CobNumberValue::Float32(val);
            // We only convert inside +/- 1e16 because the rust formatter only uses scientific notation for
            // floats with magnitude >= 1e16 (it doesn't use scientific notation for ints).
            if val.is_sign_positive() && val < 1e16 {
                if let Some(converted) = num.as_u128() {
                    return self.write_u128(converted);
                }
            } else if val.is_sign_negative() && val > -1e16 {
                if let Some(converted) = num.as_i128() {
                    return self.write_i128(converted);
                }
            }
        }

        write!(self, "{:?}", val)
    }
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error>
    {
        self.write(bytes)?;
        Ok(())
    }
}

impl<'a> std::io::Write for DefaultRawSerializer<'a>
{
    fn write(&mut self, bytes: &[u8]) -> Result<usize, std::io::Error>
    {
        self.cursor.write(bytes)
    }

    fn flush(&mut self) -> Result<(), std::io::Error>
    {
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------
