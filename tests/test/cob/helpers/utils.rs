use bevy::prelude::*;
use bevy_cobweb_ui::prelude::cob::*;

use crate::cob::helpers::*;

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_test_app() -> App
{
    let mut app = App::new();
    app.add_plugins(SerdeTypesPlugin);
    app.update();
    app
}

//-------------------------------------------------------------------------------------------------------------------

pub fn test_span(val: &str) -> Span
{
    Span::new_extra(val, CobLocationMetadata { file: "test.cob" })
}

//-------------------------------------------------------------------------------------------------------------------
