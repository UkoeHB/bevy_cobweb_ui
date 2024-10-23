use std::fmt::Debug;

use bevy::prelude::*;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy_cobweb_ui::prelude::*;
use serde::de::DeserializeSeed;

//-------------------------------------------------------------------------------------------------------------------
/*
pub fn caf_round_trip(raw: impl AsRef<str>)
{
    let raw = raw.as_ref().as_bytes();

    // Caf raw to Caf value
    //TODO

    // Caf value to caf raw
    let mut bytes = Vec::<u8>::default();
    let mut cursor = Cursor::new(&mut bytes);
    value.write_to(&mut cursor).unwrap();

    // Compare to raw.
    assert_eq!(raw, &bytes[..]);

}
*/
//-------------------------------------------------------------------------------------------------------------------
/*
pub fn caf_parse_skip_space(raw: impl AsRef<str>, value: Caf)
{
    let raw = raw.as_ref().as_bytes();


    // Caf raw to Caf value
    //TODO

    // Compare to expected Caf value
    // TODO
}
*/
//-------------------------------------------------------------------------------------------------------------------

pub fn caf_parse_test_result(raw: impl AsRef<str>, value: Caf) -> bool
{
    let raw = raw.as_ref().as_bytes();

    // Caf raw to Caf value
    //TODO

    // Compare to expected Caf value
    // TODO

    // Caf value to caf raw
    let mut bytes = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut bytes);
    value.write_to(&mut serializer).unwrap();

    // Compare to raw.
    raw == &bytes[..]
}

//-------------------------------------------------------------------------------------------------------------------

pub fn caf_parse_test(raw: impl AsRef<str>, value: Caf)
{
    assert!(caf_parse_test_result(raw, value));
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

/// Tests if a raw CAF instruction, raw CAF value, raw JSON, and rust struct are equivalent.
///
/// Only works for types without reflect-defaulted fields.
pub fn test_equivalence_impl<T: Loadable + Debug>(
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
    // TODO

    // Rust value to caf instruction
    // TODO: remove once raw -> instruction is available
    let instruction_from_rust = CafInstruction::extract(&value, &type_registry).unwrap();
    let cafvalue_from_rust = CafValue::extract(&value).unwrap();
    let direct_value = T::deserialize(&instruction_from_rust).unwrap();
    let direct_value_from_value = T::deserialize(&cafvalue_from_rust).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Caf instruction to reflect
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&instruction_from_rust).unwrap();
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_val = deserializer.deserialize(&cafvalue_from_rust).unwrap();

    // Reflect to rust value
    let extracted_inst = T::from_reflect(reflected_inst.as_reflect()).unwrap();
    let extracted_val = T::from_reflect(reflected_val.as_reflect()).unwrap();
    if check_vals {
        assert_eq!(value, extracted_inst);
        assert_eq!(value, extracted_val);
    }

    // Rust value to caf instruction
    let instruction_from_rust = CafInstruction::extract(&value, &type_registry).unwrap();

    // Caf instruction to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    instruction_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw, reconstructed_raw);

    // Caf value to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cafvalue_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw_val, reconstructed_raw_val);
}

//-------------------------------------------------------------------------------------------------------------------
