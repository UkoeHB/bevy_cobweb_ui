use std::io::Cursor;

use bevy_cobweb_ui::prelude::*;

//-------------------------------------------------------------------------------------------------------------------
/*
pub(crate) fn caf_round_trip(raw: impl AsRef<str>)
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
pub(crate) fn caf_parse_skip_space(raw: impl AsRef<str>, value: Caf)
{
    let raw = raw.as_ref().as_bytes();


    // Caf raw to Caf value
    //TODO

    // Compare to expected Caf value
    // TODO
}
*/
//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn caf_parse_test_result(raw: impl AsRef<str>, value: Caf) -> bool
{
    let raw = raw.as_ref().as_bytes();

    // Caf raw to Caf value
    //TODO

    // Compare to expected Caf value
    // TODO

    // Caf value to caf raw
    let mut bytes = Vec::<u8>::default();
    let mut cursor = Cursor::new(&mut bytes);
    value.write_to(&mut cursor).unwrap();

    // Compare to raw.
    (raw == &bytes[..]
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn caf_parse_test(raw: impl AsRef<str>, value: Caf)
{
    assert!(caf_parse_test_result(raw, value));
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn caf_parse_test_fail(raw: impl AsRef<str>, value: Caf)
{
    assert!(!caf_parse_test_result(raw, value));
}

//-------------------------------------------------------------------------------------------------------------------
