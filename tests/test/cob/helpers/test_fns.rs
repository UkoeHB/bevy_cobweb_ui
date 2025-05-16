use std::fmt::Debug;

use bevy::prelude::*;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy_cobweb_ui::cob::*;
use bevy_cobweb_ui::prelude::*;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

use crate::cob::helpers::test_span;

//-------------------------------------------------------------------------------------------------------------------

/// Tests if a raw COB loadable, raw COB value, raw JSON, and rust struct are equivalent.
///
/// Only works for types without reflect-defaulted fields.
fn test_equivalence_impl<T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>>(
    w: &World,
    cob_raw: &str,
    cob_raw_val: &str,
    value: T,
    check_vals: bool,
)
{
    let type_registry = w.resource::<AppTypeRegistry>().read();
    let registration = type_registry.get(std::any::TypeId::of::<T>()).unwrap();

    // Cob raw to Cob loadable
    let loadable_parsed = match CobLoadable::try_parse(CobFill::default(), test_span(cob_raw)) {
        Ok((Some(loadable_parsed), _, _)) => loadable_parsed,
        Err(err) => panic!("{cob_raw}, ERR={err:?}"),
        _ => panic!("{cob_raw}, TRY FAILED"),
    };

    // Cob raw val to Cob value
    let cobvalue_parsed = match CobValue::try_parse(CobFill::default(), test_span(cob_raw_val)) {
        Ok((Some(cobvalue_parsed), _, _)) => cobvalue_parsed,
        Err(err) => panic!("{cob_raw}, ERR={err:?}"),
        _ => panic!("{cob_raw}, TRY FAILED"),
    };

    // Cob raw to Cob command raw
    let command_raw = format!("#commands\n{cob_raw}\n");
    let mut cob_parsed = match Cob::parse(test_span(command_raw.as_str())) {
        Ok(cob_parsed) => cob_parsed,
        Err(err) => panic!("{command_raw}, ERR={err:?}"),
    };
    let CobSection::Commands(commands) = &mut cob_parsed.sections[0] else { unreachable!() };
    let CobCommandEntry(cmd_loadable) = &mut commands.entries[0];
    cmd_loadable.fill = CobFill::default(); // Clear fill so equality test works.
    assert_eq!(*cmd_loadable, loadable_parsed);

    // Cob raw to Cob scene raw
    // TODO

    // Cob loadable to reflect
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&loadable_parsed).unwrap();
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_val = deserializer.deserialize(&cobvalue_parsed).unwrap();

    // Reflect to rust value
    let extracted_inst = T::from_reflect(reflected_inst.as_partial_reflect()).unwrap();
    let extracted_val = T::from_reflect(reflected_val.as_partial_reflect()).unwrap();
    if check_vals {
        assert_eq!(value, extracted_inst);
        assert_eq!(value, extracted_val);
    }

    // Rust value to cob loadable
    let mut loadable_from_rust = CobLoadable::extract_with_registry(&value, &type_registry).unwrap();
    let mut cobvalue_from_rust = CobValue::extract(&value).unwrap();
    let mut loadable_from_rust_reflect = CobLoadable::extract_reflect(&value, &type_registry).unwrap();
    let mut cobvalue_from_rust_reflect = CobValue::extract_reflect(&value, &type_registry).unwrap();
    loadable_from_rust.recover_fill(&loadable_parsed);
    loadable_from_rust_reflect.recover_fill(&loadable_parsed);
    cobvalue_from_rust.recover_fill(&cobvalue_parsed);
    cobvalue_from_rust_reflect.recover_fill(&cobvalue_parsed);
    assert_eq!(loadable_from_rust, loadable_parsed);
    assert_eq!(loadable_from_rust_reflect, loadable_parsed);
    assert_eq!(cobvalue_from_rust, cobvalue_parsed);
    assert_eq!(cobvalue_from_rust_reflect, cobvalue_parsed);

    // Rust value from cob loadable parsed (direct)
    let direct_value = T::deserialize(&loadable_parsed).unwrap();
    let direct_value_from_value = T::deserialize(&cobvalue_parsed).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Rust value from cob loadable from rust (direct)
    let direct_value = T::deserialize(&loadable_from_rust).unwrap();
    let direct_value_from_value = T::deserialize(&cobvalue_from_rust).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Cob loadable-from-raw to cob raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    loadable_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(cob_raw, reconstructed_raw);

    // Cob value-from-raw to cob raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cobvalue_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(cob_raw_val, reconstructed_raw_val);

    // Cob loadable-from-rust to cob raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    loadable_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(cob_raw, reconstructed_raw);

    // Cob value-from-rust to cob raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cobvalue_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(cob_raw_val, reconstructed_raw_val);
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`test_equivalence_impl`].
pub fn test_equivalence<T>(w: &World, cob_raw: &str, cob_raw_val: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_impl(w, cob_raw, cob_raw_val, value, true);
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`test_equivalence_impl`].
pub fn test_equivalence_skip_value_eq<T>(w: &World, cob_raw: &str, cob_raw_val: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_impl(w, cob_raw, cob_raw_val, value, false);
}

//-------------------------------------------------------------------------------------------------------------------

