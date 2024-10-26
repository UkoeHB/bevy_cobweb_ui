use std::fmt::Debug;

use bevy::prelude::*;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy_cobweb_ui::prelude::caf::*;
use bevy_cobweb_ui::prelude::*;
use serde::de::DeserializeSeed;

use crate::caf::helpers::test_span;

//-------------------------------------------------------------------------------------------------------------------

/// Tests if a raw CAF instruction, raw CAF value, raw JSON, and rust struct are equivalent.
///
/// Only works for types without reflect-defaulted fields.
fn test_equivalence_impl<T: Loadable + Debug>(
    w: &World,
    caf_raw: &str,
    caf_raw_val: &str,
    value: T,
    check_vals: bool,
)
{
    let type_registry = w.resource::<AppTypeRegistry>().read();
    let registration = type_registry.get(std::any::TypeId::of::<T>()).unwrap();

    // Caf raw to Caf instruction
    let instruction_parsed = match CafInstruction::try_parse(CafFill::default(), test_span(caf_raw)) {
        Ok((Some(instruction_parsed), _, _)) => instruction_parsed,
        Err(err) => panic!("{caf_raw}, ERR={err:?}"),
        _ => panic!("{caf_raw}, TRY FAILED"),
    };

    // Caf raw val to Caf value
    let cafvalue_parsed = match CafValue::try_parse(CafFill::default(), test_span(caf_raw_val)) {
        Ok((Some(cafvalue_parsed), _, _)) => cafvalue_parsed,
        Err(err) => panic!("{caf_raw}, ERR={err:?}"),
        _ => panic!("{caf_raw}, TRY FAILED"),
    };

    // Caf raw to Caf command raw
    // TODO

    // Caf raw to Caf scene raw
    // TODO

    // Caf instruction to reflect
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&instruction_parsed).unwrap();
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_val = deserializer.deserialize(&cafvalue_parsed).unwrap();

    // Reflect to rust value
    let extracted_inst = T::from_reflect(reflected_inst.as_reflect()).unwrap();
    let extracted_val = T::from_reflect(reflected_val.as_reflect()).unwrap();
    if check_vals {
        assert_eq!(value, extracted_inst);
        assert_eq!(value, extracted_val);
    }

    // Rust value to caf instruction
    let mut instruction_from_rust = CafInstruction::extract(&value, &type_registry).unwrap();
    let mut cafvalue_from_rust = CafValue::extract(&value).unwrap();
    instruction_from_rust.recover_fill(&instruction_parsed);
    cafvalue_from_rust.recover_fill(&cafvalue_parsed);
    assert_eq!(instruction_from_rust, instruction_parsed);
    assert_eq!(cafvalue_from_rust, cafvalue_parsed);

    // Rust value from caf instruction parsed (direct)
    let direct_value = T::deserialize(&instruction_parsed).unwrap();
    let direct_value_from_value = T::deserialize(&cafvalue_parsed).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Rust value from caf instruction from rust (direct)
    let direct_value = T::deserialize(&instruction_from_rust).unwrap();
    let direct_value_from_value = T::deserialize(&cafvalue_from_rust).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Caf instruction-from-raw to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    instruction_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw, reconstructed_raw);

    // Caf value-from-raw to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cafvalue_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw_val, reconstructed_raw_val);

    // Caf instruction-from-rust to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    instruction_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw, reconstructed_raw);

    // Caf value-from-rust to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cafvalue_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw_val, reconstructed_raw_val);
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`test_equivalence_impl`].
pub fn test_equivalence<T: Loadable + Debug>(w: &World, caf_raw: &str, caf_raw_val: &str, value: T)
{
    test_equivalence_impl(w, caf_raw, caf_raw_val, value, true);
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`test_equivalence_impl`].
pub fn test_equivalence_skip_value_eq<T: Loadable + Debug>(w: &World, caf_raw: &str, caf_raw_val: &str, value: T)
{
    test_equivalence_impl(w, caf_raw, caf_raw_val, value, false);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn test_caf(raw: &[u8]) -> Caf
{
    // Parse
    let string = String::from_utf8_lossy(raw);
    let parsed = Caf::parse(test_span(&string)).unwrap();

    // Write back
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    parsed.write_to(&mut serializer).unwrap();
    assert_eq!(String::from_utf8_lossy(&buff), string);

    parsed
}

//-------------------------------------------------------------------------------------------------------------------

/// Expects parsing a CAF byte sequence to fail, with `remaining` bytes unparsed.
pub fn test_caf_fail(raw: &[u8], remaining: &[u8])
{
    // Parse
    let string = String::from_utf8_lossy(raw);
    let Err(error) = Caf::parse(test_span(&string)) else { unreachable!() };
    let span = unwrap_error_content(error);
    let remaining = String::from_utf8_lossy(remaining);
    assert_eq!(remaining, *span.fragment());
}

//-------------------------------------------------------------------------------------------------------------------
