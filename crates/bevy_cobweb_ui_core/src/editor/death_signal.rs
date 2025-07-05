use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

/// Component that sets a signal to `true` on drop. See `DeathSignal`.
#[derive(Component)]
pub struct DeathSignaler
{
    signal: Arc<AtomicBool>,
}

impl DeathSignaler
{
    pub fn new() -> (Self, DeathSignal)
    {
        let signal = Arc::new(AtomicBool::new(false));
        (Self { signal: signal.clone() }, DeathSignal { signal })
    }
}

impl Drop for DeathSignaler
{
    fn drop(&mut self)
    {
        self.signal.store(true, Ordering::Relaxed);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// If [`Self::is_dead`] returns `true` then the paired `DeathSignaler` has been dropped.
#[derive(Clone, Debug)]
pub struct DeathSignal
{
    signal: Arc<AtomicBool>,
}

impl DeathSignal
{
    pub fn is_dead(&self) -> bool
    {
        self.signal.load(Ordering::Relaxed)
    }
}

//-------------------------------------------------------------------------------------------------------------------
