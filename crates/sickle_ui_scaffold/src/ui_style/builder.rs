use bevy::prelude::*;
use smol_str::SmolStr;

use crate::{prelude::FluxInteraction, theme::prelude::*};

use super::{
    attribute::{
        CustomAnimatedStyleAttribute, CustomInteractiveStyleAttribute, CustomStaticStyleAttribute,
    },
    generated::*,
    LogicalEq,
};

pub struct InteractiveStyleBuilder<'a> {
    pub style_builder: &'a mut StyleBuilder,
}

impl<'a> InteractiveStyleBuilder<'a> {
    pub fn custom(
        &mut self,
        callback: impl Fn(Entity, FluxInteraction, &mut World) + Send + Sync + 'static,
    ) -> &mut Self {
        self.style_builder.add(DynamicStyleAttribute::Interactive(
            InteractiveStyleAttribute::Custom(CustomInteractiveStyleAttribute::new(callback)),
        ));

        self
    }
}

pub struct AnimatedStyleBuilder<'a> {
    pub style_builder: &'a mut StyleBuilder,
}

impl AnimatedStyleBuilder<'_> {
    pub fn add_and_extract_animation(
        &mut self,
        attribute: DynamicStyleAttribute,
    ) -> &mut AnimationSettings {
        let index = self.style_builder.add(attribute.clone());

        let DynamicStyleAttribute::Animated {
            controller: DynamicStyleController {
                ref mut animation, ..
            },
            ..
        } = self.style_builder.attributes[index].attribute
        else {
            unreachable!();
        };

        animation
    }

    pub fn custom(
        &mut self,
        callback: impl Fn(Entity, AnimationState, &mut World) + Send + Sync + 'static,
    ) -> &mut AnimationSettings {
        let attribute = DynamicStyleAttribute::Animated {
            attribute: AnimatedStyleAttribute::Custom(CustomAnimatedStyleAttribute::new(callback)),
            controller: DynamicStyleController::default(),
        };

        self.add_and_extract_animation(attribute)
    }
}

#[derive(Clone, Debug)]
pub struct ContextStyleAttributeConfig {
    placement: Option<SmolStr>,
    target: Option<SmolStr>,
    attribute: DynamicStyleAttribute,
}

impl LogicalEq for ContextStyleAttributeConfig {
    fn logical_eq(&self, other: &Self) -> bool {
        self.placement == other.placement
            && self.target == other.target
            && self.attribute.logical_eq(&other.attribute)
    }
}

#[derive(Default, Debug)]
pub struct StyleBuilder {
    placement: Option<SmolStr>,
    target: Option<SmolStr>,
    attributes: Vec<ContextStyleAttributeConfig>,
}

impl From<StyleBuilder> for DynamicStyle {
    fn from(value: StyleBuilder) -> Self {
        value.attributes.iter().for_each(|attr| {
            if attr.placement.is_some() || attr.target.is_some() {
                warn!(
                    "StyleBuilder with context-bound attributes converted without context! \
                    Some attributes discarded! \
                    This can be the result of using `PseudoTheme::build()` and calling \
                    `style_builder.switch_placement(CONTEXT)` in the callback, which is not supported.",                    
                );
            }
        });

        DynamicStyle::new(
            value
                .attributes
                .iter()
                .filter(|attr| attr.placement.is_none() || attr.target.is_none())
                .map(|attr| attr.attribute.clone())
                .collect(),
        )
    }
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_capacity(num_attributes: usize) -> Self {
        Self {
            placement: None,
            target: None,
            attributes: Vec::with_capacity(num_attributes),
        }
    }

    pub fn add(&mut self, attribute: DynamicStyleAttribute) -> usize {
        let index = self.attributes.iter().position(|csac| {
            csac.placement == self.placement
                && csac.target == self.target
                && csac.attribute.logical_eq(&attribute)
        });

        match index {
            Some(index) => {
                warn!(
                    "Overwriting {:?} with {:?}",
                    self.attributes[index], attribute
                );
                self.attributes[index].attribute = attribute;

                index
            }
            None => {
                self.attributes.push(ContextStyleAttributeConfig {
                    placement: self.placement.clone(),
                    target: self.target.clone(),
                    attribute,
                });
                self.attributes.len() - 1
            }
        }
    }

    pub fn custom(
        &mut self,
        callback: impl Fn(Entity, &mut World) + Send + Sync + 'static,
    ) -> &mut Self {
        self.add(DynamicStyleAttribute::Static(StaticStyleAttribute::Custom(
            CustomStaticStyleAttribute::new(callback),
        )));

        self
    }

    pub fn interactive(&mut self) -> InteractiveStyleBuilder {
        InteractiveStyleBuilder {
            style_builder: self,
        }
    }

    pub fn animated(&mut self) -> AnimatedStyleBuilder {
        AnimatedStyleBuilder {
            style_builder: self,
        }
    }

    /// Switch context of styling by changing the placement of the DynamicStyle and the target of interaction styling.
    /// Values are mapped to the UiContext of the themed component. `None` placement refers to the main entity.
    /// `None` target refers to the current placement entity.
    pub fn switch_context(
        &mut self,
        placement: impl Into<Option<&'static str>>,
        target: impl Into<Option<&'static str>>,
    ) -> &mut Self {
        self.placement = placement.into().map(|p| SmolStr::new_static(p));
        self.target = target.into().map(|p| SmolStr::new_static(p));

        self
    }

    /// Resets both placement and target to the main entity.
    pub fn reset_context(&mut self) -> &mut Self {
        self.placement = None;
        self.target = None;
        self
    }

    /// Revert StyleBuilder to place style on the main entity.
    pub fn reset_placement(&mut self) -> &mut Self {
        self.placement = None;
        self
    }

