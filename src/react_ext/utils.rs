use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Despawns the `token`'s reactor when `entity` is despawned.
pub fn cleanup_reactor_on_despawn(c: &mut Commands, entity: Entity, token: RevokeToken)
{
    c.react().on(despawn(entity), move |mut c: Commands| { c.react().revoke(token.clone()); });
}

//-------------------------------------------------------------------------------------------------------------------
