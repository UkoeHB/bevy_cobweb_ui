use bevy::utils::default;
use bevy_cobweb_ui::prelude::*;

use super::helpers::*;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn end_helpers()
{
    assert_eq!(CafFill::new("\n").ends_with_newline(), true);
    assert_eq!(CafFill::new(" \n").ends_with_newline(), true);
    assert_eq!(CafFill::new("\n ").ends_with_newline(), false);
    assert_eq!(CafFill::new(" \n").ends_newline_then_num_spaces(), Some(0));
    assert_eq!(CafFill::new("\n ").ends_newline_then_num_spaces(), Some(1));
    assert_eq!(CafFill::new(" ").ends_newline_then_num_spaces(), None);
    assert_eq!(CafFill::new("\n //").ends_newline_then_num_spaces(), None);
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn whitespace()
{
    caf_parse_test("\n", Caf { end_fill: CafFill::new("\n"), ..default() });
    caf_parse_test("\n\n", Caf { end_fill: CafFill::new("\n\n"), ..default() });
    caf_parse_test(" \n", Caf { end_fill: CafFill::new(" \n"), ..default() });
}

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn comments()
{
    caf_parse_test("//\n", Caf { end_fill: CafFill::new("//\n"), ..default() });
    caf_parse_test("// \n", Caf { end_fill: CafFill::new("// \n"), ..default() });
    caf_parse_test(" // \n", Caf { end_fill: CafFill::new(" // \n"), ..default() });
    caf_parse_test("//a\n", Caf { end_fill: CafFill::new("//a\n"), ..default() });
    caf_parse_test("/**/\n", Caf { end_fill: CafFill::new("/**/\n"), ..default() });
    caf_parse_test("/* a */\n", Caf { end_fill: CafFill::new("/* a */\n"), ..default() });
    caf_parse_test(
        "// b\n/* a */\n",
        Caf { end_fill: CafFill::new("// b\n/* a */\n"), ..default() },
    );
    caf_parse_test(
        "// b/* a */\n",
        Caf { end_fill: CafFill::new("// b/* a */\n"), ..default() },
    );
    caf_parse_test(
        "// b/* a */\n// x \n",
        Caf { end_fill: CafFill::new("// b/* a */\n// x \n"), ..default() },
    );
    caf_parse_test_fail("a /* a */\n", Caf { end_fill: CafFill::new(" /* a */\n"), ..default() });
}

//-------------------------------------------------------------------------------------------------------------------
