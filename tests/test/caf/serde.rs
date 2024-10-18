//! Serializing and deserializing instructions and values.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use bevy::prelude::*;

use super::helpers::*;

// TODO: test lossy conversions (scientific notation, multiline strings, manual builtin to auto-builtin,
// reflect-defaulted fields, unicode with leading zeros, ??) (these require parsing to be implemented)

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn unit_struct()
{
    let a = prepare_test_app();
    test_equivalence(a.world(), "UnitStruct", "()", UnitStruct);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn plain_struct()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "PlainStruct{boolean:false}",
        "{boolean:false}",
        PlainStruct { boolean: false },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn numbers()
{
    let a = prepare_test_app();
    let w = a.world();
    test_equivalence(w, "FloatStruct(0)", "0", FloatStruct(0.0f64));
    test_equivalence(w, "FloatStruct(0.1)", "0.1", FloatStruct(0.1f64));
    test_equivalence(w, "FloatStruct(1)", "1", FloatStruct(1.0f64));
    test_equivalence_skip_value_eq(w, "FloatStruct(nan)", "nan", FloatStruct(f64::NAN));
    test_equivalence(w, "FloatStruct(inf)", "inf", FloatStruct(f64::INFINITY));
    test_equivalence(w, "FloatStruct(-inf)", "-inf", FloatStruct(f64::NEG_INFINITY));
    test_equivalence(w, "FloatStruct(10000000)", "10000000", FloatStruct(10000000.0f64));
    test_equivalence(
        w,
        "FloatStruct(10000000000)",
        "10000000000",
        FloatStruct(10000000000.0f64),
    );
    test_equivalence(
        w,
        "FloatStruct(1.002002e17)",
        "1.002002e17",
        FloatStruct(100200200000000000.0f64),
    );
    test_equivalence(
        w,
        "FloatStruct(-1.002002e17)",
        "-1.002002e17",
        FloatStruct(-100200200000000000.0f64),
    );
    test_equivalence(w, "FloatStruct(1e-7)", "1e-7", FloatStruct(0.0000001f64));
    test_equivalence(
        w,
        "FloatStruct(1.0000000001)",
        "1.0000000001",
        FloatStruct(1.0000000001f64),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn strings()
{
    let a = prepare_test_app();
    let w = a.world();
    test_equivalence(w, "StringStruct(\"\")", "\"\"", StringStruct("".into()));
    test_equivalence(w, "StringStruct(\"hi\")", "\"hi\"", StringStruct("hi".into()));
    test_equivalence(w, "StringStruct(\"hi\\n\")", "\"hi\\n\"", StringStruct("hi\n".into()));
    test_equivalence(
        w,
        "StringStruct(\"hi\\nhi\")",
        "\"hi\\nhi\"",
        StringStruct("hi\nhi".into()),
    );
    test_equivalence(w, "StringStruct(\"\\u{df}\")", "\"\\u{df}\"", StringStruct("ß".into()));
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn newtypes()
{
    let a = prepare_test_app();
    test_equivalence(a.world(), "NewtypeStruct(1)", "1", NewtypeStruct(1));
    test_equivalence(
        a.world(),
        "WrapNewtypeStruct(1)",
        "1",
        WrapNewtypeStruct(NewtypeStruct(1)),
    );
    test_equivalence(a.world(), "NewtypeEnum::Tuple(())", "Tuple(())", NewtypeEnum::Tuple(()));
    test_equivalence(
        a.world(),
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
    let a = prepare_test_app();
    test_equivalence(a.world(), "EnumStruct::A", "A", EnumStruct::A);
    test_equivalence(a.world(), "EnumStruct::B(())", "B(())", EnumStruct::B(UnitStruct));
    test_equivalence(
        a.world(),
        "EnumStruct::C{boolean:true s_plain:{boolean:true}}",
        "C{boolean:true s_plain:{boolean:true}}",
        EnumStruct::C { boolean: true, s_plain: PlainStruct { boolean: true } },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn aggregate_struct()
{
    let a = prepare_test_app();
    // TODO: can only test keys that implement Ord, and entries must be sorted in the data representations for
    // consistency
    let mut map = BTreeMap::default();
    map.insert(10u32, 10u32);
    map.insert(20u32, 20u32);
    test_equivalence(
        a.world(),
        r#"AggregateStruct{uint:1 float:1 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10 20:20} s_struct:() s_enum:B(()) s_plain:{boolean:true}}"#,
        r#"{uint:1 float:1 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10 20:20} s_struct:() s_enum:B(()) s_plain:{boolean:true}}"#,
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
    let a = prepare_test_app();
    test_equivalence(a.world(), "WrapArray[]", "[]", WrapArray(vec![]));
    test_equivalence(a.world(), "WrapArray[()]", "[()]", WrapArray(vec![UnitStruct]));
    test_equivalence(
        a.world(),
        "WrapArray[() ()]",
        "[() ()]",
        WrapArray(vec![UnitStruct, UnitStruct]),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn tuple_struct()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "TupleStruct(() {boolean:true} true)",
        "(() {boolean:true} true)",
        TupleStruct(UnitStruct, PlainStruct { boolean: true }, true),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn single_generic()
{
    let a = prepare_test_app();
    test_equivalence(a.world(), "SingleGeneric<u32>", "{}", SingleGeneric::<u32>::default());
    test_equivalence(
        a.world(),
        "SingleGeneric<(u32, u32)>",
        "{}",
        SingleGeneric::<(u32, u32)>::default(),
    );
    test_equivalence(
        a.world(),
        "SingleGeneric<UnitStruct>",
        "{}",
        SingleGeneric::<UnitStruct>::default(),
    );
    test_equivalence(
        a.world(),
        "SingleGeneric<SingleGeneric<u32>>",
        "{}",
        SingleGeneric::<SingleGeneric<u32>>::default(),
    );
    test_equivalence(
        a.world(),
        "SingleGeneric<MultiGeneric<u32, u32, u32>>",
        "{}",
        SingleGeneric::<MultiGeneric<u32, u32, u32>>::default(),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn single_generic_tuple()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "SingleGenericTuple<u32>(1)",
        "1",
        SingleGenericTuple::<u32>(1),
    );
    test_equivalence(
        a.world(),
        "SingleGenericTuple<UnitStruct>(())",
        "()",
        SingleGenericTuple::<UnitStruct>(UnitStruct),
    );
    test_equivalence(
        a.world(),
        "SingleGenericTuple<SingleGeneric<u32>>({})",
        "{}",
        SingleGenericTuple::<SingleGeneric<u32>>(SingleGeneric::default()),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn multi_generic()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "MultiGeneric<u32, u32, u32>",
        "{}",
        MultiGeneric::<u32, u32, u32>::default(),
    );
    test_equivalence(
        a.world(),
        "MultiGeneric<u32, u32, UnitStruct>",
        "{}",
        MultiGeneric::<u32, u32, UnitStruct>::default(),
    );
    test_equivalence(
        a.world(),
        "MultiGeneric<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>",
        "{}",
        MultiGeneric::<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>::default(),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn enum_generic()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "EnumGeneric<bool>::A{uint:1}",
        "A{uint:1}",
        EnumGeneric::<bool>::A { uint: 1, _p: PhantomData },
    );
    test_equivalence(
        a.world(),
        "EnumGeneric<UnitStruct>::B{s_enum:B(())}",
        "B{s_enum:B(())}",
        EnumGeneric::<UnitStruct>::B { s_enum: EnumStruct::B(UnitStruct), _p: PhantomData },
    );
    test_equivalence(
        a.world(),
        "EnumGeneric<SingleGeneric<u32>>::A{uint:1}",
        "A{uint:1}",
        EnumGeneric::<SingleGeneric<u32>>::A { uint: 1, _p: PhantomData },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn builtins()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "BuiltinCollection{auto:auto px:0px percent:1% vw:1vw vh:1vh vmin:1vmin vmax:1vmax color:#FFFFFF}",
        "{auto:auto px:0px percent:1% vw:1vw vh:1vh vmin:1vmin vmax:1vmax color:#FFFFFF}",
        BuiltinCollection {
            auto: Val::Auto,
            px: Val::Px(0.0),
            percent: Val::Percent(1.0),
            vw: Val::Vw(1.0),
            vh: Val::Vh(1.0),
            vmin: Val::VMin(1.0),
            vmax: Val::VMax(1.0),
            color: Color::Srgba(Default::default()),
        },
    );
    test_equivalence(
        a.world(),
        "BuiltinCollection{auto:auto px:1.1px percent:1.1% vw:1.1vw vh:1.1vh vmin:1.1vmin vmax:1.1vmax color:#FF0000}",
        "{auto:auto px:1.1px percent:1.1% vw:1.1vw vh:1.1vh vmin:1.1vmin vmax:1.1vmax color:#FF0000}",
        BuiltinCollection {
            auto: Val::Auto,
            px: Val::Px(1.1),
            percent: Val::Percent(1.1),
            vw: Val::Vw(1.1),
            vh: Val::Vh(1.1),
            vmin: Val::VMin(1.1),
            vmax: Val::VMax(1.1),
            color: Color::Srgba(Srgba::RED),
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------
