use bevy::{prelude::*, ui::FocusPolicy};

use sickle_macros::UiContext;
use sickle_ui_scaffold::prelude::*;

use crate::sickle::widgets::layout::{
    container::UiContainerExt,
    label::{LabelConfig, UiLabelExt},
};

#[cfg(feature = "observable")]
#[derive(Event, Copy, Clone, Debug)]
pub struct RadioButtonChanged {
    pub selected: Option<usize>,
}

pub struct RadioGroupPlugin;

impl Plugin for RadioGroupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ComponentThemePlugin::<RadioGroup>::default(),
            ComponentThemePlugin::<RadioButton>::default(),
        ))
        .add_systems(
            Update,
            (toggle_radio_button, update_radio_group_buttons, update_radio_button)
                .chain()
                .after(FluxInteractionUpdate),
        );

        #[cfg(feature = "observable")]
        app.add_event::<RadioButtonChanged>();
    }
}

fn toggle_radio_button(
    mut q_radio_buttons: Query<(&mut RadioButton, &FluxInteraction), Changed<FluxInteraction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_group: Query<&mut RadioGroup>,
    mut commands: Commands,
) {
    for (mut radio_button, interaction) in &mut q_radio_buttons {
        if *interaction == FluxInteraction::Pressed {
            let mut changed = false;

            if radio_button.checked
                && radio_button.unselectable
                && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            {
                radio_button.checked = false;
                changed = true;
            } else if !radio_button.checked {
                radio_button.checked = true;
                changed = true;
            }

            if !changed {
                continue;
            }

            let Ok(mut radio_group) = q_group.get_mut(radio_button.group) else {
                continue;
            };

            radio_group.selected = if radio_button.checked {
                radio_button.index.into()
            } else {
                None
            };

            #[cfg(feature = "observable")]
            commands.trigger_targets(
                RadioButtonChanged { selected: radio_group.selected },
                radio_button.group,
            );
        }
    }
}

fn update_radio_group_buttons(
    mut q_radio_buttons: Query<(&RadioGroup, &Children), Changed<RadioGroup>>,
    mut q_radio_button: Query<&mut RadioButton>,
) {
    for (radio_group, children) in &mut q_radio_buttons {
        for child in children {
            if let Ok(mut button) = q_radio_button.get_mut(*child) {
                // This is to avoid double triggering the change
                let checked = radio_group.selected == button.index.into();
                if button.checked != checked {
                    button.checked = checked;
                }
            }
        }
    }
}

fn update_radio_button(
    q_radio_buttons: Query<(Entity, &RadioButton), Changed<RadioButton>>,
    mut commands: Commands,
) {
    for (entity, radio_button) in &q_radio_buttons {
        commands
            .style_unchecked(radio_button.radiomark)
            .visibility(match radio_button.checked {
                true => Visibility::Inherited,
                false => Visibility::Hidden,
            });

        if radio_button.checked {
            commands
                .entity(entity)
                .add_pseudo_state(PseudoState::Checked);
        } else {
            commands
                .entity(entity)
                .remove_pseudo_state(PseudoState::Checked);
        }
    }
}

#[derive(Component, Debug, Reflect, UiContext)]
#[reflect(Component)]
pub struct RadioGroup {
    pub selected: Option<usize>,
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self { selected: None }
    }
}

impl DefaultTheme for RadioGroup {
    fn default_theme() -> Option<Theme<RadioGroup>> {
        RadioGroup::theme().into()
    }
}

impl RadioGroup {
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select(&mut self, value: impl Into<Option<usize>>) {
        let selected = value.into();
        if self.selected != selected {
            self.selected = selected;
        }
    }

