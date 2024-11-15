use bevy::prelude::*;

use crate::load_embedded_scene_file;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CobEditorTemplatePlugin;

impl Plugin for CobEditorTemplatePlugin
{
    fn build(&self, app: &mut App)
    {
        load_embedded_scene_file!(app, "bevy_cobweb_ui", "src/editor/template", "frame.cob");
    }
}

//-------------------------------------------------------------------------------------------------------------------
