use bevy::prelude::*;

//use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct BuiltinColorsPlugin;

impl Plugin for BuiltinColorsPlugin
{
    fn build(&self, _app: &mut App)
    {
        // TODO: re-enable once COB constants are implemented
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "basic.cob.json");
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "colors.cob.json");
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "css.cob.json");
        //load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/builtin/colors", "tailwind.cob.json");
    }
}

//-------------------------------------------------------------------------------------------------------------------
