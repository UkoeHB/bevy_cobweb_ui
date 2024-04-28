
//-------------------------------------------------------------------------------------------------------------------

fn extract_animation_value<T: Animatable>(entity: Entity, state: AnimationState, world: &mut World)
{
    // Get the animation bundle if possible.
    let Some(bundle) = world.get_entity(entity).filter_map(EntityWorldMut::get::<CachedAnimationBundle<T>>) else { return };
    let new_value = bundle.cached.to_value(state);

    // Apply the value to the entity.
    T::apply(entity, new_value, world);
}

//-------------------------------------------------------------------------------------------------------------------

fn update_animation<T: Animatable>(
    In((entity, animation)): In<(Entity, Animate<T>)>,
    mut commands: Commands,
    mut query: Query<(Option<&mut DynamicStyle>, Option<&mut LoadedThemes>)>,
)
{
    // Store the AnimationBundle on the entity.
    let Some(mut ec) = commands.get_entity(entity) else { return };
    ec.try_insert(CachedAnimationBundle::<T>{ cached: animation.values });

    // Prepare an updated DynamicStyleAttribute.
    let mut controller = DynamicStyleController::default();
    controller.animation = animation.settings;

    let attribute = DynamicStyleAttribute::Animated {
        attribute: AnimatedStyleAttribute::Custom(extract_animation_value::<T>),
        controller,
    };

    // Access the entity.
    let Ok((mut maybe_style, mut maybe_themes)) = query.get_mut(entity) else { return };

    // If there is a loaded theme, then add this animation to the theme.
    if let Some(themes) = maybe_themes {
        themes.update(animation.state, attribute);
        commands.add(RefreshLoadedTheme{ entity })
        return;
    }

    // If there is no loaded theme, add this animation directly.
    if let Some(mut existing) = maybe_style {
        existing.merge(vec![attribute]);
    } else {
        let style = DynamicStyle::new(vec![attribute]);
        ec.try_insert(style);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug)]
struct CachedAnimationBundle<T: Animatable>
{
    cached: AnimationBundle<T::Value>
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
pub trait Animatable: Loadable
{
    /// Specifies the value-type that will be animated on the target (e.g. `f32`, `Color`, etc.).
    type Value: Lerp + Loadable;
    /// Specifies the interactivity loadable used by this animatable.
    ///
    /// Can be used to hook target entities up to custom interactivity.
    ///
    /// For example: [`Interactive`] (for `bevy_ui`), [`Interactive2d`] (for 2d UI).
    type Interactive: Default + ApplyLoadable;

    /// Specifies how values should be applied to an entity.
    fn apply(entity: Entity, value: Self::Value, world: &mut World);
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable type for animatable values.
///
/// Note that the `AnimationBundle::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animate<T: Animatable>
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for this animation to become active.
    ///
    /// Only used if this struct is applied to an entity with a loaded theme.
    pub state: Option<Vec<PseudoState>>,
    /// The values that are end-targets for each animation.
    pub values: AnimationBundle<T::Value>,
    /// Settings that control how values are interpolated.
    pub settings: StyleAnimation,
}

impl<T: Animatable> ApplyLoadable for Animate<T>
{
    fn apply(self, ec: &mut EntityCommands)
    {
        T::Interactive::default().apply(ec);

        let id = ec.id();
        ec.commands().syscall((id, self), update_animation::<T>);
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: Respond<T>, same as Animate<T> but only toggles states
// TODO: Themed<T>, just adds a value directly to the theme
