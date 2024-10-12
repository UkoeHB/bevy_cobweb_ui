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

use std::collections::BTreeMap;
use std::marker::PhantomData;

use super::helpers::*;

// TODO: test built-in values
// TODO: test lossy conversions (scientific notation, multiline strings, manual builtin to auto-builtin, ??)

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn unit_struct()
{
    let app = prepare_test_app();
    test_equivalence(app.world(), "UnitStruct", "()", UnitStruct);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn plain_struct()
{
    let app = prepare_test_app();
    test_equivalence(
        app.world(),
        "PlainStruct{boolean:false}",
        "{boolean:false}",
        PlainStruct { boolean: false },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn newtypes()
{
    let app = prepare_test_app();
    test_equivalence(app.world(), "NewtypeStruct(1)", "1", NewtypeStruct(1));
    test_equivalence(
        app.world(),
        "WrapNewtypeStruct(1)",
        "1",
        WrapNewtypeStruct(NewtypeStruct(1)),
    );
    test_equivalence(
        app.world(),
        "NewtypeEnum::Tuple(())",
        "Tuple(())",
        NewtypeEnum::Tuple(()),
    );
    test_equivalence(
        app.world(),
        "ContainsNewtypes{n:1 w:[()]}",
        "{n:1 w:[()]}",
        ContainsNewtypes {
            n: WrapNewtypeStruct(NewtypeStruct(1)),
            w: WrapArray(vec![UnitStruct]),
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn enum_struct()
{
    let app = prepare_test_app();
    test_equivalence(app.world(), "EnumStruct::A", "A", EnumStruct::A);
    test_equivalence(app.world(), "EnumStruct::B(())", "B(())", EnumStruct::B(UnitStruct));
    test_equivalence(
        app.world(),
        "EnumStruct::C{boolean:true s_plain:{boolean:true}}",
        "C{boolean:true s_plain:{boolean:true}}",
        EnumStruct::C { boolean: true, s_plain: PlainStruct { boolean: true } },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn aggregate_struct()
{
    let app = prepare_test_app();
    // TODO: can only test keys that implement Ord, and entries must be sorted in the data representations for
    // consistency
    let mut map = BTreeMap::default();
    map.insert(10u32, 10u32);
    map.insert(20u32, 20u32);
    test_equivalence(
        app.world(),
        r#"AggregateStruct{uint:1 float:1.0 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10 20:20} s_struct:() s_enum:B(()) s_plain:{boolean:true}}"#,
        r#"{uint:1 float:1.0 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10 20:20} s_struct:() s_enum:B(()) s_plain:{boolean:true}}"#,
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
    test_equivalence(app.world(), "WrapArray[]", "[]", WrapArray(vec![]));
    test_equivalence(app.world(), "WrapArray[()]", "[()]", WrapArray(vec![UnitStruct]));
    test_equivalence(
        app.world(),
        "WrapArray[() ()]",
        "[() ()]",
        WrapArray(vec![UnitStruct, UnitStruct]),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn tuple_struct()
{
    let app = prepare_test_app();
    test_equivalence(
        app.world(),
        "TupleStruct(() {boolean:true} true)",
        "(() {boolean:true} true)",
        TupleStruct(UnitStruct, PlainStruct { boolean: true }, true),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn single_generic()
{
    let app = prepare_test_app();
    test_equivalence(app.world(), "SingleGeneric<u32>", "{}", SingleGeneric::<u32>::default());
    test_equivalence(
        app.world(),
        "SingleGeneric<(u32, u32)>",
        "{}",
        SingleGeneric::<(u32, u32)>::default(),
    );
    test_equivalence(
        app.world(),
        "SingleGeneric<UnitStruct>",
        "{}",
        SingleGeneric::<UnitStruct>::default(),
    );
    test_equivalence(
        app.world(),
        "SingleGeneric<SingleGeneric<u32>>",
        "{}",
        SingleGeneric::<SingleGeneric<u32>>::default(),
    );
    test_equivalence(
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
    test_equivalence(
        app.world(),
        "SingleGenericTuple<u32>(1)",
        "1",
        SingleGenericTuple::<u32>(1),
    );
    test_equivalence(
        app.world(),
        "SingleGenericTuple<UnitStruct>(())",
        "()",
        SingleGenericTuple::<UnitStruct>(UnitStruct),
    );
    test_equivalence(
        app.world(),
        "SingleGenericTuple<SingleGeneric<u32>>({})",
        "{}",
        SingleGenericTuple::<SingleGeneric<u32>>(SingleGeneric::default()),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn multi_generic()
{
    let app = prepare_test_app();
    test_equivalence(
        app.world(),
        "MultiGeneric<u32, u32, u32>",
        "{}",
        MultiGeneric::<u32, u32, u32>::default(),
    );
    test_equivalence(
        app.world(),
        "MultiGeneric<u32, u32, UnitStruct>",
        "{}",
        MultiGeneric::<u32, u32, UnitStruct>::default(),
    );
    test_equivalence(
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
    test_equivalence(
        app.world(),
        "EnumGeneric<bool>::A{uint:1}",
        "A{uint:1}",
        EnumGeneric::<bool>::A { uint: 1, _p: PhantomData },
    );
    test_equivalence(
        app.world(),
        "EnumGeneric<UnitStruct>::B{s_enum:B(())}",
        "B{s_enum:B(())}",
        EnumGeneric::<UnitStruct>::B { s_enum: EnumStruct::B(UnitStruct), _p: PhantomData },
    );
    test_equivalence(
        app.world(),
        "EnumGeneric<SingleGeneric<u32>>::A{uint:1}",
        "A{uint:1}",
        EnumGeneric::<SingleGeneric<u32>>::A { uint: 1, _p: PhantomData },
    );
}

//-------------------------------------------------------------------------------------------------------------------
