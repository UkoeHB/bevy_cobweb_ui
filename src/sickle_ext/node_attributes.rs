use std::any::{type_name, Any, TypeId};

use bevy::prelude::*;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

fn extract_static_value<T: StaticAttribute>(entity: Entity, world: &mut World, ref_val: &dyn Any)
{
    let Some(ref_val) = ref_val.downcast_ref::<T::Value>() else {
        tracing::error!("failed downcasting static attribute ref value for extraction of {} (this is a bug)",
            type_name::<T>());
        return;
    };
    T::update(entity, world, ref_val.clone());
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_responsive_value<T: ResponsiveAttribute>(
    entity: Entity,
    state: FluxInteraction,
    world: &mut World,
    ref_vals: &dyn Any,
)
{
    let Some(ref_vals) = ref_vals.downcast_ref::<ResponsiveVals<T::Value>>() else {
        tracing::error!("failed downcasting responsive attribute ref value for extraction of {} (this is a bug)",
            type_name::<T>());
        return;
    };

    let value = T::extract(entity, world, ref_vals, state);
    T::update(entity, world, value);
}

//-------------------------------------------------------------------------------------------------------------------

fn extract_animation_value<T: AnimatedAttribute>(
    entity: Entity,
    state: AnimationState,
    world: &mut World,
    ref_vals: &dyn Any,
)
{
    let Some(ref_vals) = ref_vals.downcast_ref::<AnimatedVals<T::Value>>() else {
        tracing::error!("failed downcasting animated attribute ref value for extraction of {} (this is a bug)",
            type_name::<T>());
        return;
    };

    let value = T::extract(entity, world, ref_vals, &state);
    T::update(entity, world, value);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub(super) struct PseudoTheme
{
    state: Option<SmallVec<[PseudoState; 3]>>,
    style: SmallVec<[NodeAttribute; 3]>,
}

impl PseudoTheme
{
    fn new(state: Option<SmallVec<[PseudoState; 3]>>, attribute: NodeAttribute) -> Self
    {
        let mut style = SmallVec::new();
        style.push(attribute);
        Self { state, style }
    }

    fn matches(&self, state: &Option<SmallVec<[PseudoState; 3]>>) -> bool
    {
        self.state == *state
    }

    fn set_attribute(&mut self, attribute: NodeAttribute) -> Option<NodeAttribute>
    {
        // Merge attribute with existing list.
        if let Some(index) = self
            .style
            .iter()
            .position(|(_, attr)| attr.logical_eq(&attribute))
        {
            let prev = std::mem::replace(&mut self.style[index], attribute);
            Some(prev)
        } else {
            self.style.push(attribute);
            None
        }
    }

    pub(super) fn is_subset(&self, node_states: &[PseudoState]) -> Option<usize>
    {
        match &self.state {
            // Only consider pseudo themes that are specific to an inclusive subset of the themed element's pseudo
            // states. A theme for [Checked, Disabled] will apply to elements with [Checked, Disabled,
            // FirstChild], but will not apply to elements with [Checked] (because the theme targets
            // more specific elements) or [Checked, FirstChild] (because they are disjoint).
            Some(theme_states) => match theme_states.iter().all(|state| node_states.contains(state)) {
                true => Some(theme_states.len()),
                false => None,
            },
            None => Some(0),
        }
    }

    /// Adds all attributes to the style builder.
    pub(super) fn build(&self, label: &SmolStr, style_builder: &mut StyleBuilder)
    {
        for attribute in self.style.iter() {
            // Clear source if it points to self.
            // TODO: why is this necessary? Something weird in sickle_ui means if the root node sources itself
            // then child nodes' attributes won't properly respond to interactions on the root.
            let source = attribute
                .responds_to()
                .map(|r| if r == label { None } else { r });

            // Set the placement (the placement of DynamicStyle, which is the 'interaction source' for non-static
            // attributes).
            if let Some(source) = source {
                style_builder.switch_placement_with(source.clone());
            } else {
                style_builder.reset_placement();
            }

            // Set the target to self.
            style_builder.switch_target_with(label.clone());

            // Insert attribute.
            style_builder.add(attribute.dynamic_style_attribute());
        }
    }

    /// Tries to remove an attribute by name.
    fn try_remove(&mut self, name: &str) -> Option<NodeAttribute>
    {
        let pos = self.iter().find(|a| a.name() == Some(name))?;
        self.style.remove(pos)
    }

    /// Gets the attribute's info if this theme contains an attribute with the given name.
    fn get_info(&self, name: &str) -> Option<(&[PseudoState], &NodeAttribute)>
    {
        if let Some(attr) = self.iter().find(|a| a.name() == Some(name)) {
            (self.state.as_ref().map(|s| &s), attr)
        } else {
            None
        }
    }

    /// Gets an attribute mutably by name.
    fn get_mut(&mut self, name: &str) -> Option<&mut NodeAttribute>
    {
        let name = name.as_ref();
        self.style.iter_mut().find(|a| a.name() == Some(name))
    }

    /// Gets an attribute with state and type id.
    fn get_with(&self, state: Option<&[PseudoState; 3]>, type_id: TypeId) -> Option<&NodeAttribute>
    {
        if self.states.as_ref().map(|s| &s) != state {
            return None;
        }
        self.style.iter().find(|a| a.type_id() == type_id)
    }

    /// Gets an attribute mutably with state and type id.
    fn get_with_mut(&mut self, state: Option<&[PseudoState; 3]>, type_id: TypeId) -> Option<&mut NodeAttribute>
    {
        if self.states.as_ref().map(|s| &s) != state {
            return None;
        }
        self.style.iter_mut().find(|a| a.type_id() == type_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

enum AttributeExtractor
{
    Static(fn(Entity, &mut World, &dyn Any)),
    Responsive(fn(Entity, FluxInteraction, &mut World, &dyn Any)),
    Animated(fn(Entity, AnimationState, &mut World, &dyn Any)),
}

//-------------------------------------------------------------------------------------------------------------------

/// The attribute type of a [`NodeAttribute`].
#[derive(Debug)]
pub enum AttributeType
{
    /// Static attributes are single values that are applied when an entity's pseudo states match the attribute.
    ///
    /// See [`Static`].
    Static,
    /// Responsive attributes are values that are applied when interactions on an entity change
    /// (hover, press, idle).
    ///
    /// See [`Responsive`].
    Responsive,
    /// Animated attributes are values that are animated when interactions on an entity change
    /// (enter state, hover, press, idle, etc.).
    ///
    /// See [`Animated`].
    Animated,
}

//-------------------------------------------------------------------------------------------------------------------

/// An attribute that can be used to control components on a UI entity in response to entity [`PseudoState`]
/// changes or interactions.
///
/// See the [`Static`], [`Responsive`], and [`Animated`] loadables.
#[derive(Debug)]
pub struct NodeAttribute
{
    name: Option<SmolStr>,
    attribute_type: AttributeType,
    extractor: AttributeExtractor,
    type_id: TypeId,
    respond_to: Option<SmolStr>,
    /// Reference value for this attribute.
    reference: Arc<dyn Any + Send + Sync + 'static>,
    settings: Option<AnimationSettings>,
}

impl NodeAttribute
{
    /// Makes a new static attribute.
    pub fn new_static<T: StaticAttribute>(name: Option<SmolStr>, reference: T::Value) -> Self
    {
        let type_id = TypeId::of::<T>();
        Self {
            name,
            attribute_type: AttributeType::Static,
            extractor: AttributeExtractor::Static(extract_static_value::<T>),
            type_id,
            respond_to: None,
            reference: Arc::new(reference),
            settings: None,
        }
    }

    /// Makes a new responsive attribute.
    pub fn new_responsive<T: ResponsiveAttribute>(
        name: Option<SmolStr>,
        respond_to: Option<SmolStr>,
        reference: ResponsiveVals<T::Value>,
    ) -> Self
    {
        let type_id = TypeId::of::<T>();
        Self {
            name,
            attribute_type: AttributeType::Responsive,
            extractor: AttributeExtractor::Responsive(extract_responsive_value::<T>),
            type_id,
            respond_to,
            reference: Arc::new(reference),
            settings: None,
        }
    }

    /// Makes a new animated attribute.
    pub fn new_animated<T: AnimatedAttribute>(
        name: Option<SmolStr>,
        respond_to: Option<SmolStr>,
        reference: AnimatedVals<T::Value>,
        settings: AnimationSettings,
    ) -> Self
    {
        let type_id = TypeId::of::<T>();
        Self {
            name,
            attribute_type: AttributeType::Animated,
            extractor: AttributeExtractor::Animated(extract_animation_value::<T>),
            type_id,
            respond_to,
            reference: Arc::new(reference),
            settings: Some(settings),
        }
    }

    /// Gets the attribute name (if any).
    pub fn name(&self) -> Option<&str>
    {
        self.name.as_ref().map(|s| s.as_str())
    }

    pub fn attribute_type(&self) -> AttributeType
    {
        self.attribute_type
    }

    pub fn type_id(&self) -> TypeId
    {
        self.type_id
    }

    /// Gets the control group member this attribute responds to (if any).
    pub fn responds_to(&self) -> Option<&SmolStr>
    {
        self.respond_to.as_ref()
    }

    /// Gets the inner static reference value.
    ///
    /// Returns `None` if self is not `AttributeType::Static` or the requested type doesn't match.
    pub fn static_val<T: StaticAttribute>(&mut self) -> Option<&mut T::Value>
    {
        let val = self.reference.make_mut();
        val.downcast_mut::<T::Value>()
    }

    /// Gets the inner responsive reference values.
    ///
    /// Returns `None` if self is not `AttributeType::Responsive` or the requested type doesn't match.
    pub fn responsive_vals<T: ResponsiveAttribute>(&mut self) -> Option<&mut ResponsiveVals<T::Value>>
    {
        let val = self.reference.make_mut();
        val.downcast_mut::<ResponsiveVals<T::Value>>()
    }

    /// Gets the inner animated reference values.
    ///
    /// Returns `None` if self is not `AttributeType::Animated` or the requested type doesn't match.
    pub fn animated_vals<T: AnimatedAttribute>(&mut self) -> Option<&mut AnimatedVals<T::Value>>
    {
        let val = self.reference.make_mut();
        val.downcast_mut::<AnimatedVals<T::Value>>()
    }

    /// Gets `AnimationSettings.
    ///
    /// Returns `None` if self is not `AttributeType::Animated`.
    pub fn animation_settings(&self) -> Option<&AnimationSettings>
    {
        self.settings.as_ref()
    }

    /// Gets a mutable reference to `AnimationSettings.
    ///
    /// Returns `None` if self is not `AttributeType::Animated`.
    pub fn animation_settings_mut(&mut self) -> Option<&mut AnimationSettings>
    {
        self.settings.as_mut()
    }

    /// Makes a new `DynamicStyleAttribute` from self.
    pub fn dynamic_style_attribute(&self) -> DynamicStyleAttribute
    {
        match self.extractor {
            AttributeExtractor::Static(callback) => DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
                CustomStaticStyleAttribute::new(self.type_id, self.reference.clone(), callback),
            )),
            AttributeExtractor::Responsive(callback) => DynamicStyleAttribute::Responsive(
                ResponsiveStyleAttribute::new(self.type_id, self.reference.clone(), callback),
            ),
            AttributeExtractor::Animated(callback) => DynamicStyleAttribute::Animated {
                attribute: AnimatedStyleAttribute::new(self.type_id, self.reference.clone(), callback),
                controller: DynamicStyleController::new(
                    self.settings.clone().unwrap_or_default(),
                    AnimationState::default(),
                ),
            },
        }
    }
}

impl LogicalEq for NodeAttribute
{
    fn logical_eq(&self, other: &Self) -> bool
    {
        self.respond_to == other.respond_to && self.type_id.logical_eq(&other.type_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that stores [`NodeAttribute`] values for an entity.
///
/// **Warning**: `NodeAttributes` may get reset if the entity loads a COB scene node and the node gets
/// hot-reloaded. Attributes that are inserted or modified manually should be re-inserted/modified if that happens
/// (e.g. use [`UiBuilderReactExt::update`] to auto-insert on hot reload).
#[derive(Component, Debug, Default)]
pub struct NodeAttributes
{
    themes: SmallVec<[PseudoTheme; 1]>,
}

impl NodeAttributes
{
    /// Inserts an attribute.
    ///
    /// Returns the old attribute if one existed.
    ///
    /// Logs a warning if the attribute has the same name as an existing attribute.
    pub fn insert(
        &mut self,
        mut state: Option<SmallVec<[PseudoState; 3]>>,
        attribute: NodeAttribute,
    ) -> Option<NodeAttribute>
    {
        if let Some(states) = state.as_deref_mut() {
            states.sort_unstable();
        }

        if let Some(name) = attribute.name() {
            if let Some((prev_state, prev)) = self.get_info(name) {
                if state.map(|s| &s) != prev_state || !attribute.logical_eq(prev) {
                    tracing::warn!("adding node attribute to entity that already has an attribute with the same name ({}); \
                        only the older attribute will be accessible by NodeAttributes::get, etc.", name);
                }
            }
        }

        match self.themes.iter_mut().find(|t| t.matches(&state)) {
            Some(theme) => theme.set_attribute(attribute),
            None => {
                self.themes.push(PseudoTheme::new(state, attribute));
                None
            }
        }
    }

    /// Removes an attribute by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<NodeAttribute>
    {
        let name = name.as_ref();
        self.themes
            .iter_mut()
            .filter_map(|t| t.try_remove(name))
            .next()
    }

    /// Gets an attribute by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&NodeAttribute>
    {
        self.get_info(name).map(|(_, a)| a)
    }

    /// Gets an attribute and its state by name.
    ///
    /// If multiple attributes have the same name, this will return the state of the one that was inserted first.
    pub fn get_info(&self, name: impl AsRef<str>) -> Option<(&[PseudoState], &NodeAttribute)>
    {
        let name = name.as_ref();
        self.themes.iter().filter_map(|t| t.get_info(name)).next()
    }

    /// Gets an attribute mutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<&mut NodeAttribute>
    {
        let name = name.as_ref();
        self.themes
            .iter_mut()
            .filter_map(|t| t.get_mut(name))
            .next()
    }

    /// Gets an attribute with state and type id.
    pub fn get_with(&self, state: Option<&[PseudoState; 3]>, type_id: TypeId) -> Option<&NodeAttribute>
    {
        self.themes
            .iter()
            .filter_map(|t| t.get_with(state, type_id))
            .next()
    }

    /// Gets an attribute mutably with state and type id.
    pub fn get_with_mut(&mut self, state: Option<&[PseudoState; 3]>, type_id: TypeId)
        -> Option<&mut NodeAttribute>
    {
        self.themes
            .iter_mut()
            .filter_map(|t| t.get_with_mut(state, type_id))
            .next()
    }

    pub(super) fn iter_themes(&self) -> impl Iterator<Item = &PseudoTheme> + '_
    {
        self.themes.iter()
    }
}

//-------------------------------------------------------------------------------------------------------------------
