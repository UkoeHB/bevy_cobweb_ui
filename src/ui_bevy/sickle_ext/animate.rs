
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
    mut query: Query<Option<&mut DynamicStyle>>,
)
{
    // Store the AnimationBundle on the entity.
    let Some(ec) = commands.get_entity(entity) else { return };
    ec.try_insert(CachedAnimationBundle::<T>{ cached: animation.values });

    // Prepare an updated DynamicStyleAttribute.
    let mut controller = DynamicStyleController::default();
    controller.animation = animation.settings;

    let attribute = DynamicStyleAttribute::Animated {
        attribute: AnimatedStyleAttribute::Custom(extract_animation_value::<T>),
        controller,
    };

    // Insert or update the attribute.
    if let Ok(mut existing) = query.get_mut(entity) {
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
    cached: AnimationBundle<T::ValueType>
}

//-------------------------------------------------------------------------------------------------------------------

/// Trait for loadable types that can be animated in response to interactions.
pub trait Animatable: Loadable
{
    /// Specifies the value-type that will be animated on the target (e.g. `f32`, `Color`, etc.).
    type ValueType: Lerp + Copy + Loadable;

    /// Specifies how values should be applied to an entity.
    fn apply(entity: Entity, value: Self::ValueType, world: &mut World);
}

//-------------------------------------------------------------------------------------------------------------------

/// Lodable type for animatable values.
///
/// Automatically includes the [`Interactive`] loadable.
///
/// Note that the `AnimationBundle::idle` field must always be set, which means it is effectively the 'default' value
/// for `T` that will be applied to the entity and override any value you set elsewhere.
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Animate<T: Animatable>
{
    values: AnimationBundle<T::ValueType>,
    settings: StyleAnimation,
}

impl<T: Animatable> ApplyLoadable for Animate<T>
{
    fn apply(self, ec: &mut EntityCommands)
    {
        Interactive.apply(ec);

        let id = ec.id();
        ec.commands().syscall((id, self), update_animation::<T>);
    }
}

//-------------------------------------------------------------------------------------------------------------------
