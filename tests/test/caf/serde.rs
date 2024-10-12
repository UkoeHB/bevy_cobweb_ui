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

use std::collections::HashMap;
use std::marker::PhantomData;

use super::helpers::*;

// TODO: test newtype and newtype variant of tuple
// TODO: test newtype of vec as inner value
// TODO: test built-in values
// TODO: test lossy conversions (scientific notation, multiline strings, ??)

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn unit_struct()
{
    let app = prepare_test_app();
    test_instruction_equivalence(app.world(), "UnitStruct", "{}", UnitStruct);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn plain_struct()
{
    let app = prepare_test_app();
    test_instruction_equivalence(
        app.world(),
        "PlainStruct{boolean:false}",
        r#"{"boolean":false}"#,
        PlainStruct { boolean: false },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn enum_struct()
{
    let app = prepare_test_app();
    test_instruction_equivalence(app.world(), "EnumStruct::A", r#""A""#, EnumStruct::A);
    test_instruction_equivalence(
        app.world(),
        "EnumStruct::B(())",
        r#"{"B": []}"#,
        EnumStruct::B(UnitStruct),
    );
    test_instruction_equivalence(
        app.world(),
        "EnumStruct::C{boolean:true s_plain:{boolean:true}}",
        r#"{"C":{"boolean":true,"s_plain":{"boolean":true}}}"#,
        EnumStruct::C { boolean: true, s_plain: PlainStruct { boolean: true } },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn aggregate_struct()
{
    let app = prepare_test_app();
    let mut map = HashMap::default();
    map.insert(10u32, 10u32);
    // TODO: can't test maps with multiple entries since HashMap ordering is not specified.
    //map.insert(20u32, 20u32);
    test_instruction_equivalence(
        app.world(),
        r#"AggregateStruct{uint:1 float:1.0 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10} s_struct:() s_enum:B(()) s_plain:{boolean:true}}"#,
        r#"{"uint":1,"float":1.0,"boolean":true,"string":"hi","vec":[{"boolean":true},{"boolean":false}],"map":{"10":10},"s_struct":[],"s_enum":{"B":[]},"s_plain":{"boolean":true}}"#,
        AggregateStruct {
            uint: 1,
            float: 1.0,
            boolean: true,
            string: String::from("hi"),
            vec: vec![PlainStruct{boolean: true}, PlainStruct{boolean: false}],
            map,
            s_struct: UnitStruct,
            s_enum: EnumStruct::B(UnitStruct),
            s_plain: PlainStruct { boolean: true },
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn wrap_array()
{
    let app = prepare_test_app();
    test_instruction_equivalence(app.world(), "WrapArray[]", "[[]]", WrapArray(vec![]));
    test_instruction_equivalence(app.world(), "WrapArray[()]", "[[[]]]", WrapArray(vec![UnitStruct]));
    test_instruction_equivalence(
        app.world(),
        "WrapArray[() ()]",
        "[[[],[]]]",
        WrapArray(vec![UnitStruct, UnitStruct]),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn tuple_struct()
{
    let app = prepare_test_app();
    test_instruction_equivalence(
        app.world(),
        "TupleStruct(() {boolean:true} true)",
        r#"[[],{"boolean":true},true]"#,
        TupleStruct(UnitStruct, PlainStruct { boolean: true }, true),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn single_generic()
{
    let app = prepare_test_app();
    test_instruction_equivalence(app.world(), "SingleGeneric<u32>", "{}", SingleGeneric::<u32>::default());
    test_instruction_equivalence(
        app.world(),
        "SingleGeneric<(u32, u32)>",
        "{}",
        SingleGeneric::<(u32, u32)>::default(),
    );
    test_instruction_equivalence(
        app.world(),
        "SingleGeneric<UnitStruct>",
        "{}",
        SingleGeneric::<UnitStruct>::default(),
    );
    test_instruction_equivalence(
        app.world(),
        "SingleGeneric<SingleGeneric<u32>>",
        "{}",
        SingleGeneric::<SingleGeneric<u32>>::default(),
    );
    test_instruction_equivalence(
        app.world(),
        "SingleGeneric<MultiGeneric<u32, u32, u32>>",
        "{}",
        SingleGeneric::<MultiGeneric<u32, u32, u32>>::default(),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn single_generic_tuple()
{
    let app = prepare_test_app();
    test_instruction_equivalence(
        app.world(),
        "SingleGenericTuple<u32>(1)",
        "[1]",
        SingleGenericTuple::<u32>(1),
    );
    test_instruction_equivalence(
        app.world(),
        "SingleGenericTuple<UnitStruct>(())",
        "[[]]",
        SingleGenericTuple::<UnitStruct>(UnitStruct),
    );
    test_instruction_equivalence(
        app.world(),
        "SingleGenericTuple<SingleGeneric<u32>>({})",
        "[{}]",
        SingleGenericTuple::<SingleGeneric<u32>>(SingleGeneric::default()),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn multi_generic()
{
    let app = prepare_test_app();
    test_instruction_equivalence(
        app.world(),
        "MultiGeneric<u32, u32, u32>",
        "{}",
        MultiGeneric::<u32, u32, u32>::default(),
    );
    test_instruction_equivalence(
        app.world(),
        "MultiGeneric<u32, u32, UnitStruct>",
        "{}",
        MultiGeneric::<u32, u32, UnitStruct>::default(),
    );
    test_instruction_equivalence(
        app.world(),
        "MultiGeneric<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>",
        "{}",
        MultiGeneric::<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>::default(),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn enum_generic()
{
    let app = prepare_test_app();
    test_instruction_equivalence(
        app.world(),
        "EnumGeneric<bool>::A{uint:1}",
        r#"{"A":{"uint":1}}"#,
        EnumGeneric::<bool>::A { uint: 1, _p: PhantomData },
    );
    test_instruction_equivalence(
        app.world(),
        "EnumGeneric<UnitStruct>::B{s_enum:B(())}",
        r#"{"B":{"s_enum":{"B":[]}}}"#,
        EnumGeneric::<UnitStruct>::B { s_enum: EnumStruct::B(UnitStruct), _p: PhantomData },
    );
    test_instruction_equivalence(
        app.world(),
        "EnumGeneric<SingleGeneric<u32>>::A{uint:1}",
        r#"{"A":{"uint":1}}"#,
        EnumGeneric::<SingleGeneric<u32>>::A { uint: 1, _p: PhantomData },
    );
}

//-------------------------------------------------------------------------------------------------------------------
