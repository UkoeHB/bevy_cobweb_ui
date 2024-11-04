use bevy::prelude::*;

use sickle_ui_scaffold::{ui_builder::*, ui_style::prelude::*};

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Icon;

impl Icon {
    fn bundle() -> impl Bundle {
        ImageBundle {
            style: Style {
                width: Val::Px(16.),
                height: Val::Px(16.),
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiIconExt {
    fn icon(&mut self, path: impl Into<String>) -> UiBuilder<Entity>;
}

impl UiIconExt for UiBuilder<'_, Entity> {
    fn icon(&mut self, path: impl Into<String>) -> UiBuilder<Entity> {
        let mut icon = self.spawn((Name::new("Icon"), Icon::bundle(), Icon));

        icon.style().image(ImageSource::Path(path.into()));

        icon
    }
}
