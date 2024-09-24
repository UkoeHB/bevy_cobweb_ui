use bevy::utils::default;
use bevy_cobweb_ui::prelude::*;

use super::utils::caf_parse_test;

//-------------------------------------------------------------------------------------------------------------------

#[test]
fn whitespace()
{
    caf_parse_test(" ", Caf { end_fill: CafFill::space(), ..default() });
    caf_parse_test("  ", Caf { end_fill: CafFill::spaces(2), ..default() });
    caf_parse_test("\n", Caf { end_fill: CafFill::newline(), ..default() });
    caf_parse_test("\n\n", Caf { end_fill: CafFill::newlines(2), ..default() });
    caf_parse_test(
        " \n",
        Caf {
            end_fill: CafFill { segments: vec![CafFillSegment::comment(" \n")] },
            ..default()
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------
