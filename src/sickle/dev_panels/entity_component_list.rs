use bevy::prelude::*;

use crate::prelude::*;

pub struct EntityComponentListPlugin;

impl Plugin for EntityComponentListPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EntityComponentTagPlugin)
            .add_systems(Update, update_entity_component_lists);
    }
}

fn update_entity_component_lists(world: &mut World) {
    let changed: Vec<(Entity, Option<Entity>)> = world
        .query::<(Entity, Ref<EntityComponentList>)>()
        .iter(world)
        .filter(|(_, list)| list.is_changed())
        .map(|(e, list_ref)| (e, list_ref.entity))
        .collect();

    for (container, selected_entity) in changed.iter().copied() {
        update_entity_component_list(container, selected_entity, world);
    }
}

fn update_entity_component_list(
    container: Entity,
    selected_entity: Option<Entity>,
    world: &mut World,
) {
    if let Some(selected) = selected_entity {
        if world.get_entity(selected).is_none() {
            world.commands().entity(container).despawn_descendants();

            return;
        }

        let debug_infos: Vec<_> = world
            .inspect_entity(selected)
            .into_iter()
            .map(UiUtils::simplify_component_name)
            .collect();

        // TODO: Maybe re-use existing tags if they exist
        world.commands().entity(container).despawn_descendants();
        let mut commands = world.commands();
        let mut builder = commands.ui_builder(container);
        for info in debug_infos.iter().cloned() {
            builder.entity_component_tag(info);
        }
    } else {
        world.commands().entity(container).despawn_descendants();
    }

    world.flush();
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct EntityComponentList {
    pub entity: Option<Entity>,
}

pub trait UiEntityComponentListExt {
    fn entity_component_list(&mut self, entity: Option<Entity>) -> UiBuilder<Entity>;
}

impl UiEntityComponentListExt for UiBuilder<'_, Entity> {
    fn entity_component_list(&mut self, entity: Option<Entity>) -> UiBuilder<Entity> {
        self.row(|row| {
            row.insert((
                Name::new("Entity Component List"),
                EntityComponentList { entity },
            ))
            .style()
            .overflow(Overflow::clip())
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::FlexStart)
            .align_content(AlignContent::FlexStart);
        })
    }
}

// TODO: Turn Tag into a standalone widget, use a theme override in the list container
pub struct EntityComponentTagPlugin;

impl Plugin for EntityComponentTagPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<EntityComponentTag>::default());
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct EntityComponentTag {
    label: Entity,
}

impl Default for EntityComponentTag {
    fn default() -> Self {
        Self {
            label: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for EntityComponentTag {
    fn default_theme() -> Option<Theme<EntityComponentTag>> {
        EntityComponentTag::theme().into()
    }
}

impl UiContext for EntityComponentTag {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            EntityComponentTag::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exist for EntityComponentTag. Possible contexts: {:?}",
                target,
                Vec::from_iter(self.contexts())
            )),
        }
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [EntityComponentTag::LABEL].into_iter()
    }
}

impl EntityComponentTag {
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<EntityComponentTag> {
        let base_theme = PseudoTheme::deferred(None, EntityComponentTag::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);

        style_builder
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .border_radius(BorderRadius::all(Val::Px(theme_spacing.corners.small)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.accent(Accent::Tertiary),
                enter_from: Color::NONE.into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(EntityComponentTag::LABEL)
            .sized_font(font)
            .font_color(colors.on(OnColor::Tertiary));
    }

    fn frame() -> impl Bundle {
        (Name::new("Entity Component Tag"), NodeBundle::default())
    }
}

pub trait UiEntityComponentTagExt {
    fn entity_component_tag(&mut self, label: String) -> UiBuilder<Entity>;
}

impl UiEntityComponentTagExt for UiBuilder<'_, Entity> {
    fn entity_component_tag(&mut self, label: String) -> UiBuilder<Entity> {
        let mut tag = EntityComponentTag::default();
        let mut widget = self.container(EntityComponentTag::frame(), |container| {
            tag.label = container.label(LabelConfig { label, ..default() }).id();
        });

        widget.insert(tag);

        widget
    }
}
