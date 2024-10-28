use bevy::prelude::*;
use bevy_cobweb_ui::prelude::caf::*;

use crate::caf::helpers::*;

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
    Span::new_extra(val, CafLocationMetadata { file: "test.caf" })
}

//-------------------------------------------------------------------------------------------------------------------
