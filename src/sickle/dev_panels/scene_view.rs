use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    ui::widget::UiImageSize,
    utils::HashMap,
};

use sickle_macros::UiContext;
use sickle_ui_scaffold::prelude::*;

use crate::widgets::{
    inputs::{
        checkbox::{Checkbox, UiCheckboxExt},
        radio_group::{RadioGroup, UiRadioGroupExt},
        slider::{Slider, SliderAxis, SliderConfig, UiSliderExt},
    },
    layout::{column::UiColumnExt, row::UiRowExt},
    WidgetLibraryUpdate,
};

pub struct SceneViewPlugin;

impl Plugin for SceneViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveSceneViews>()
            .configure_sets(Update, SpawnSceneViewUpdate.after(WidgetLibraryUpdate))
            .add_plugins(ComponentThemePlugin::<SceneControls>::default())
            .add_systems(
                PreUpdate,
                (
                    spawn_scene_view,
                    cleanup_despawned_scene_views,
                    set_scene_view_cam_viewport,
                    update_scene_view_controls,
                )
                    .in_set(SpawnSceneViewPreUpdate),
            )
            .add_systems(
                Update,
                (process_scene_view_controls, update_scene_views)
                    .chain()
                    .in_set(SpawnSceneViewUpdate),
            );
    }
}

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SpawnSceneViewPreUpdate;

#[derive(SystemSet, Clone, Eq, Debug, Hash, PartialEq)]
pub struct SpawnSceneViewUpdate;

fn spawn_scene_view(
    q_spanw_scene_view: Query<(Entity, &SpawnSceneView), Added<SpawnSceneView>>,
    asset_server: Res<AssetServer>,
    mut active_scene_views: ResMut<ActiveSceneViews>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    for (container, spawn_scene_view) in &q_spanw_scene_view {
        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

        // This is the texture that will be rendered to.
        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        // fill image.data with zeroes
        image.resize(size);
        let image_handle = images.add(image);

        let scene_camera = commands
            .spawn((
                Camera3dBundle {
                    camera: Camera {
                        clear_color: ClearColorConfig::Custom(
                            bevy::color::palettes::css::DARK_GRAY.into(),
                        ),
                        order: 0,
                        target: image_handle.clone().into(),
                        ..default()
                    },
                    transform: Transform::from_xyz(0., 2., -3.)
                        .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                    ..default()
                },
                FogSettings {
                    color: Color::srgb(0.25, 0.25, 0.25),
                    falloff: FogFalloff::Linear {
                        start: 7.0,
                        end: 12.0,
                    },
                    ..default()
                },
            ))
            .id();

        let transform =
            Transform::from_xyz(0., 10., 3.).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        let scene_light = commands
            .spawn((DirectionalLightBundle {
                directional_light: DirectionalLight {
                    color: Color::srgb(1., 0.953, 0.886),
                    shadows_enabled: true,
                    ..default()
                },
                transform,
                ..default()
            },))
            .id();

        let mut transform = Transform::from_xyz(0., 0., 0.);
        transform.scale = Vec3::splat(1.);

        let asset = spawn_scene_view.asset_path.clone();
        let scene_asset = commands
            .spawn((TransformBundle::default(), VisibilityBundle::default()))
            .with_children(|scene| {
                scene.spawn((SceneBundle {
                    scene: asset_server.load(asset),
                    ..default()
                },));
            })
            .id();

        commands
            .entity(container)
            .insert((
                SceneView {
                    camera: scene_camera,
                    light: scene_light,
                    asset_root: scene_asset,
                },
                SceneViewSettings::default(),
                UiImage::new(image_handle),
                UiImageSize::default(),
            ))
            .remove::<SpawnSceneView>();

        commands.ui_builder(container).row(|scene_controls| {
            let radio_group_theme = PseudoTheme::build(None, |style_builder| {
                style_builder.flex_wrap(FlexWrap::NoWrap);
            });

            scene_controls.insert((
                SceneControls {
                    scene_view: container,
                },
                Theme::<RadioGroup>::new(vec![radio_group_theme]),
            ));

            scene_controls
                .checkbox(String::from("Rotate Scene"), false)
                .insert(SceneRotationControl {
                    scene_view: container,
                });
            scene_controls
                .slider(SliderConfig::new(
                    String::from("Rotation Speed"),
                    -1.,
                    1.,
                    0.1,
                    true,
                    SliderAxis::Horizontal,
                ))
                .insert(SceneRotationSpeedControl {
                    scene_view: container,
                })
                .style()
                .min_width(Val::Px(250.));
            scene_controls
                .row(|row| {
                    row.radio_group(vec!["Natural", "Dim", "Night"], 1, false)
                        .insert(SceneLightControl {
                            scene_view: container,
                        });
                })
                .style()
                .min_width(Val::Px(150.));
        });

        active_scene_views.scene_views.insert(
            container,
            SceneView {
                camera: scene_camera,
                light: scene_light,
                asset_root: scene_asset,
            },
        );
    }
}

