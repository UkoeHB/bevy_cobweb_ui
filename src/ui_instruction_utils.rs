use bevy::prelude::*;
use bevy_cobweb::prelude::*;


//-------------------------------------------------------------------------------------------------------------------

/// Despawns the `token`'s reactor when `node` is despawned.
pub fn cleanup_reactor_on_despawn(rc: &mut ReactCommands, node: Entity, token: RevokeToken)
{
    rc.on(despawn(node), move |mut rc: ReactCommands| { rc.revoke(token.clone()); });
}

//-------------------------------------------------------------------------------------------------------------------
