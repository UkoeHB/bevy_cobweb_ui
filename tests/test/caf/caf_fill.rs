use bevy::utils::default;
use bevy_cobweb_ui::prelude::*;

use super::utils::caf_parse_test;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn end_helpers()
{
    assert_eq!(CafFill::new("\n").ends_with_newline(), true);
    assert_eq!(CafFill::new(" \n").ends_with_newline(), true);
    assert_eq!(CafFill::new("\n ").ends_with_newline(), false);
    assert_eq!(CafFill::new(" \n").ends_newline_then_spaces(), Some(0));
    assert_eq!(CafFill::new("\n ").ends_newline_then_spaces(), Some(1));
    assert_eq!(CafFill::new(" ").ends_newline_then_spaces(), None);
    assert_eq!(CafFill::new("\n //").ends_newline_then_spaces(), None);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn whitespace()
{
    caf_parse_test(" ", Caf { end_fill: CafFill::new(" "), ..default() });
    caf_parse_test("  ", Caf { end_fill: CafFill::new("  "), ..default() });
    caf_parse_test("\n", Caf { end_fill: CafFill::new("\n"), ..default() });
    caf_parse_test("\n\n", Caf { end_fill: CafFill::new("\n\n"), ..default() });
    caf_parse_test(" \n", Caf { end_fill: CafFill::new(" \n"), ..default() });
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn comments()
{
    caf_parse_test("//", Caf { end_fill: CafFill::new("//"), ..default() });
    caf_parse_test("// ", Caf { end_fill: CafFill::new("// "), ..default() });
    caf_parse_test(" // ", Caf { end_fill: CafFill::new(" // "), ..default() });
    caf_parse_test("//a", Caf { end_fill: CafFill::new("//a"), ..default() });
    caf_parse_test("//a\n", Caf { end_fill: CafFill::new("//a\n"), ..default() });
    caf_parse_test("/**/", Caf { end_fill: CafFill::new("/**/"), ..default() });
    caf_parse_test("/* a */", Caf { end_fill: CafFill::new("/* a */"), ..default() });
    caf_parse_test("/* a */", Caf { end_fill: CafFill::new("/* a */"), ..default() });
    caf_parse_test("// b\n/* a */", Caf { end_fill: CafFill::new(" /* a */"), ..default() });
    caf_parse_test("// b/* a */", Caf { end_fill: CafFill::new("// b/* a */"), ..default() });
    caf_parse_test("// b/* a */\n// x ", Caf { end_fill: CafFill::new("// b/* a */\n// x "), ..default() });
    caf_parse_test_invalid("a /* a */", Caf { end_fill: CafFill::new(" /* a */"), ..default() });
}

//-------------------------------------------------------------------------------------------------------------------
