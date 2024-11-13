use bevy_cobweb_ui::prelude::cob::*;

use super::helpers::{test_cob, test_cob_fail};

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn defs_section_constants()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    let res = test_cob(
        b"#defs
",
    );
    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 0);

    let res = test_cob(
        b"#defs
$a = 10
",
    );
    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 1);
    let CobDefEntry::Constant(constant) = &defs.entries[0] else { unreachable!() };
    assert_eq!(constant.name.as_str(), "a");
    let CobConstantValue::Value(CobValue::Number(number)) = &constant.value else { unreachable!() };
    assert_eq!(number.number.as_u128().unwrap(), 10);

    let res = test_cob(
        b"
#defs
$a = 10
$b = X{ a: 1, b: 2 }
$c = $b
$d = $a::b::c
$e = \\ 10 10 10 $a \\
",
    );
    let CobSection::Defs(defs) = &res.sections[0] else { unreachable!() };
    assert_eq!(defs.entries.len(), 5);
    let CobDefEntry::Constant(constant) = &defs.entries[0] else { unreachable!() };
    assert_eq!(constant.name.as_str(), "a");
    let CobConstantValue::Value(CobValue::Number(number)) = &constant.value else { unreachable!() };
    assert_eq!(number.number.as_u128().unwrap(), 10);
    let CobDefEntry::Constant(constant) = &defs.entries[1] else { unreachable!() };
    assert_eq!(constant.name.as_str(), "b");
    assert!(matches!(constant.value, CobConstantValue::Value(CobValue::Enum(_))));
    let CobDefEntry::Constant(constant) = &defs.entries[2] else { unreachable!() };
    assert_eq!(constant.name.as_str(), "c");
    let CobConstantValue::Value(CobValue::Constant(assigned)) = &constant.value else { unreachable!() };
    assert_eq!(assigned.path.as_str(), "b");
    let CobDefEntry::Constant(constant) = &defs.entries[3] else { unreachable!() };
    assert_eq!(constant.name.as_str(), "d");
    let CobConstantValue::Value(CobValue::Constant(assigned)) = &constant.value else { unreachable!() };
    assert_eq!(assigned.path.as_str(), "a::b::c");
    let CobDefEntry::Constant(constant) = &defs.entries[4] else { unreachable!() };
    assert_eq!(constant.name.as_str(), "e");
    let CobConstantValue::ValueGroup(group) = &constant.value else { unreachable!() };
    assert_eq!(group.entries.len(), 4);
    let CobValueGroupEntry::Value(CobValue::Number(number)) = &group.entries[0] else { unreachable!() };
    assert_eq!(number.number.as_u128().unwrap(), 10);
    let CobValueGroupEntry::Value(CobValue::Constant(inner_const)) = &group.entries[3] else { unreachable!() };
    assert_eq!(inner_const.path.as_str(), "a");
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn constants_errors()
{
    /*
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    */

    // Non-definition in section
    test_cob_fail(
        b"#defs
$a = 10
1",
        b"1",
    );
    // Section not starting on newline
    test_cob_fail(
        b" #defs
", b"#defs\n",
    );
    // Entry not starting with newline
    test_cob_fail(
        b"#defs
 $a = 10",
        b"$a = 10",
    );
    // Definition is not lowercase
    test_cob_fail(
        b"#defs
$A = 10
",
        b"$A = 10\n",
    );
    // Definition contains path segments
    test_cob_fail(
        b"#defs
$a::b = 10
",
        b"::b = 10\n",
    );
}

//-------------------------------------------------------------------------------------------------------------------
