use std::any::{type_name, Any, TypeId};
use std::fmt::Debug;
use std::sync::Arc;

use bevy::prelude::*;
use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

struct CachedStaticAttribute<T: StaticAttribute>
{
    value: T::Value,
}

impl<T: StaticAttribute> CachedStaticAttribute<T>
{
    fn try_resolve(val: &dyn Any) -> Option<&Self>
    {
        val.downcast_ref::<Self>()
    }
    fn try_resolve_mut(val: &mut dyn Any) -> Option<&mut Self>
    {
        val.downcast_mut::<Self>()
    }
}

impl<T: StaticAttribute> Debug for CachedStaticAttribute<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("CachedStaticAttribute<")?;
        f.write_str(type_name::<T>())?;
        f.write_str(">")?;
        Ok(())
    }
}

impl<T: StaticAttribute> Clone for CachedStaticAttribute<T>
{
    fn clone(&self) -> Self
    {
        Self { value: self.value.clone() }
    }
}

impl<T: StaticAttribute> StaticAttributeObject for CachedStaticAttribute<T>
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any
    {
        self
    }

    fn apply(&self, entity: Entity, world: &mut World)
    {
        T::update(entity, world, self.value.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct CachedResponsiveAttribute<T: ResponsiveAttribute>
{
    vals: ResponsiveVals<T::Value>,
}

impl<T: ResponsiveAttribute> CachedResponsiveAttribute<T>
{
    fn try_resolve(val: &dyn Any) -> Option<&Self>
    {
        val.downcast_ref::<Self>()
    }
    fn try_resolve_mut(val: &mut dyn Any) -> Option<&mut Self>
    {
        val.downcast_mut::<Self>()
    }
}

impl<T: ResponsiveAttribute> Debug for CachedResponsiveAttribute<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("CachedResponsiveAttribute<")?;
        f.write_str(type_name::<T>())?;
        f.write_str(">")?;
        Ok(())
    }
}

impl<T: ResponsiveAttribute> Clone for CachedResponsiveAttribute<T>
{
    fn clone(&self) -> Self
    {
        Self { vals: self.vals.clone() }
    }
}

impl<T: ResponsiveAttribute> ResponsiveAttributeObject for CachedResponsiveAttribute<T>
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any
    {
        self
    }

    fn apply(&self, entity: Entity, world: &mut World, state: FluxInteraction)
    {
        let value = T::extract(entity, world, &self.vals, state);
        T::update(entity, world, value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

struct CachedAnimatedAttribute<T: AnimatedAttribute>
{
    vals: AnimatedVals<T::Value>,
}

impl<T: AnimatedAttribute> CachedAnimatedAttribute<T>
{
    fn try_resolve(val: &dyn Any) -> Option<&Self>
    {
        val.downcast_ref::<Self>()
    }
    fn try_resolve_mut(val: &mut dyn Any) -> Option<&mut Self>
    {
        val.downcast_mut::<Self>()
    }
}

impl<T: AnimatedAttribute> Debug for CachedAnimatedAttribute<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        f.write_str("CachedAnimatedAttribute<")?;
        f.write_str(type_name::<T>())?;
        f.write_str(">")?;
        Ok(())
    }
}

impl<T: AnimatedAttribute> Clone for CachedAnimatedAttribute<T>
{
    fn clone(&self) -> Self
    {
        Self { vals: self.vals.clone() }
    }
}