/// Tests if a raw COB loadable, raw COB value, raw JSON, and rust struct are equivalent.
///
/// Expects the conversion `raw -> Cob (-> value -> Cob) -> raw` to be lossy in the sense that original syntax
/// won't be preserved.
fn test_equivalence_lossy_impl<T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>>(
    w: &World,
    cob_raw: &str,
    cob_raw_reserialized: &str,
    value: T,
    allow_t_deserialize: bool,
)
{
    let type_registry = w.resource::<AppTypeRegistry>().read();
    let registration = type_registry.get(std::any::TypeId::of::<T>()).unwrap();

    // Cob raw to Cob loadable
    let loadable_parsed = match CobLoadable::try_parse(CobFill::default(), test_span(cob_raw)) {
        Ok((Some(loadable_parsed), _, _)) => loadable_parsed,
        Err(err) => panic!("{cob_raw}, ERR={err:?}"),
        _ => panic!("{cob_raw}, TRY FAILED"),
    };

    // Cob loadable to reflect
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&loadable_parsed).unwrap();

    // Reflect to rust value
    let extracted_inst = T::from_reflect(reflected_inst.as_partial_reflect()).unwrap();
    assert_eq!(value, extracted_inst);

    // Rust value to cob loadable
    let mut loadable_from_rust = CobLoadable::extract_with_registry(&value, &type_registry).unwrap();
    let mut loadable_from_rust_reflect = CobLoadable::extract_reflect(&value, &type_registry).unwrap();
    loadable_from_rust.recover_fill(&loadable_parsed);
    loadable_from_rust_reflect.recover_fill(&loadable_parsed);
    assert_eq!(loadable_from_rust, loadable_from_rust_reflect);
    //assert_eq!(loadable_from_rust, loadable_parsed); // possibly not true e.g. in the case of builtin
    // conversions

    // Rust value from cob loadable parsed (direct)
    if allow_t_deserialize {
        let direct_value = T::deserialize(&loadable_parsed).unwrap();
        assert_eq!(value, direct_value);
    }

    // Rust value from cob loadable from rust (direct)
    let direct_value = T::deserialize(&loadable_from_rust).unwrap();
    assert_eq!(value, direct_value);

    // Rust value from cob loadable from rust (reflect)
    // - Need to make sure the 'canonical' representation can be deserialized with reflection.
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&loadable_from_rust).unwrap();
    let extracted_inst = T::from_reflect(reflected_inst.as_partial_reflect()).unwrap();
    assert_eq!(value, extracted_inst);

    // Cob loadable-from-raw to cob raw
    // NOTE: we don't test this since there are 'intermediate' lossy cases where the lossiness occurs on
    // raw -> Cob -> raw instead of raw -> Cob -> rust -> Cob -> raw

    // Cob loadable-from-rust to cob raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    loadable_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(reconstructed_raw, cob_raw_reserialized);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn test_equivalence_lossy<T>(w: &World, cob_raw: &str, cob_raw_reserialized: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_lossy_impl(w, cob_raw, cob_raw_reserialized, value, true);
}

//-------------------------------------------------------------------------------------------------------------------

/// The original cob value will only be deserialized via reflection.
///
/// Useful when `T` has reflect-defaulted fields.
pub fn test_equivalence_lossy_reflection<T>(w: &World, cob_raw: &str, cob_raw_reserialized: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_lossy_impl(w, cob_raw, cob_raw_reserialized, value, false);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn test_cob(raw: &[u8]) -> Cob
{
    // Parse
    let string = String::from_utf8_lossy(raw);
    let parsed = Cob::parse(test_span(&string)).unwrap();

    // Write back
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    parsed.write_to(&mut serializer).unwrap();
    assert_eq!(String::from_utf8_lossy(&buff), string);

    parsed
}

//-------------------------------------------------------------------------------------------------------------------

/// Expects parsing a COB byte sequence to fail, with `remaining` bytes unparsed.
pub fn test_cob_fail(raw: &[u8], remaining: &[u8])
{
    // Parse
    let string = String::from_utf8_lossy(raw);
    let Err(error) = Cob::parse(test_span(&string)) else { unreachable!() };
    let span = unwrap_error_content(error);
    let remaining = String::from_utf8_lossy(remaining);
    assert_eq!(remaining, *span.fragment());
}

//-------------------------------------------------------------------------------------------------------------------

pub fn test_string_conversion(raw: &str, converted: &str, reserialized: &str, num_segments: usize)
{
    // Wrap raw in quotes
    let raw = format!("\"{}\"", raw);

    // Raw to value
    let cobvalue_parsed = match CobValue::try_parse(CobFill::default(), test_span(raw.as_str())) {
        Ok((Some(cobvalue_parsed), _, _)) => cobvalue_parsed,
        Err(err) => panic!("{}, ERR={err:?}", raw.as_str()),
        _ => panic!("{}, TRY FAILED", raw.as_str()),
    };
    let CobValue::String(string) = &cobvalue_parsed else { panic!("{cobvalue_parsed:?}") };
    assert_eq!(string.segments.len(), num_segments);
    assert_eq!(string.as_str(), converted);

    // Value to raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cobvalue_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(raw, reconstructed_raw_val);

    // Converted to value
    let cobvalue_from_str = CobValue::extract(converted).unwrap();
    let CobValue::String(string) = &cobvalue_from_str else { panic!("{cobvalue_from_str:?}") };
    assert_eq!(string.segments.len(), 1);
    assert_eq!(string.as_str(), converted);
    //don't test equality of the CobValue, since extraction-from-converted is lossy

    // Converted value to raw
    // Note: 'reserialized' contains escape sequences, whereas 'converted' contains literal chars
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cobvalue_from_str.write_to(&mut serializer).unwrap();
    let converted_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(format!("\"{reserialized}\""), converted_raw_val);
}

//-------------------------------------------------------------------------------------------------------------------