fn cleanup_despawned_scene_views(
    mut q_removed_scene_views: RemovedComponents<SceneView>,
    mut active_scene_views: ResMut<ActiveSceneViews>,
    mut commands: Commands,
) {
    for entity in q_removed_scene_views.read() {
        let Some(data) = active_scene_views.scene_views.remove(&entity) else {
            error!("Tried to clean up untracked scene view {}", entity);
            continue;
        };

        commands.entity(data.asset_root).despawn_recursive();
        commands.entity(data.camera).despawn_recursive();
        commands.entity(data.light).despawn_recursive();
    }
}

fn set_scene_view_cam_viewport(
    q_scene_views: Query<(&SceneView, &Node), Changed<GlobalTransform>>,
    mut images: ResMut<Assets<Image>>,
    mut q_camera: Query<&mut Camera>,
) {
    for (scene_view, node) in &q_scene_views {
        let Ok(mut camera) = q_camera.get_mut(scene_view.camera()) else {
            continue;
        };

        let size = node.size();

        if size.x == 0. || size.y == 0. {
            camera.is_active = false;
            continue;
        }

        camera.is_active = true;

        if let RenderTarget::Image(render_texture) = camera.target.clone() {
            let Some(texture) = images.get_mut(&render_texture) else {
                continue;
            };

            let size = Extent3d {
                width: size.x as u32,
                height: size.y as u32,
                ..default()
            };

            texture.resize(size);
        }
    }
}

fn update_scene_view_controls(
    q_scene_view_settings: Query<&SceneViewSettings, Changed<SceneViewSettings>>,
    mut q_rotation_controls: Query<(&mut Checkbox, &SceneRotationControl)>,
    mut q_rotation_speed_controls: Query<(&mut Slider, &SceneRotationSpeedControl)>,
    mut q_light_controls: Query<(&mut RadioGroup, &SceneLightControl)>,
) {
    for (mut checkbox, control) in &mut q_rotation_controls {
        let Ok(settings) = q_scene_view_settings.get(control.scene_view) else {
            continue;
        };

        if checkbox.checked != settings.do_rotate {
            checkbox.checked = settings.do_rotate;
        }
    }

    for (mut slider, control) in &mut q_rotation_speed_controls {
        let Ok(settings) = q_scene_view_settings.get(control.scene_view) else {
            continue;
        };

        if slider.value() != settings.rotation_speed {
            slider.set_value(settings.rotation_speed);
        }
    }

    for (mut radio_group, control) in &mut q_light_controls {
        let Ok(settings) = q_scene_view_settings.get(control.scene_view) else {
            continue;
        };

        if radio_group.selected != settings.light.into() {
            radio_group.selected = settings.light.into();
        }
    }
}

fn process_scene_view_controls(
    mut q_scene_view_settings: Query<&mut SceneViewSettings>,
    q_rotation_controls: Query<(&Checkbox, &SceneRotationControl), Changed<Checkbox>>,
    q_rotation_speed_controls: Query<(&Slider, &SceneRotationSpeedControl), Changed<Slider>>,
    q_light_controls: Query<(&RadioGroup, &SceneLightControl), Changed<RadioGroup>>,
) {
    for (checkbox, control) in &q_rotation_controls {
        let Ok(mut settings) = q_scene_view_settings.get_mut(control.scene_view) else {
            continue;
        };

        if checkbox.checked != settings.do_rotate {
            settings.do_rotate = checkbox.checked;
        }
    }

    for (slider, control) in &q_rotation_speed_controls {
        let Ok(mut settings) = q_scene_view_settings.get_mut(control.scene_view) else {
            continue;
        };

        if slider.value() != settings.rotation_speed {
            settings.rotation_speed = slider.value();
        }
    }

    for (radio_group, control) in &q_light_controls {
        let Ok(mut settings) = q_scene_view_settings.get_mut(control.scene_view) else {
            continue;
        };

        if radio_group.selected != settings.light.into() {
            let Some(light) = radio_group.selected else {
                continue;
            };
            settings.light = light;
        }
    }
}

