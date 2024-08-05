use bevy::prelude::*;

use crate::widgets::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CobwebWidgetsPlugin;

impl Plugin for CobwebWidgetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(radio_button::CobwebRadioButtonPlugin);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/*
todo

- add built-in color schemes (css::tailwind)
- enable customizing widget theming
    - radio_button_style_base.caf.json
        - imports built-in color schemes
        - imports built-in animation configs?
    - radio_button_style.caf.json
        - imports radio_button_style_base.caf.json
        - manifest key: builtin.widgets.radio_button.style
    - add plugin configs for disabling radio_button_style.caf.json
        - then the user can load their own file with manifest key `builtin.widgets.radio_button.style`
*/
