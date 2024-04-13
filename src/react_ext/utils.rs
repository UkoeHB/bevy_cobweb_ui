use bevy::prelude::*;
use bevy_cobweb::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Despawns the `token`'s reactor when `entity` is despawned.
pub fn cleanup_reactor_on_despawn(rc: &mut ReactCommands, entity: Entity, token: RevokeToken)
{
    rc.on(despawn(entity), move |mut rc: ReactCommands| { rc.revoke(token.clone()); });
}

//-------------------------------------------------------------------------------------------------------------------