impl<T: AnimatedAttribute> AnimatedAttributeObject for CachedAnimatedAttribute<T>
{
    fn as_any(&self) -> &dyn Any
    {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any
    {
        self
    }

    fn initialize_enter(&mut self, entity: Entity, world: &World)
    {
        // If an enter_ref was pre-specified then we don't overwrite it.
        if self.vals.enter_ref.is_some() {
            return;
        }
        if let Some(current_value) = T::get_value(entity, world) {
            self.vals.enter_ref = Some(current_value);
        }
    }

    fn apply(&self, entity: Entity, world: &mut World, state: AnimationState)
    {
        let value = T::extract(entity, world, &self.vals, &state);
        T::update(entity, world, value);
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
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
            .position(|attr| attr.logical_eq(&attribute))
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
                .and_then(|r| if r == label { None } else { Some(r) });

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
        let pos = self.style.iter().position(|a| a.name() == Some(name))?;
        Some(self.style.remove(pos))
    }

    /// Gets the attribute's info if this theme contains an attribute with the given name.
    fn get_info(&self, name: &str) -> Option<(Option<&[PseudoState]>, &NodeAttribute)>
    {
        if let Some(attr) = self.style.iter().find(|a| a.name() == Some(name)) {
            Some((self.state.as_ref().map(|s| s.as_slice()), attr))
        } else {
            None
        }
    }

    /// Gets an attribute mutably by name.
    fn get_mut(&mut self, name: &str) -> Option<&mut NodeAttribute>
    {
        self.style.iter_mut().find(|a| a.name() == Some(name))
    }

    /// Gets an attribute with state and type id.
    fn get_with<'a>(&'a self, state: Option<&[PseudoState]>, type_id: TypeId) -> Option<&'a NodeAttribute>
    {
        if self.state.as_ref().map(|s| s.as_slice()) != state {
            return None;
        }
        for attr in self.style.iter() {
            if attr.attr_type_id() == type_id {
                return Some(attr);
            }
        }
        None
    }

    /// Gets an attribute mutably with state and type id.
    fn get_with_mut(&mut self, state: Option<&[PseudoState]>, type_id: TypeId) -> Option<&mut NodeAttribute>
    {
        if self.state.as_ref().map(|s| s.as_slice()) != state {
            return None;
        }
        self.style.iter_mut().find(|a| a.attr_type_id() == type_id)
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum CachedAttribute
{
    Static(Arc<dyn StaticAttributeObject>),
    Responsive(Arc<dyn ResponsiveAttributeObject>),
    Animated(Arc<dyn AnimatedAttributeObject>),
}

//-------------------------------------------------------------------------------------------------------------------

/// The attribute type of a [`NodeAttribute`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
    type_id: TypeId,

    respond_to: Option<SmolStr>,

    cached: CachedAttribute,
    settings: Option<AnimationSettings>,
}

impl NodeAttribute
{
    /// Makes a new static attribute.
    pub fn new_static<T: StaticAttribute>(name: Option<SmolStr>, value: T::Value) -> Self
    {
        let type_id = TypeId::of::<T>();
        Self {
            name,
            attribute_type: AttributeType::Static,
            type_id,
            respond_to: None,
            cached: CachedAttribute::Static(Arc::new(CachedStaticAttribute::<T> { value })),
            settings: None,
        }
    }

    /// Makes a new responsive attribute.
    pub fn new_responsive<T: ResponsiveAttribute>(
        name: Option<SmolStr>,
        respond_to: Option<SmolStr>,
        vals: ResponsiveVals<T::Value>,
    ) -> Self
    {
        let type_id = TypeId::of::<T>();
        Self {
            name,
            attribute_type: AttributeType::Responsive,
            type_id,
            respond_to,
            cached: CachedAttribute::Responsive(Arc::new(CachedResponsiveAttribute::<T> { vals })),
            settings: None,
        }
    }

