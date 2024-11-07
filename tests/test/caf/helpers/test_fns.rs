use std::fmt::Debug;

use bevy::prelude::*;
use bevy::reflect::serde::TypedReflectDeserializer;
use bevy_cobweb_ui::prelude::caf::*;
use bevy_cobweb_ui::prelude::*;
use serde::de::DeserializeSeed;
use serde::{Deserialize, Serialize};

use crate::caf::helpers::test_span;

//-------------------------------------------------------------------------------------------------------------------

/// Tests if a raw CAF loadable, raw CAF value, raw JSON, and rust struct are equivalent.
///
/// Only works for types without reflect-defaulted fields.
fn test_equivalence_impl<T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>>(
    w: &World,
    caf_raw: &str,
    caf_raw_val: &str,
    value: T,
    check_vals: bool,
)
{
    let type_registry = w.resource::<AppTypeRegistry>().read();
    let registration = type_registry.get(std::any::TypeId::of::<T>()).unwrap();

    // Caf raw to Caf loadable
    let loadable_parsed = match CafLoadable::try_parse(CafFill::default(), test_span(caf_raw)) {
        Ok((Some(loadable_parsed), _, _)) => loadable_parsed,
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
    let command_raw = format!("#commands\n{caf_raw}\n");
    let mut caf_parsed = match Caf::parse(test_span(command_raw.as_str())) {
        Ok(caf_parsed) => caf_parsed,
        Err(err) => panic!("{command_raw}, ERR={err:?}"),
    };
    let CafSection::Commands(commands) = &mut caf_parsed.sections[0] else { unreachable!() };
    let CafCommandEntry::Loadable(cmd_loadable) = &mut commands.entries[0] else { unreachable!() };
    cmd_loadable.fill = CafFill::default(); // Clear fill so equality test works.
    assert_eq!(*cmd_loadable, loadable_parsed);

    // Caf raw to Caf scene raw
    // TODO

    // Caf loadable to reflect
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&loadable_parsed).unwrap();
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_val = deserializer.deserialize(&cafvalue_parsed).unwrap();

    // Reflect to rust value
    let extracted_inst = T::from_reflect(reflected_inst.as_reflect()).unwrap();
    let extracted_val = T::from_reflect(reflected_val.as_reflect()).unwrap();
    if check_vals {
        assert_eq!(value, extracted_inst);
        assert_eq!(value, extracted_val);
    }

    // Rust value to caf loadable
    let mut loadable_from_rust = CafLoadable::extract(&value, &type_registry).unwrap();
    let mut cafvalue_from_rust = CafValue::extract(&value).unwrap();
    let mut loadable_from_rust_reflect = CafLoadable::extract_reflect(&value, &type_registry).unwrap();
    let mut cafvalue_from_rust_reflect = CafValue::extract_reflect(&value, &type_registry).unwrap();
    loadable_from_rust.recover_fill(&loadable_parsed);
    loadable_from_rust_reflect.recover_fill(&loadable_parsed);
    cafvalue_from_rust.recover_fill(&cafvalue_parsed);
    cafvalue_from_rust_reflect.recover_fill(&cafvalue_parsed);
    assert_eq!(loadable_from_rust, loadable_parsed);
    assert_eq!(loadable_from_rust_reflect, loadable_parsed);
    assert_eq!(cafvalue_from_rust, cafvalue_parsed);
    assert_eq!(cafvalue_from_rust_reflect, cafvalue_parsed);

    // Rust value from caf loadable parsed (direct)
    let direct_value = T::deserialize(&loadable_parsed).unwrap();
    let direct_value_from_value = T::deserialize(&cafvalue_parsed).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Rust value from caf loadable from rust (direct)
    let direct_value = T::deserialize(&loadable_from_rust).unwrap();
    let direct_value_from_value = T::deserialize(&cafvalue_from_rust).unwrap();
    if check_vals {
        assert_eq!(value, direct_value);
        assert_eq!(value, direct_value_from_value);
    }

    // Caf loadable-from-raw to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    loadable_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw, reconstructed_raw);

    // Caf value-from-raw to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cafvalue_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(caf_raw_val, reconstructed_raw_val);

    // Caf loadable-from-rust to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    loadable_from_rust.write_to(&mut serializer).unwrap();
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
pub fn test_equivalence<T>(w: &World, caf_raw: &str, caf_raw_val: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_impl(w, caf_raw, caf_raw_val, value, true);
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`test_equivalence_impl`].
pub fn test_equivalence_skip_value_eq<T>(w: &World, caf_raw: &str, caf_raw_val: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_impl(w, caf_raw, caf_raw_val, value, false);
}

//-------------------------------------------------------------------------------------------------------------------

/// Tests if a raw CAF loadable, raw CAF value, raw JSON, and rust struct are equivalent.
///
/// Expects the conversion `raw -> Caf (-> value -> Caf) -> raw` to be lossy in the sense that original syntax
/// won't be preserved.
fn test_equivalence_lossy_impl<T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>>(
    w: &World,
    caf_raw: &str,
    caf_raw_reserialized: &str,
    value: T,
    allow_t_deserialize: bool,
)
{
    let type_registry = w.resource::<AppTypeRegistry>().read();
    let registration = type_registry.get(std::any::TypeId::of::<T>()).unwrap();

    // Caf raw to Caf loadable
    let loadable_parsed = match CafLoadable::try_parse(CafFill::default(), test_span(caf_raw)) {
        Ok((Some(loadable_parsed), _, _)) => loadable_parsed,
        Err(err) => panic!("{caf_raw}, ERR={err:?}"),
        _ => panic!("{caf_raw}, TRY FAILED"),
    };

    // Caf loadable to reflect
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&loadable_parsed).unwrap();

    // Reflect to rust value
    let extracted_inst = T::from_reflect(reflected_inst.as_reflect()).unwrap();
    assert_eq!(value, extracted_inst);

    // Rust value to caf loadable
    let mut loadable_from_rust = CafLoadable::extract(&value, &type_registry).unwrap();
    let mut loadable_from_rust_reflect = CafLoadable::extract_reflect(&value, &type_registry).unwrap();
    loadable_from_rust.recover_fill(&loadable_parsed);
    loadable_from_rust_reflect.recover_fill(&loadable_parsed);
    assert_eq!(loadable_from_rust, loadable_from_rust_reflect);
    //assert_eq!(loadable_from_rust, loadable_parsed); // possibly not true e.g. in the case of builtin
    // conversions

    // Rust value from caf loadable parsed (direct)
    if allow_t_deserialize {
        let direct_value = T::deserialize(&loadable_parsed).unwrap();
        assert_eq!(value, direct_value);
    }

    // Rust value from caf loadable from rust (direct)
    let direct_value = T::deserialize(&loadable_from_rust).unwrap();
    assert_eq!(value, direct_value);

    // Rust value from caf loadable from rust (reflect)
    // - Need to make sure the 'canonical' representation can be deserialized with reflection.
    let deserializer = TypedReflectDeserializer::new(registration, &type_registry);
    let reflected_inst = deserializer.deserialize(&loadable_from_rust).unwrap();
    let extracted_inst = T::from_reflect(reflected_inst.as_reflect()).unwrap();
    assert_eq!(value, extracted_inst);

    // Caf loadable-from-raw to caf raw
    // NOTE: we don't test this since there are 'intermediate' lossy cases where the lossiness occurs on
    // raw -> Caf -> raw instead of raw -> Caf -> rust -> Caf -> raw

    // Caf loadable-from-rust to caf raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    loadable_from_rust.write_to(&mut serializer).unwrap();
    let reconstructed_raw = String::from_utf8(buff).unwrap();
    assert_eq!(reconstructed_raw, caf_raw_reserialized);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn test_equivalence_lossy<T>(w: &World, caf_raw: &str, caf_raw_reserialized: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_lossy_impl(w, caf_raw, caf_raw_reserialized, value, true);
}

//-------------------------------------------------------------------------------------------------------------------

/// The original caf value will only be deserialized via reflection.
///
/// Useful when `T` has reflect-defaulted fields.
pub fn test_equivalence_lossy_reflection<T>(w: &World, caf_raw: &str, caf_raw_reserialized: &str, value: T)
where
    T: Loadable + Debug + Serialize + for<'de> Deserialize<'de>,
{
    test_equivalence_lossy_impl(w, caf_raw, caf_raw_reserialized, value, false);
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

pub fn test_string_conversion(raw: &str, converted: &str, reserialized: &str, num_segments: usize)
{
    // Wrap raw in quotes
    let raw = format!("\"{}\"", raw);

    // Raw to value
    let cafvalue_parsed = match CafValue::try_parse(CafFill::default(), test_span(raw.as_str())) {
        Ok((Some(cafvalue_parsed), _, _)) => cafvalue_parsed,
        Err(err) => panic!("{}, ERR={err:?}", raw.as_str()),
        _ => panic!("{}, TRY FAILED", raw.as_str()),
    };
    let CafValue::String(string) = &cafvalue_parsed else { panic!("{cafvalue_parsed:?}") };
    assert_eq!(string.segments.len(), num_segments);
    assert_eq!(string.as_str(), converted);

    // Value to raw
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cafvalue_parsed.write_to(&mut serializer).unwrap();
    let reconstructed_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(raw, reconstructed_raw_val);

    // Converted to value
    let cafvalue_from_str = CafValue::extract(converted).unwrap();
    let CafValue::String(string) = &cafvalue_from_str else { panic!("{cafvalue_from_str:?}") };
    assert_eq!(string.segments.len(), 1);
    assert_eq!(string.as_str(), converted);
    //don't test equality of the CafValue, since extraction-from-converted is lossy

    // Converted value to raw
    // Note: 'reserialized' contains escape sequences, whereas 'converted' contains literal chars
    let mut buff = Vec::<u8>::default();
    let mut serializer = DefaultRawSerializer::new(&mut buff);
    cafvalue_from_str.write_to(&mut serializer).unwrap();
    let converted_raw_val = String::from_utf8(buff).unwrap();
    assert_eq!(format!("\"{reserialized}\""), converted_raw_val);
}

//-------------------------------------------------------------------------------------------------------------------
