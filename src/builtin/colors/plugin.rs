use bevy::prelude::*;

//use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinColorsPlugin;

impl Plugin for BuiltinColorsPlugin
{
    fn build(&self, _app: &mut App)
    {
        // TODO: re-enable once CAF constants are implemented
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "basic.caf.json");
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "colors.caf.json");
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "css.caf.json");
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "tailwind.caf.json");
    }
}

//-------------------------------------------------------------------------------------------------------------------
