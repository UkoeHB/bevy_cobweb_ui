use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// System sets in [`PostUpdate`] where localization updates are performed.
#[derive(SystemSet, Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub enum LocalizationSet
{
    /// System set where languages are negotiated when [`Locale`](crate::Locale) changes.
    Negotiate,
    /// System set where auto-localization occurs as needed.
    Update,
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LocalizationSetPlugin;

impl Plugin for LocalizationSetPlugin
{
    fn build(&self, app: &mut App)
    {
        app.configure_sets(
            PostUpdate,
            (LocalizationSet::Negotiate, LocalizationSet::Update).chain(),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
