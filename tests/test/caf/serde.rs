//! Serializing and deserializing instructions and values.

// Need to distinguish between CAF input and expected CAF output (after JSON round trip),
// since multi-line string formatting is lossy when entering JSON/Rust.

// Value round trip: rust type -> json value -> Caf -> json value -> reflect -> rust type
//   - Replace with CAF round trip once CAF parsing is ready. Note that Caf -> CAF -> Caf is potentially mutating
//   if whitespace is inserted during serialization.
// CAF round trip: CAF -> Caf -> json value -> reflect rust type (check against expected) -> json value
// -> Caf (+ recover fill) -> CAF
//   - Need separate sequence for testing #[reflect(default)] fields, since defaulted 'dont show' fields are not
//   known in rust.

use super::helpers::*;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn unit_struct()
{
    let app = prepare_test_app();
    test_instruction_equivalence(app.world(), "UnitStruct", "{}", UnitStruct);
}

//-------------------------------------------------------------------------------------------------------------------
