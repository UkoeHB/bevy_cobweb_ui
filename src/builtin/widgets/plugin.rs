use bevy::prelude::*;

use crate::builtin::widgets::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinWidgetsPlugin;

impl Plugin for BuiltinWidgetsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(checkbox::CobwebCheckboxPlugin)
            .add_plugins(radio_button::CobwebRadioButtonPlugin)
            .add_plugins(scroll::CobwebScrollPlugin)
            .add_plugins(slider::CobwebSliderPlugin)
            //.add_plugins(slider::CobwebTooltipPlugin)
            ;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/*
todo
- maybe not? kind of convoluted

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
