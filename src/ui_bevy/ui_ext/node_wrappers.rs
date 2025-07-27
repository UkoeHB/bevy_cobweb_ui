use bevy::prelude::*;
use bevy::ui::UiSystem;
use bevy_cobweb_ui_core::ui::*;
use cob_sickle_ui_scaffold::DynamicStylePostUpdate;

use crate::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

fn refresh_display_control(
    mut nodes: Query<
        (&mut Node, &DisplayControl, Option<&DisplayType>),
        Or<(Changed<Node>, Changed<DisplayControl>)>,
    >,
)
{
    for (mut node, control, maybe_cache) in nodes.iter_mut() {
        let cache = maybe_cache.copied().unwrap_or_default();
        let display = control.to_display(cache.into());
        if node.display != display {
            node.display = display;
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

impl StaticAttribute for DisplayControl
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct NodeWrappersPlugin;

impl Plugin for NodeWrappersPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_instruction_type::<AbsoluteNode>()
            .register_instruction_type::<FlexNode>()
            .register_instruction_type::<AbsoluteGridNode>()
            .register_instruction_type::<GridNode>()
            .register_static::<DisplayControl>()
            .add_systems(
                PostUpdate,
                refresh_display_control
                    .after(DynamicStylePostUpdate)
                    .before(UiSystem::Prepare),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
