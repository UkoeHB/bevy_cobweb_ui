use bevy_cobweb::prelude::*;

use crate::prelude::ScenePath;

//-------------------------------------------------------------------------------------------------------------------

/// Error returned by [`SceneHandle`](crate::prelude::SceneHandle) methods.
#[derive(Debug)]
pub enum SceneHandleError
{
    GetEntity(ScenePath),
    GetEntityFromRoot(ScenePath),
}

impl std::error::Error for SceneHandleError
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)>
    {
        None
    }
}

impl std::fmt::Display for SceneHandleError
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        match self {
            Self::GetEntity(err) => f.write_fmt(format_args!("GetEntity({:?})", err)),
            Self::GetEntityFromRoot(err) => f.write_fmt(format_args!("GetEntityFromRoot({:?})", err)),
        }
    }
}

impl From<SceneHandleError> for IgnoredError
{
    fn from(_: SceneHandleError) -> Self
    {
        IgnoredError
    }
}

impl From<SceneHandleError> for WarnError
{
    fn from(err: SceneHandleError) -> Self
    {
        WarnError::Msg(format!("SceneHandleError::{}", err))
    }
}

//-------------------------------------------------------------------------------------------------------------------