    /// Makes a new animated attribute.
    pub fn new_animated<T: AnimatedAttribute>(
        name: Option<SmolStr>,
        respond_to: Option<SmolStr>,
        vals: AnimatedVals<T::Value>,
        settings: AnimationSettings,
    ) -> Self
    {
        let type_id = TypeId::of::<T>();
        Self {
            name,
            attribute_type: AttributeType::Animated,
            type_id,
            respond_to,
            cached: CachedAttribute::Animated(Arc::new(CachedAnimatedAttribute::<T> { vals })),
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

    pub fn attr_type_id(&self) -> TypeId
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
    pub fn static_val<T: StaticAttribute>(&self) -> Option<&T::Value>
    {
        match &self.cached {
            CachedAttribute::Static(attr) => {
                CachedStaticAttribute::<T>::try_resolve(attr.as_any()).map(|c| &c.value)
            }
            _ => None,
        }
    }

    /// Gets the inner responsive reference values.
    ///
    /// Returns `None` if self is not `AttributeType::Responsive` or the requested type doesn't match.
    pub fn responsive_vals<T: ResponsiveAttribute>(&self) -> Option<&ResponsiveVals<T::Value>>
    {
        match &self.cached {
            CachedAttribute::Responsive(attr) => {
                CachedResponsiveAttribute::<T>::try_resolve(attr.as_any()).map(|c| &c.vals)
            }
            _ => None,
        }
    }

    /// Gets the inner animated reference values.
    ///
    /// Returns `None` if self is not `AttributeType::Animated` or the requested type doesn't match.
    pub fn animated_vals<T: AnimatedAttribute>(&self) -> Option<&AnimatedVals<T::Value>>
    {
        match &self.cached {
            CachedAttribute::Animated(attr) => {
                CachedAnimatedAttribute::<T>::try_resolve(attr.as_any()).map(|c| &c.vals)
            }
            _ => None,
        }
    }

    /// Gets `AnimationSettings.
    ///
    /// Returns `None` if self is not `AttributeType::Animated`.
    pub fn animation_settings(&self) -> Option<&AnimationSettings>
    {
        self.settings.as_ref()
    }

    /// Gets the inner static reference value.
    ///
    /// Returns `None` if self is not `AttributeType::Static` or the requested type doesn't match.
    pub fn static_val_mut<T: StaticAttribute>(&mut self) -> Option<&mut T::Value>
    {
        match &mut self.cached {
            CachedAttribute::Static(attr) => {
                let attr = dyn_clone::arc_make_mut(attr);
                CachedStaticAttribute::<T>::try_resolve_mut(attr.as_any_mut()).map(|c| &mut c.value)
            }
            _ => None,
        }
    }

    /// Gets the inner responsive reference values.
    ///
    /// Returns `None` if self is not `AttributeType::Responsive` or the requested type doesn't match.
    pub fn responsive_vals_mut<T: ResponsiveAttribute>(&mut self) -> Option<&mut ResponsiveVals<T::Value>>
    {
        match &mut self.cached {
            CachedAttribute::Responsive(attr) => {
                let attr = dyn_clone::arc_make_mut(attr);
                CachedResponsiveAttribute::<T>::try_resolve_mut(attr.as_any_mut()).map(|c| &mut c.vals)
            }
            _ => None,
        }
    }

    /// Gets the inner animated reference values.
    ///
    /// Returns `None` if self is not `AttributeType::Animated` or the requested type doesn't match.
    pub fn animated_vals_mut<T: AnimatedAttribute>(&mut self) -> Option<&mut AnimatedVals<T::Value>>
    {
        match &mut self.cached {
            CachedAttribute::Animated(attr) => {
                let attr = dyn_clone::arc_make_mut(attr);
                CachedAnimatedAttribute::<T>::try_resolve_mut(attr.as_any_mut()).map(|c| &mut c.vals)
            }
            _ => None,
        }
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
        match self.cached.clone() {
            CachedAttribute::Static(attr) => DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
                CustomStaticStyleAttribute::new(self.type_id, attr),
            )),
            CachedAttribute::Responsive(attr) => {
                DynamicStyleAttribute::Responsive(ResponsiveStyleAttribute::new(self.type_id, attr))
            }
            CachedAttribute::Animated(attr) => DynamicStyleAttribute::Animated {
                attribute: AnimatedStyleAttribute::new(self.type_id, attr),
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
        self.respond_to == other.respond_to && self.type_id == other.type_id
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
                if state.as_ref().map(|s| s.as_slice()) != prev_state || !attribute.logical_eq(prev) {
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
    pub fn get_info(&self, name: impl AsRef<str>) -> Option<(Option<&[PseudoState]>, &NodeAttribute)>
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

    /// Gets an attribute's static value immutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn static_val<T: StaticAttribute>(&self, name: impl AsRef<str>) -> Option<&T::Value>
    {
        self.get(name).and_then(|a| a.static_val::<T>())
    }

    /// Gets an attribute's responsive values immutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn responsive_vals<T: ResponsiveAttribute>(
        &self,
        name: impl AsRef<str>,
    ) -> Option<&ResponsiveVals<T::Value>>
    {
        self.get(name).and_then(|a| a.responsive_vals::<T>())
    }

    /// Gets an attribute's animated values immutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn animated_vals<T: AnimatedAttribute>(&self, name: impl AsRef<str>) -> Option<&AnimatedVals<T::Value>>
    {
        self.get(name).and_then(|a| a.animated_vals::<T>())
    }

    /// Gets an attribute's animation settings immutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn animation_settings(&self, name: impl AsRef<str>) -> Option<&AnimationSettings>
    {
        self.get(name).and_then(|a| a.animation_settings())
    }

    /// Gets an attribute's static value mutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn static_val_mut<T: StaticAttribute>(&mut self, name: impl AsRef<str>) -> Option<&mut T::Value>
    {
        self.get_mut(name).and_then(|a| a.static_val_mut::<T>())
    }

    /// Gets an attribute's responsive values mutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn responsive_vals_mut<T: ResponsiveAttribute>(
        &mut self,
        name: impl AsRef<str>,
    ) -> Option<&mut ResponsiveVals<T::Value>>
    {
        self.get_mut(name)
            .and_then(|a| a.responsive_vals_mut::<T>())
    }

    /// Gets an attribute's animated values mutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn animated_vals_mut<T: AnimatedAttribute>(
        &mut self,
        name: impl AsRef<str>,
    ) -> Option<&mut AnimatedVals<T::Value>>
    {
        self.get_mut(name).and_then(|a| a.animated_vals_mut::<T>())
    }

    /// Gets an attribute's animation settings mutably by name.
    ///
    /// If multiple attributes have the same name, this will return the one that was inserted first.
    pub fn animation_settings_mut(&mut self, name: impl AsRef<str>) -> Option<&mut AnimationSettings>
    {
        self.get_mut(name).and_then(|a| a.animation_settings_mut())
    }

    /// Edits an attribute's static value mutably by name.
    ///
    /// The callback will not be invoked if the requested attribute is not found or is not `AttributeType::Static`.
    pub fn edit_static_val<T: StaticAttribute>(
        &mut self,
        name: impl AsRef<str>,
        callback: impl FnOnce(&mut T::Value),
    ) -> bool
    {
        let Some(s) = self.static_val_mut::<T>(name) else { return false };
        (callback)(s);
        true
    }

    /// Edits an attribute's responsive values mutably by name.
    ///
    /// The callback will not be invoked if the requested attribute is not found or is not
    /// `AttributeType::Responsive`.
    pub fn edit_responsive_vals<T: ResponsiveAttribute>(
        &mut self,
        name: impl AsRef<str>,
        callback: impl FnOnce(&mut ResponsiveVals<T::Value>),
    ) -> bool
    {
        let Some(s) = self.responsive_vals_mut::<T>(name) else { return false };
        (callback)(s);
        true
    }

    /// Edits an attribute's animated values mutably by name.
    ///
    /// The callback will not be invoked if the requested attribute is not found or is not
    /// `AttributeType::Animated`.
    pub fn edit_animated_vals<T: AnimatedAttribute>(
        &mut self,
        name: impl AsRef<str>,
        callback: impl FnOnce(&mut AnimatedVals<T::Value>),
    ) -> bool
    {
        let Some(s) = self.animated_vals_mut::<T>(name) else { return false };
        (callback)(s);
        true
    }

    /// Edits an attribute's animation settings mutably by name.
    ///
    /// The callback will not be invoked if the requested attribute is not found or is not
    /// `AttributeType::Animated`.
    pub fn edit_animation_settings(
        &mut self,
        name: impl AsRef<str>,
        callback: impl FnOnce(&mut AnimationSettings),
    ) -> bool
    {
        let Some(s) = self.animation_settings_mut(name) else { return false };
        (callback)(s);
        true
    }

    /// Gets an attribute with state and type id.
    pub fn get_with(&self, state: Option<&[PseudoState]>, type_id: TypeId) -> Option<&NodeAttribute>
    {
        self.themes
            .iter()
            .filter_map(|t| t.get_with(state, type_id))
            .next()
    }

    /// Gets an attribute mutably with state and type id.
    pub fn get_with_mut(&mut self, state: Option<&[PseudoState]>, type_id: TypeId) -> Option<&mut NodeAttribute>
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
