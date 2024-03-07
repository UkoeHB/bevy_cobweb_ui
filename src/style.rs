//local shortcuts
use crate::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use serde::{Deserialize, Serialize};

//standard shortcuts
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
struct StyleLoaderReactors
{
    handles: HashMap<TypeId, SystemCommand>,
}

impl Default for StyleLoaderReactors
{
    fn default() -> Self
    {
        Self{ handles: HashMap::default() }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Updates the style `T` on nodes when the stylesheet is updated or when a loaded node receives a `FinishNode` event.
fn style_loader_reactor<T: CobwebStyle>(
    node_event : EntityEvent<FinishNode>,
    mut rc     : ReactCommands,
    types      : Res<AppTypeRegistry>,
    styles     : ReactRes<StyleSheet>,
    mut nodes  : Query<(&mut React<T>, &mut LoadedStyle<T>)>
){
    // Prep node updater.
    let mut updater = |node: &mut React<T>, ctx: &mut LoadedStyle<T>|
    {
        // Look up the type's short name.
        let type_registry = types.read();
        let Some(type_info) = type_registry.get(TypeId::of::<T>())
        else
        {
            tracing::error!("type registry info missing for {:?}, make sure this type is registered in \
                your app with App::register_type", std::any::type_name::<T>());
            return;
        };
        let long_name = type_info.type_info().type_path();
        let full_style_path = FullStylePath::new(ctx.style_ref.path.clone(), long_name);
        let full_style_ref = FullStyleRef::new(ctx.style_ref.file.clone(), full_style_path);


        // Update the node.
        let reflected = node.get_mut_noreact();
        if styles.apply(&full_style_ref, ctx.initialized, reflected)
        {
            // Trigger change detection and mark initialized.
            node.get_mut(&mut rc);
            ctx.initialized = true;
        }
    };

    // Check if triggered by a node event.
    if let Some((node, _)) = node_event.read()
    {
        // Look up the node.
        let Ok((mut node, mut ctx)) = nodes.get_mut(*node)
        else { tracing::warn!(?node, "node missing on style update for {:?}", std::any::type_name::<T>()); return; };

        // Update the node.
        updater(&mut node, &mut ctx);
    }
    else // Otherwise assume it was triggered for all nodes.
    {
        for (mut node, mut ctx) in nodes.iter_mut()
        {
            updater(&mut node, &mut ctx);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
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
pub trait CobwebStyle:
  ReactComponent
+ Reflect
+ FromReflect
+ PartialEq
+ Clone
+ Default
+ Serialize
+ for<'de> Deserialize<'de>
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

/// Component added to nodes that load `T` from the stylesheet.
///
/// If removed from the node, then the associated style will no longer be updated on the entity when the stylesheet
/// changes.
#[derive(Component)]
pub struct LoadedStyle<T: CobwebStyle>
{
    style_ref: StyleRef,
    initialized: bool,
    p: PhantomData<T>
}

impl<T: CobwebStyle> LoadedStyle<T>
{
    fn new(style_ref: StyleRef) -> Self
    {
        Self{ style_ref, initialized: false, p: PhantomData::default() }
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

        // Save the loading context to this node.
        // - This component is important for filtering for entities with styles that are loaded.
        rc.commands().entity(node).insert(LoadedStyle::<T>::new(self.style_ref));

        // Prep reactor for loading styles for this node.
        // - We manually manage the `style_loader_reactor` because it is generic over `T`.
        rc.commands().syscall(node,
            |In(node): In<Entity>, mut rc: ReactCommands, mut reactors: ResMut<StyleLoaderReactors>|
            {
                let reactor = reactors.handles
                    .entry(TypeId::of::<T>())
                    .or_insert_with(
                        || rc.on_persistent(resource_mutation::<StyleSheet>(), style_loader_reactor::<T>)
                    );
                rc.with(entity_event::<FinishNode>(node), *reactor, ReactorMode::Persistent);
            }
        );
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

pub(crate) struct StylePlugin;

impl Plugin for StylePlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<StyleLoaderReactors>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