    pub fn theme() -> Theme<RadioGroup> {
        let base_theme = PseudoTheme::build(None, RadioGroup::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder) {
        style_builder.flex_wrap(FlexWrap::Wrap);
    }

    fn container() -> impl Bundle {
        (Name::new("Radio Group"), NodeBundle::default())
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct RadioButton {
    pub index: usize,
    pub checked: bool,
    unselectable: bool,
    group: Entity,
    radiomark_background: Entity,
    radiomark: Entity,
    label: Entity,
}

impl Default for RadioButton {
    fn default() -> Self {
        Self {
            index: 0,
            checked: false,
            unselectable: false,
            group: Entity::PLACEHOLDER,
            radiomark_background: Entity::PLACEHOLDER,
            radiomark: Entity::PLACEHOLDER,
            label: Entity::PLACEHOLDER,
        }
    }
}

impl UiContext for RadioButton {
    fn get(&self, target: &str) -> Result<Entity, String> {
        match target {
            RadioButton::RADIOMARK_BACKGROUND => Ok(self.radiomark_background),
            RadioButton::RADIOMARK => Ok(self.radiomark),
            RadioButton::LABEL => Ok(self.label),
            _ => Err(format!(
                "{} doesn't exist for RadioButton. Possible contexts: {:?}",
                target,
                Vec::from_iter(self.contexts())
            )),
        }
    }

    fn contexts(&self) -> impl Iterator<Item = &str> + '_ {
        [RadioButton::RADIOMARK_BACKGROUND, RadioButton::RADIOMARK, RadioButton::LABEL].into_iter()
    }
}

impl DefaultTheme for RadioButton {
    fn default_theme() -> Option<Theme<RadioButton>> {
        RadioButton::theme().into()
    }
}

impl RadioButton {
    pub const RADIOMARK_BACKGROUND: &'static str = "RadiomarkBackground";
    pub const RADIOMARK: &'static str = "Radiomark";
    pub const LABEL: &'static str = "Label";

    pub fn theme() -> Theme<RadioButton> {
        let base_theme = PseudoTheme::deferred(None, RadioButton::primary_style);
        let checked_theme = PseudoTheme::deferred(vec![PseudoState::Checked], RadioButton::checked_style);
        Theme::new(vec![base_theme, checked_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .height(Val::Px(theme_spacing.areas.small))
            .justify_content(JustifyContent::Start)
            .align_items(AlignItems::Center)
            .margin(UiRect::horizontal(Val::Px(theme_spacing.gaps.small)))
            .background_color(Color::NONE);

        style_builder
            .switch_target(RadioButton::RADIOMARK_BACKGROUND)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .align_content(AlignContent::Center)
            .size(Val::Px(theme_spacing.inputs.radio_button.radiomark_outer_size))
            .margin(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .border(UiRect::all(Val::Px(theme_spacing.inputs.radio_button.border_size)))
            .border_radius(BorderRadius::all(Val::Px(
                theme_spacing.inputs.radio_button.radiomark_outer_size,
            )))
            .visibility(Visibility::Inherited)
            .animated()
            .border_color(AnimatedVals {
                idle: colors.on(OnColor::SurfaceVariant),
                hover: colors.on(OnColor::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(RadioButton::RADIOMARK_BACKGROUND)
            .animated()
            .background_color(AnimatedVals {
                idle: Color::NONE,
                hover: colors.accent(Accent::Primary).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);

        style_builder
            .switch_target(RadioButton::RADIOMARK)
            .size(Val::Px(theme_spacing.inputs.radio_button.radiomark_size))
            .background_color(colors.on(OnColor::Primary))
            .border_radius(BorderRadius::all(Val::Px(
                theme_spacing.inputs.radio_button.radiomark_size,
            )));

        let font = theme_data
            .text
            .get(FontStyle::Body, FontScale::Medium, FontType::Regular);
        style_builder
            .switch_target(RadioButton::LABEL)
            .margin(UiRect::px(theme_spacing.gaps.small, theme_spacing.gaps.medium, 0., 0.))
            .sized_font(font)
            .animated()
            .font_color(AnimatedVals {
                idle: colors.on(OnColor::SurfaceVariant),
                hover: colors.on(OnColor::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.interaction_animation);
    }

    fn checked_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .switch_target(RadioButton::RADIOMARK_BACKGROUND)
            .background_color(colors.accent(Accent::Primary))
            .border(UiRect::all(Val::Px(0.)));

        style_builder
            .switch_target(RadioButton::RADIOMARK)
            .animated()
            .background_color(AnimatedVals {
                idle: colors.on(OnColor::Primary),
                enter_from: colors.on(OnColor::Surface).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(RadioButton::RADIOMARK)
            .animated()
            .size(AnimatedVals {
                idle: Val::Px(theme_spacing.inputs.radio_button.radiomark_size),
                enter_from: Val::Px(0.).into(),
                ..default()
            })
            .copy_from(theme_data.enter_animation);

        style_builder
            .switch_target(RadioButton::LABEL)
            .font_color(colors.on(OnColor::SurfaceVariant));
    }

    fn button(name: String) -> impl Bundle {
        (Name::new(name), ButtonBundle::default(), TrackedInteraction::default())
    }

    fn radio_mark_background() -> impl Bundle {
        (
            Name::new("Radiomark Background"),
            NodeBundle { focus_policy: FocusPolicy::Pass, ..default() },
            LockedStyleAttributes::lock(LockableStyleAttribute::FocusPolicy),
        )
    }

    fn radio_mark() -> impl Bundle {
        (
            Name::new("Radiomark"),
            NodeBundle { focus_policy: FocusPolicy::Pass, ..default() },
            LockedStyleAttributes::from_vec(vec![
                LockableStyleAttribute::FocusPolicy,
                LockableStyleAttribute::Visibility,
            ]),
        )
    }
}

pub trait UiRadioGroupExt {
    fn radio_group(
        &mut self,
        options: Vec<impl Into<String>>,
        selected: impl Into<Option<usize>>,
        unselectable: bool,
    ) -> UiBuilder<Entity>;
}

impl UiRadioGroupExt for UiBuilder<'_, Entity> {
    /// A simple radio group with options. Optionally, the radio group can be "unselected"
    ///
    /// ### PseudoState usage
    /// - `PseudoState::Checked` is added to the currently selected `RadioButton` entity
    fn radio_group(
        &mut self,
        options: Vec<impl Into<String>>,
        selected: impl Into<Option<usize>>,
        unselectable: bool,
    ) -> UiBuilder<Entity> {
        let mut radio_group = self.spawn((RadioGroup::container(), RadioGroup { selected: selected.into() }));

        let mut index = 0;
        let group = radio_group.id();
        for option in options {
            let label = option.into();
            let name = format!("Radio Button [{}]", label);
            let mut radio_button = RadioButton { checked: false, unselectable, index, group, ..default() };

            radio_group
                .container(RadioButton::button(name), |button| {
                    radio_button.radiomark_background = button
                        .container(RadioButton::radio_mark_background(), |radio_mark_bg| {
                            radio_button.radiomark = radio_mark_bg.spawn(RadioButton::radio_mark()).id();
                        })
                        .id();
                    radio_button.label = button.label(LabelConfig { label, ..default() }).id();
                })
                .insert(radio_button);

            index += 1;
        }

        radio_group
    }
}
