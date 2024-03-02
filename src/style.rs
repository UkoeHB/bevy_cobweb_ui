//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::any::TypeId;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------

/// Trait representing [`UiInstructions`](UiInstruction) that can be loaded with [`StyleSheet`].
///
/// Styles must be inserted as [`ReactComponents`](bevy_cobweb::prelude::ReactComponent) to node entities, and their
/// [`UiInstruction`] implementation should include reaction logic for handling mutations to the style component
/// caused by stylesheet loading.
///
/// Note that it is typically safe to manually mutate `CobwebStyle` components on node entities, because stylesheet
/// loading is only used for initialization in production settings.
/// If you *do* reload a stylesheet (e.g. during development), then existing dynamic styles that were changed will be
/// overwritten.
pub trait CobwebStyle: ReactComponent + Reflect + Default + Serialize + for<'de> Deserialize<'de>
{
    /// Applies a style to a node.
    ///
    /// Implementing this enables styles to be used as UI instructions. The [`UiInstruction`] implmentation for styles
    /// invokes this method. The UI instruction then inserts `Self` as a `ReactComponent` on the node. You should not
    /// insert it manually.
    fn apply_style(&self, rc: &mut ReactCommands, node: Entity);
}

impl<T: CobwebStyle> UiInstruction for T
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        Self::apply_style(&self, rc, node);
        rc.insert(node, self);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A [`UiInstruction`] for loading a [`CobwebStyle`] from the [`StyleSheet`].
pub struct StyleLoader<T: CobwebStyle>
{
    style_ref: StyleRef,
    p: PhantomData<T>,
}

impl<T: CobwebStyle> UiInstruction for StyleLoader<T>
{
    fn apply(self, rc: &mut ReactCommands, node: Entity)
    {
        // Default-initialize the instruction and apply it.
        T::default().apply(rc, node);

        // Update the style `T` on this node when the stylesheet is updated.
        let mut initialized = false;
        let token = rc.on_revokable((entity_event::<FinishNode>(node), resource_mutation::<StyleSheet>()),
            move
            |
                mut rc    : ReactCommands,
                types     : Res<AppTypeRegistry>,
                styles    : ReactRes<StyleSheet>,
                mut nodes : Query<&mut React<T>>
            |
            {
                // Look up the type's short name.
                let type_registry = types.read();
                let Some(type_info) = type_registry.get(TypeId::of::<T>())
                else { tracing::error!("type registry info missing for {:?}", std::any::type_name::<T>()); return; };
                let long_name = type_info.type_info().type_path();
                let full_style_path = FullStylePath::new(self.style_ref.path.clone(), long_name);
                let full_style_ref = FullStyleRef::new(self.style_ref.file.clone(), full_style_path);

                // Get the stylesheet entry if it exists and if its file was changed (or we need to initialize).
                let Some(loaded_style) = styles.get::<T>(&full_style_ref, initialized) else { return; };
                initialized = true;

                // Update the node.
                let Ok(mut node) = nodes.get_mut(node)
                else { tracing::warn!(?node, "node missing on style update for {:?}", full_style_ref); return; };

                *node.get_mut(&mut rc) = loaded_style;
            }
        );
        cleanup_reactor_on_despawn(rc, node, token);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Helper trait for creating [`StyleLoaders`](StyleLoader).
pub trait IntoStyleLoader<T: CobwebStyle>
{
    /// Makes a style loader for loading `T` from the stylesheet using `style_ref`.
    fn load(style_ref: &StyleRef) -> StyleLoader<T>;
}

impl<T: CobwebStyle> IntoStyleLoader<T> for T
{
    fn load(style_ref: &StyleRef) -> StyleLoader<T>
    {
        StyleLoader{ style_ref: style_ref.clone(), p: PhantomData::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------