fn update_scene_views(
    time: Res<Time>,
    q_scene_views: Query<(&SceneView, Ref<SceneViewSettings>)>,
    mut ambient_light: ResMut<AmbientLight>,
    mut q_directional_light: Query<&mut DirectionalLight>,
    mut q_fog_settings: Query<&mut FogSettings>,
    mut q_transform: Query<&mut Transform>,
) {
    for (scene_view, settings) in &q_scene_views {
        let Ok(mut transform) = q_transform.get_mut(scene_view.camera()) else {
            continue;
        };

        if settings.do_rotate && settings.rotation_speed != 0. {
            transform.rotate_around(
                Vec3::ZERO,
                Quat::from_euler(
                    EulerRot::default(),
                    -time.delta_seconds() * settings.rotation_speed,
                    0.,
                    0.,
                ),
            );
        }

        if settings.is_changed() {
            let Ok(mut light) = q_directional_light.get_mut(scene_view.light()) else {
                continue;
            };
            let Ok(mut fog) = q_fog_settings.get_mut(scene_view.camera()) else {
                continue;
            };

            match settings.light {
                0 => {
                    light.color = Color::srgb(1., 0.953, 0.886);
                    light.illuminance = 13500.;
                    ambient_light.brightness = 500.;
                    fog.falloff = FogFalloff::Linear {
                        start: 7.0,
                        end: 12.0,
                    };
                }
                1 => {
                    light.color = Color::srgb(0.78, 0.76, 0.745);
                    light.illuminance = 9000.;
                    ambient_light.brightness = 300.;
                    fog.falloff = FogFalloff::Linear {
                        start: 6.0,
                        end: 15.0,
                    };
                }
                2 => {
                    light.color = Color::srgb(0.73, 0.90, 0.95); // Color::rgb(0.53, 0.8, 0.92);
                    light.illuminance = 300.;
                    ambient_light.brightness = 5.;
                    fog.falloff = FogFalloff::Linear {
                        start: 5.0,
                        end: 20.0,
                    };
                }
                _ => (),
            };
        }
    }
}

#[derive(Resource, Debug, Reflect)]
#[reflect(Resource)]
struct ActiveSceneViews {
    scene_views: HashMap<Entity, SceneView>,
}

impl Default for ActiveSceneViews {
    fn default() -> Self {
        Self {
            scene_views: HashMap::new(),
        }
    }
}

#[derive(Component, Debug, Reflect, UiContext)]
#[reflect(Component)]
struct SceneControls {
    scene_view: Entity,
}

impl Default for SceneControls {
    fn default() -> Self {
        Self {
            scene_view: Entity::PLACEHOLDER,
        }
    }
}

impl DefaultTheme for SceneControls {
    fn default_theme() -> Option<Theme<SceneControls>> {
        SceneControls::theme().into()
    }
}

impl SceneControls {
    pub fn theme() -> Theme<SceneControls> {
        let base_theme = PseudoTheme::deferred(None, SceneControls::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .justify_self(JustifySelf::Start)
            .height(Val::Px(theme_spacing.areas.small))
            .position_type(PositionType::Absolute)
            .background_color(colors.surface(Surface::Surface))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)));
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneRotationControl {
    scene_view: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneRotationSpeedControl {
    scene_view: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneLightControl {
    scene_view: Entity,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
struct SpawnSceneView {
    asset_path: String,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct SceneView {
    camera: Entity,
    light: Entity,
    asset_root: Entity,
}

impl Default for SceneView {
    fn default() -> Self {
        Self {
            camera: Entity::PLACEHOLDER,
            light: Entity::PLACEHOLDER,
            asset_root: Entity::PLACEHOLDER,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct SceneViewSettings {
    do_rotate: bool,
    rotation_speed: f32,
    light: usize,
}

impl Default for SceneViewSettings {
    fn default() -> Self {
        Self {
            do_rotate: false,
            rotation_speed: 0.1,
            light: 1,
        }
    }
}

impl SceneView {
    pub fn camera(&self) -> Entity {
        self.camera
    }

    pub fn light(&self) -> Entity {
        self.light
    }

    pub fn asset_root(&self) -> Entity {
        self.asset_root
    }
}

pub trait UiSceneViewExt {
    fn scene_view(&mut self, asset: impl Into<String>) -> UiBuilder<Entity>;
}

impl UiSceneViewExt for UiBuilder<'_, Entity> {
    fn scene_view(&mut self, asset: impl Into<String>) -> UiBuilder<Entity> {
        let column = self
            .column(|_| {})
            .insert((
                Name::new("Scene View"),
                SpawnSceneView {
                    asset_path: asset.into(),
                },
            ))
            .style() // Needed until UiImage stops depending on background color
            .background_color(Color::WHITE)
            .width(Val::Percent(100.))
            .id();

        self.commands().ui_builder(column)
    }
}
