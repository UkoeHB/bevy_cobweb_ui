use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinColorsPlugin;

impl Plugin for BuiltinColorsPlugin
{
    fn build(&self, app: &mut App)
    {
        load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "basic.cob");
        load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "colors.cob");
        load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "css.cob");
        load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "tailwind.cob");
    }
}

//-------------------------------------------------------------------------------------------------------------------