    /// Revert StyleBuilder to target the main entity for styling.
    pub fn reset_target(&mut self) -> &mut Self {
        self.target = None;
        self
    }

    /// All subsequent calls to the StyleBuilder will add styling to the selected sub-component.
    /// NOTE: The DynamicStyle will be placed on the selected sub-component and interactions will be
    /// detected on it. This allows styling sub-components directly. It also allows detecting interactions
    /// on a sub-component and proxying it to the main entity or other sub-components.
    pub fn switch_placement(&mut self, placement: &'static str) -> &mut Self {
        self.switch_placement_with(SmolStr::new_static(placement))
    }

    /// See [`Self::switch_placement`].
    pub fn switch_placement_with(&mut self, placement: SmolStr) -> &mut Self {
        self.placement = Some(placement);
        self
    }

    /// All subsequent calls to the StyleBuilder will target styling to the selected sub-component.
    /// NOTE: The DynamicStyle will still be set on the main entity and interactions will be
    /// detected on it. This allows styling sub-components by proxy from the current placement.
    pub fn switch_target(&mut self, target: &'static str) -> &mut Self {
        self.switch_target_with(SmolStr::new_static(target))
    }

    /// See [`Self::switch_target`].
    pub fn switch_target_with(&mut self, target: SmolStr) -> &mut Self {
        self.target = Some(target);
        self
    }

    pub fn convert_with(mut self, context: &impl UiContext) -> Vec<(Option<Entity>, DynamicStyle)> {
        self.attributes
            .sort_unstable_by(|a, b| a.placement.cmp(&b.placement));
        let count = self
            .attributes
            .chunk_by(|a, b| a.placement == b.placement)
            .count();

        let mut result: Vec<(Option<Entity>, DynamicStyle)> = Vec::with_capacity(count);
        result.extend(self.convert_to_iter(context));
        result
    }

    pub fn convert_to_iter<'a>(
        &'a mut self,
        context: &'a impl UiContext,
    ) -> impl Iterator<Item = (Option<Entity>, DynamicStyle)> + 'a {
        self.convert_to_iter_with_buffers(context, Vec::default)
    }

    /// Converts to `DynamicStyles` using a buffer source for the `DynamicStyle` inner attribute buffer.
    ///
    /// This method is potentially non-allocating if the returned buffers have enough capacity and all attributes
    /// can be cloned without allocating.
    pub fn convert_to_iter_with_buffers<'a>(
        &'a mut self,
        context: &'a impl UiContext,
        buffer_source: impl FnMut() -> Vec<ContextStyleAttribute> + 'a,
    ) -> impl Iterator<Item = (Option<Entity>, DynamicStyle)> + 'a {
        self.attributes
            .sort_unstable_by(|a, b| a.placement.cmp(&b.placement));

        self.attributes
            .chunk_by(|a, b| a.placement == b.placement)
            .scan(0, |index, placement_chunk| {
                let start = *index;
                let end = start + placement_chunk.len();
                let placement = placement_chunk[0].placement.clone();
                *index = end;
                Some((start, end, placement))
            })
            .filter_map(|(start, end, placement)| {
                let mut placement_entity: Option<Entity> = None;

                if let Some(target_placement) = placement {
                    let target_entity = match context.get(target_placement.as_str()) {
                        Ok(entity) => entity,
                        Err(msg) => {
                            warn!("{}", msg);
                            return None;
                        }
                    };

                    if target_entity == Entity::PLACEHOLDER {
                        #[cfg(not(feature = "disable-ui-context-placeholder-warn"))]
                        warn!("Entity::PLACEHOLDER returned for placement target!");

                        return None;
                    } else {
                        placement_entity = Some(target_entity);
                    }
                }

                Some((start, end, placement_entity))
            })
            .scan(
                buffer_source,
                |buffer_source, (start, end, placement_entity)| {
                    let mut attributes = (buffer_source)();
                    attributes.clear();
                    Some((
                        placement_entity,
                        DynamicStyle::copy_from(self.attributes[start..end].iter().fold(
                            attributes,
                            |acc: Vec<ContextStyleAttribute>, csac| {
                                StyleBuilder::fold_context_style_attributes(acc, csac, context)
                            },
                        )),
                    ))
                },
            )
    }

    /// Clears the builder without deallocating.
    pub fn clear(&mut self) {
        self.target = None;
        self.placement = None;
        self.attributes.clear();
    }

    fn fold_context_style_attributes(
        mut acc: Vec<ContextStyleAttribute>,
        csac: &ContextStyleAttributeConfig,
        context: &impl UiContext,
    ) -> Vec<ContextStyleAttribute> {
        let new_entry: ContextStyleAttribute = match &csac.target {
            Some(target) => match context.get(target) {
                Ok(target_entity) => match target_entity == Entity::PLACEHOLDER {
                    true => {
                        #[cfg(not(feature = "disable-ui-context-placeholder-warn"))]
                        warn!("Entity::PLACEHOLDER returned for styling target!");

                        return acc;
                    }
                    false => {
                        ContextStyleAttribute::new(target_entity, csac.attribute.clone()).into()
                    }
                },
                Err(msg) => {
                    warn!("{}", msg);
                    return acc;
                }
            },
            None => ContextStyleAttribute::new(None, csac.attribute.clone()).into(),
        };

        if !acc
            .iter()
            .any(|csa: &ContextStyleAttribute| csa.logical_eq(&new_entry))
        {
            acc.push(new_entry);
        } else {
            warn!("Style overwritten for {:?}", new_entry);
            // Safe unwrap: checked in if above
            let index = acc
                .iter()
                .position(|csa| csa.logical_eq(&new_entry))
                .unwrap();
            acc[index] = new_entry;
        }

        acc
    }
}
