use std::borrow::Cow;

use bevy::prelude::*;
use bevy::window::SystemCursorIcon;
use bevy::winit::cursor::{CursorIcon, CustomCursor};
use smallvec::SmallVec;

use crate::prelude::*;
use crate::sickle::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct CursorSource
{
    primary: Option<LoadableCursor>,
    temporary: Option<LoadableCursor>,
}

impl CursorSource
{
    /// Returns a cursor if either the primary or temporary is set.
    ///
    /// Clears the temporary.
    fn get_next_cursor(&mut self, img_map: &mut ImageMap, asset_server: &AssetServer) -> Option<CursorIcon>
    {
        let cursor = self.temporary.take().or_else(|| self.primary.clone())?;

        Some(cursor.into_cursor_icon(img_map, asset_server)?)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Iterates available `TempCursors` to extract the current temp cursor.
fn get_temp_cursor(mut source: ResMut<CursorSource>, temps: Query<(Entity, &TempCursor)>)
{
    // Look for highest priority non-None cursor.
    let mut found: Option<(u8, Entity, &LoadableCursor)> = None;
    let mut found_second: Option<(u8, Entity, &LoadableCursor)> = None;
    for (entity, temp) in temps
        .iter()
        .filter(|(_, t)| !matches!(t.cursor, LoadableCursor::None))
    {
        let Some((prio, _, _)) = &found else {
            found = Some((temp.priority, entity, &temp.cursor));
            continue;
        };

        if temp.priority < *prio {
            continue;
        }

        if temp.priority == *prio {
            found_second = Some((temp.priority, entity, &temp.cursor));
            continue;
        }

        found = Some((temp.priority, entity, &temp.cursor));
    }

    // Warn if there is a conflict.
    if let Some((entity, second)) = found_second.and_then(|(prio, e, s)| {
        if prio >= found.unwrap().0 {
            return Some((e, s));
        }
        None
    }) {
        warn_once!("multiple TempCursor instances detected (first: {:?} {:?}, second: {:?} {:?}); only one can be used at a \
            time; this warning only prints once", found.unwrap().1, found.unwrap().2, entity, second);
    }

    // Set the cursor.
    if let Some((_, _, cursor)) = found {
        source.temporary = Some(cursor.clone());
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Sets the cursor if we have anything loaded in the `CursorSource`. Does nothing if no cursor is loaded, in case
/// the user wants to manage the cursor with a custom approach.
fn refresh_cursor_icon(
    asset_server: Res<AssetServer>,
    mut c: Commands,
    mut source: ResMut<CursorSource>,
    windows: Query<(Entity, Option<&CursorIcon>), With<Window>>,
    mut img_map: ResMut<ImageMap>,
)
{
    let next_cursor = source.get_next_cursor(&mut img_map, &asset_server);
    for (window_entity, current_cursor) in windows.iter() {
        if current_cursor == next_cursor.as_ref() {
            continue;
        }
        let Some(next_cursor) = next_cursor.clone() else { continue };

        c.entity(window_entity).insert(next_cursor);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// A cursor type that can be loaded via [`PrimaryCursor`] or [`TempCursor`].
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum LoadableCursor
{
    /// `None` means the loadable cursor should be ignored.
    ///
    /// Used as a default in `Responsive<TempCursor>`, where you need to set an `idle` value.
    #[default]
    None,
    /// Mirrors [`CustomCursor`].
    Custom
    {
        /// Image path. It is recommended (but not required) to pre-load the image via [`LoadedImages`].
        ///
        /// The image must be in 8 bit int or 32 bit float rgba. PNG images work well for this.
        image: Cow<'static, str>,
        /// X and Y coordinates of the hotspot in pixels. The hotspot must be within the image bounds.
        hotspot: (u16, u16),
    },
    /// A URL to an image to use as the cursor.
    ///
    /// Only usable on WASM targets.
    Url
    {
        /// Web URL to an image to use as the cursor. PNGs preferred. Cursor
        /// creation can fail if the image is invalid or not reachable.
        url: Cow<'static, str>,
        /// X and Y coordinates of the hotspot in pixels. The hotspot must be within the image bounds.
        hotspot: (u16, u16),
    },
    System(SystemCursorIcon),
}

impl LoadableCursor
{
    pub fn into_cursor_icon(self, img_map: &mut ImageMap, asset_server: &AssetServer) -> Option<CursorIcon>
    {
        match self {
            Self::None => None,
            Self::Custom { image, hotspot } => {
                let handle = img_map.get_or_load(image, asset_server);
                Some(CursorIcon::Custom(CustomCursor::Image { handle, hotspot }))
            }
            Self::Url { url, hotspot: _hotspot } => {
                #[cfg(all(target_family = "wasm", target_os = "unknown"))]
                {
                    Some(CursorIcon::Custom(CustomCursor::Url { url, hotspot: _hotspot }))
                }

                #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
                {
                    warn_once!("failed making cursor icon from URL {:?}; only WASM targets are supported, but the target \
                        is not WASM; this warning only prints once", url);
                    None
                }
            }
            Self::System(icon) => Some(CursorIcon::System(icon)),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Command that sets the primary [`CursorIcon`] on all windows of the app.
///
/// The primary icon can be temporarily overridden by a [`TempCursor`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Deref, DerefMut)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct PrimaryCursor(pub LoadableCursor);

impl Command for PrimaryCursor
{
    fn apply(self, world: &mut World)
    {
        world.resource_mut::<CursorSource>().primary = Some(self.0);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Component that tries to set [`CursorIcon`] on all windows of the app every tick. Set the value to
/// [`LoadableCursor::None`]` to disable it.
///
/// To set a long-term 'primary cursor', use the [`PrimaryCursor`] command.
///
/// See [`ResponsiveCursor`] for an easy way to use this.
#[derive(Component, Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub struct TempCursor
{
    /// Higher priority cursors will override lower priority cursors.
    ///
    /// Used as a hack so press cursors won't be overridden by hover cursors when moving off an element.
    pub priority: u8,
    pub cursor: LoadableCursor,
}

impl Instruction for TempCursor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut emut| {
            emut.insert(self);
        });
    }

    fn revert(entity: Entity, world: &mut World)
    {
        let _ = world.get_entity_mut(entity).map(|mut emut| {
            emut.remove::<Self>();
        });
    }
}

impl StaticAttribute for TempCursor
{
    type Value = Self;
    fn construct(value: Self::Value) -> Self
    {
        value
    }
}
impl ResponsiveAttribute for TempCursor {}

//-------------------------------------------------------------------------------------------------------------------

/// Instruction that sets [`TempCursor`] on the entity when it is hovered or pressed.
///
/// Note that this should usually be paired with a [`PrimaryCursor`] command for the default cursor. Otherwise
/// the cursor can get 'stuck' on responsive cursor values.
// TODO: There is a bug where if you only have `hover` set, then the hover cursor will be maintained when you
// press and drag away from the entity until you release.
#[derive(Reflect, Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResponsiveCursor
{
    /// Specifies which [`PseudoStates`](PseudoState) the entity must be in for these cursor settings to be
    /// active.
    ///
    /// Only used if this instruction is applied to an entity with a [`ControlLabel`].
    #[reflect(default)]
    pub state: Option<SmallVec<[PseudoState; 3]>>,
    /// The cursor to display when the entity is hovered.
    #[reflect(default)]
    pub hover: Option<LoadableCursor>,
    /// The cursor to display when the entity is pressed.
    #[reflect(default)]
    pub press: Option<LoadableCursor>,
}

impl Instruction for ResponsiveCursor
{
    fn apply(self, entity: Entity, world: &mut World)
    {
        // Get the cursor values to set.
        // - The press falls back to 'hover' to make sure the cursor priority gets bumped up when we have pointer
        // lock on this entity. Otherwise hovers on other entities during a pointer lock may contend with the
        // hover cursor from this entity (which would get repurposed for the press value if press is not set).
        let press = self
            .press
            .or_else(|| self.hover.clone())
            .map(|cursor| TempCursor { priority: 2, cursor });
        let hover = self.hover.map(|cursor| TempCursor { priority: 1, cursor });

        // Get the label if this entity has one. We always want the interaction source to be self.
        let respond_to = world.get::<ControlLabel>(entity).map(|l| (**l).clone());

        // Make a Responsive instruction and apply it.
        let responsive = Responsive::<TempCursor> {
            state: self.state,
            respond_to,
            idle: TempCursor { priority: 0, cursor: LoadableCursor::None },
            hover,
            press,
            ..Default::default()
        };
        responsive.apply(entity, world);
    }

    fn revert(entity: Entity, world: &mut World)
    {
        Responsive::<TempCursor>::revert(entity, world);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct CursorPlugin;

impl Plugin for CursorPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CursorSource>()
            .register_command_type::<PrimaryCursor>()
            .register_responsive::<TempCursor>()
            .register_instruction_type::<ResponsiveCursor>()
            // Note: bevy's cursor_update system runs in Last but doesn't have a system set, so we need to put
            // these in PostUpdate
            .add_systems(PostUpdate, (get_temp_cursor, refresh_cursor_icon).chain());
    }
}

//-------------------------------------------------------------------------------------------------------------------
