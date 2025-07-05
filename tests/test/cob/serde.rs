//! Serializing and deserializing instructions and values.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_cobweb_ui::prelude::{GridVal, GridValRepetition, RepeatedGridVal};

use super::helpers::*;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn unit_struct()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

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

    // Lossy conversion: leading zeroes in floats
    test_equivalence_lossy(w, "FloatStruct(00.0)", "FloatStruct(0)", FloatStruct(00.0f64));
    test_equivalence_lossy(w, "FloatStruct(01.1)", "FloatStruct(1.1)", FloatStruct(01.1f64));

    // Lossy conversion: trailing zeroes in floats
    test_equivalence_lossy(w, "FloatStruct(1.0)", "FloatStruct(1)", FloatStruct(1.0f64));
    test_equivalence_lossy(w, "FloatStruct(1.10)", "FloatStruct(1.1)", FloatStruct(1.10f64));

    // Lossy conversion: scientific notation
    test_equivalence_lossy(w, "FloatStruct(1e5)", "FloatStruct(100000)", FloatStruct(1e5f64));
    test_equivalence_lossy(w, "FloatStruct(-1e5)", "FloatStruct(-100000)", FloatStruct(-1e5f64));
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

    // Lossy conversion: leading zeroes in unicode sequence
    test_equivalence_lossy(
        w,
        "StringStruct(\"\\u{00df}\")",
        "StringStruct(\"\\u{df}\")",
        StringStruct("ß".into()),
    );

    // Lossy conversion: unicode sequence lowercased
    test_equivalence_lossy(
        w,
        "StringStruct(\"\\u{DF}\")",
        "StringStruct(\"\\u{df}\")",
        StringStruct("ß".into()),
    );

    // Lossy conversion: escapable characters will be escaped
    test_equivalence_lossy(
        w,
        "StringStruct(\"\nß\")",
        "StringStruct(\"\\n\\u{df}\")",
        StringStruct("\nß".into()),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn string_conversions()
{
    // Single-segment strings
    test_string_conversion("", "", "", 1);
    test_string_conversion("a", "a", "a", 1);
    test_string_conversion("a\n", "a\n", "a\\n", 1);
    test_string_conversion("a\nb", "a\nb", "a\\nb", 1);
    test_string_conversion("a\\nb", "a\nb", "a\\nb", 1);
    test_string_conversion("a\\\\b", "a\\b", "a\\\\b", 1);

    // Multi-segment strings
    test_string_conversion("\\\n", "", "", 2);
    test_string_conversion("a\\\nb", "ab", "ab", 2);
    test_string_conversion("a\\\n b", "ab", "ab", 2);
    test_string_conversion("a\\\nb\\\nc", "abc", "abc", 3);
    test_string_conversion("a\na1 a2\\\n\nb\\\nc d", "a\na1 a2\nbc d", "a\\na1 a2\\nbc d", 3);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn newtypes()
{
    let a = prepare_test_app();
    test_equivalence(a.world(), "NewtypeStruct<u32>(1)", "1", NewtypeStruct(1u32));
    test_equivalence(
        a.world(),
        "NewtypeStruct<NewtypeStruct<u32>>(1)",
        "1",
        NewtypeStruct(NewtypeStruct(1u32)),
    );
    test_equivalence(a.world(), "NewtypeEnum::Tuple", "Tuple", NewtypeEnum::Tuple(()));
    test_equivalence(
        a.world(),
        "ContainsNewtypes{n:1 w:[()]}",
        "{n:1 w:[()]}",
        ContainsNewtypes {
            n: WrapNewtypeStruct(NewtypeStruct(1u32)),
            w: WrapArray(vec![UnitStruct]),
        },
    );
    test_equivalence(a.world(), "WrapArray", "[]", WrapArray(vec![]));
    test_equivalence(a.world(), "WrapArray[()]", "[()]", WrapArray(vec![UnitStruct]));
    test_equivalence(
        a.world(),
        "WrapArray[() ()]",
        "[() ()]",
        WrapArray(vec![UnitStruct, UnitStruct]),
    );

    // Lossy conversion: newtype of unit struct is flattened
    test_equivalence_lossy(
        a.world(),
        "NewtypeStruct<UnitStruct>(())",
        "NewtypeStruct<UnitStruct>",
        NewtypeStruct(UnitStruct),
    );

    // Lossy conversion: newtype of newtype of unit struct is flattened
    test_equivalence_lossy(
        a.world(),
        "NewtypeStruct<NewtypeStruct<UnitStruct>>(())",
        "NewtypeStruct<NewtypeStruct<UnitStruct>>",
        NewtypeStruct(NewtypeStruct(UnitStruct)),
    );

    // Lossy conversion: newtype of empty tuple is flattened
    test_equivalence_lossy(
        a.world(),
        "NewtypeStruct<()>(())",
        "NewtypeStruct<()>",
        NewtypeStruct(()),
    );

    // Lossy conversion: newtype of tuple is flattened
    test_equivalence_lossy(
        a.world(),
        "NewtypeStruct<(u32, u32)>((1 1))",
        "NewtypeStruct<(u32, u32)>(1 1)",
        NewtypeStruct((1u32, 1u32)),
    );

    // Lossy conversion: newtype of tuple-struct is flattened
    test_equivalence_lossy(
        a.world(),
        "NewtypeStruct<SimpleTupleStruct>((1 1))",
        "NewtypeStruct<SimpleTupleStruct>(1 1)",
        NewtypeStruct(SimpleTupleStruct(1u32, 1u32)),
    );

    // Lossy conversion: newtype of vec is flattened
    test_equivalence_lossy(
        a.world(),
        "WrapArray([()])",
        "WrapArray[()]",
        WrapArray(vec![UnitStruct]),
    );

    // Lossy conversion: newtype of struct is flattened
    test_equivalence_lossy(
        a.world(),
        "NewtypeStruct<SimpleStruct>({a:1 b:2})",
        "NewtypeStruct<SimpleStruct>{a:1 b:2}",
        NewtypeStruct(SimpleStruct { a: 1, b: 2 }),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn enum_struct()
{
    let a = prepare_test_app();
    test_equivalence(a.world(), "EnumStruct::A", "A", EnumStruct::A);
    test_equivalence(a.world(), "EnumStruct::B", "B", EnumStruct::B(UnitStruct));
    test_equivalence(
        a.world(),
        "EnumStruct::C{boolean:true s_plain:{boolean:true}}",
        "C{boolean:true s_plain:{boolean:true}}",
        EnumStruct::C { boolean: true, s_plain: PlainStruct { boolean: true } },
    );
    test_equivalence(
        a.world(),
        "EnumStruct::D(1 2)",
        "D(1 2)",
        EnumStruct::D(SimpleTupleStruct(1, 2)),
    );
    test_equivalence(
        a.world(),
        "EnumStruct::E{a:1 b:2}",
        "E{a:1 b:2}",
        EnumStruct::E(SimpleStruct { a: 1, b: 2 }),
    );

    // Lossy conversion: newtype-variant of tuple-struct is flattened
    test_equivalence_lossy(
        a.world(),
        "EnumStruct::D((1 2))",
        "EnumStruct::D(1 2)",
        EnumStruct::D(SimpleTupleStruct(1, 2)),
    );

    // Lossy conversion: newtype-variant of struct is flattened
    test_equivalence_lossy(
        a.world(),
        "EnumStruct::E({a:1 b:2})",
        "EnumStruct::E{a:1 b:2}",
        EnumStruct::E(SimpleStruct { a: 1, b: 2 }),
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
        r#"AggregateStruct{uint:1 float:1 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10 20:20} s_struct:() s_enum:B s_plain:{boolean:true}}"#,
        r#"{uint:1 float:1 boolean:true string:"hi" vec:[{boolean:true} {boolean:false}] map:{10:10 20:20} s_struct:() s_enum:B s_plain:{boolean:true}}"#,
        AggregateStruct {
            uint: 1,
            float: 1.0,
            boolean: true,
            string: String::from("hi"),
            vec: vec![PlainStruct { boolean: true }, PlainStruct { boolean: false }],
            map,
            s_struct: UnitStruct,
            s_enum: EnumStruct::B(UnitStruct),
            s_plain: PlainStruct { boolean: true },
        },
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
    test_equivalence(a.world(), "SingleGeneric<u32>", "()", SingleGeneric::<u32>::default());
    test_equivalence(
        a.world(),
        "SingleGeneric<(u32, u32)>",
        "()",
        SingleGeneric::<(u32, u32)>::default(),
    );
    test_equivalence(
        a.world(),
        "SingleGeneric<UnitStruct>",
        "()",
        SingleGeneric::<UnitStruct>::default(),
    );
    test_equivalence(
        a.world(),
        "SingleGeneric<SingleGeneric<u32>>",
        "()",
        SingleGeneric::<SingleGeneric<u32>>::default(),
    );
    test_equivalence(
        a.world(),
        "SingleGeneric<MultiGeneric<u32, u32, u32>>",
        "()",
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
        "SingleGenericTuple<UnitStruct>",
        "()",
        SingleGenericTuple::<UnitStruct>(UnitStruct),
    );
    test_equivalence(
        a.world(),
        "SingleGenericTuple<SingleGeneric<u32>>",
        "()",
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
        "()",
        MultiGeneric::<u32, u32, u32>::default(),
    );
    test_equivalence(
        a.world(),
        "MultiGeneric<u32, u32, UnitStruct>",
        "()",
        MultiGeneric::<u32, u32, UnitStruct>::default(),
    );
    test_equivalence(
        a.world(),
        "MultiGeneric<SingleGeneric<u32>, SingleGeneric<SingleGeneric<u32>>, SingleGeneric<u32>>",
        "()",
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
        "EnumGeneric<UnitStruct>::B{s_enum:B}",
        "B{s_enum:B}",
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
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "BuiltinCollection{auto_val:auto px:0px percent:1% vw:1vw vh:1vh vmin:1vmin vmax:1vmax fr:1fr minmax:MinMax[auto 1px] \
            repeated_single_auto:(Count(1) auto) repeated_many:(Count(2) Many[auto 1px]) color:#FFFFFF}",
        "{auto_val:auto px:0px percent:1% vw:1vw vh:1vh vmin:1vmin vmax:1vmax fr:1fr minmax:MinMax[auto 1px] \
            repeated_single_auto:(Count(1) auto) repeated_many:(Count(2) Many[auto 1px]) color:#FFFFFF}",
        BuiltinCollection {
            auto_val: Val::Auto,
            px: Val::Px(0.0),
            percent: Val::Percent(1.0),
            vw: Val::Vw(1.0),
            vh: Val::Vh(1.0),
            vmin: Val::VMin(1.0),
            vmax: Val::VMax(1.0),
            fr: GridVal::Fraction(1.0),
            minmax: GridVal::MinMax(vec![GridVal::Auto, GridVal::Px(1.0)]),
            repeated_single_auto: RepeatedGridVal(GridValRepetition::Count(1), GridVal::Auto),
            repeated_many: RepeatedGridVal(GridValRepetition::Count(2), GridVal::Many(vec![GridVal::Auto, GridVal::Px(1.0)])),
            color: Color::Srgba(Default::default()),
        },
    );
    test_equivalence(
        a.world(),
        "BuiltinCollection{auto_val:auto px:1.1px percent:1.1% vw:1.1vw vh:1.1vh vmin:1.1vmin vmax:1.1vmax fr:1.1fr minmax:MinMax[auto 1.1px] \
            repeated_single_auto:(Count(1) auto) repeated_many:(Count(2) Many[auto 1.1px]) color:#FF0000}",
        "{auto_val:auto px:1.1px percent:1.1% vw:1.1vw vh:1.1vh vmin:1.1vmin vmax:1.1vmax fr:1.1fr minmax:MinMax[auto 1.1px] \
            repeated_single_auto:(Count(1) auto) repeated_many:(Count(2) Many[auto 1.1px]) color:#FF0000}",
        BuiltinCollection {
            auto_val: Val::Auto,
            px: Val::Px(1.1),
            percent: Val::Percent(1.1),
            vw: Val::Vw(1.1),
            vh: Val::Vh(1.1),
            vmin: Val::VMin(1.1),
            vmax: Val::VMax(1.1),
            fr: GridVal::Fraction(1.1),
            minmax: GridVal::MinMax(vec![GridVal::Auto, GridVal::Px(1.1)]),
            repeated_single_auto: RepeatedGridVal(GridValRepetition::Count(1), GridVal::Auto),
            repeated_many: RepeatedGridVal(GridValRepetition::Count(2), GridVal::Many(vec![GridVal::Auto, GridVal::Px(1.1)])),
            color: Color::Srgba(Srgba::RED),
        },
    );

    // Lossy conversion: hex color will be uppercased
    test_equivalence_lossy(
        a.world(),
        "BuiltinColor(#ff0000)",
        "BuiltinColor(#FF0000)",
        BuiltinColor(Color::Srgba(Srgba::RED)),
    );

    // Lossy conversion: RepeatedGridVal will be expanded
    test_equivalence_lossy(
        a.world(),
        "BuiltinRepeatedGridVal(auto)",
        "BuiltinRepeatedGridVal(Count(1) auto)",
        BuiltinRepeatedGridVal(RepeatedGridVal(GridValRepetition::Count(1), GridVal::Auto)),
    );
    test_equivalence_lossy(
        a.world(),
        "BuiltinRepeatedGridVal(MinContent)",
        "BuiltinRepeatedGridVal(Count(1) MinContent)",
        BuiltinRepeatedGridVal(RepeatedGridVal(GridValRepetition::Count(1), GridVal::MinContent)),
    );
    test_equivalence_lossy(
        a.world(),
        "BuiltinRepeatedGridVal(1px)",
        "BuiltinRepeatedGridVal(Count(1) 1px)",
        BuiltinRepeatedGridVal(RepeatedGridVal(GridValRepetition::Count(1), GridVal::Px(1.0))),
    );
    test_equivalence_lossy(
        a.world(),
        "BuiltinRepeatedGridVal(1fr)",
        "BuiltinRepeatedGridVal(Count(1) 1fr)",
        BuiltinRepeatedGridVal(RepeatedGridVal(GridValRepetition::Count(1), GridVal::Fraction(1.0))),
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn reflect_defaulted()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "ReflectDefaulted{a:1 b:2}",
        "{a:1 b:2}",
        ReflectDefaulted { a: 1, b: 2 },
    );

    // Lossy conversion: reflect-defaulted fields will be inserted on reserialize
    test_equivalence_lossy_reflection(
        a.world(),
        "ReflectDefaulted",
        "ReflectDefaulted{a:0 b:0}",
        ReflectDefaulted::default(),
    );
    test_equivalence_lossy_reflection(
        a.world(),
        "ReflectDefaulted{b:1}",
        "ReflectDefaulted{a:0 b:1}",
        ReflectDefaulted { b: 1, ..Default::default() },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn reflect_default_newtype()
{
    let a = prepare_test_app();
    test_equivalence(a.world(), "ReflectDefaultNewtype(1)", "1", ReflectDefaultNewtype(1));

    // Lossy conversion: reflect-defaulted fields will be inserted on reserialize
    // TODO: requires bevy_reflect update AND a solution to this problem:
    // - To support #[reflect(default)] in newtypes, I need a way to 'back out' of the loadable erased newtype
    // deserializer on value-request-error to instead hand it an empty sequence. The problem is Visitor gets
    // consumed when you call its methods. So there's no way to 'on failure, try something else'.
    // test_equivalence_lossy_reflection(
    //     a.world(),
    //     "ReflectDefaultNewtype",
    //     "ReflectDefaultNewtype(0)",
    //     ReflectDefaultNewtype::default(),
    // );
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn serde_reflect_defaulted()
{
    let a = prepare_test_app();
    test_equivalence(
        a.world(),
        "SerdeReflectDefaulted{a:1 b:2}",
        "{a:1 b:2}",
        SerdeReflectDefaulted { a: 1, b: 2 },
    );

    // Lossy conversion: reflect-defaulted fields will be inserted on reserialize
    test_equivalence_lossy_reflection(
        a.world(),
        "SerdeReflectDefaulted",
        "SerdeReflectDefaulted{a:10 b:20}",
        SerdeReflectDefaulted::default(),
    );
    test_equivalence_lossy_reflection(
        a.world(),
        "SerdeReflectDefaulted{b:1}",
        "SerdeReflectDefaulted{a:10 b:1}",
        SerdeReflectDefaulted { b: 1, ..Default::default() },
    );
}

//-------------------------------------------------------------------------------------------------------------------
