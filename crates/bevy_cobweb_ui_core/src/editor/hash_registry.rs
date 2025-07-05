use core::hash::Hasher;
use std::hash::BuildHasher;
use std::sync::{Arc, Mutex};

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use foldhash::fast::FixedState;

//-------------------------------------------------------------------------------------------------------------------

/// Tracks a map of [ file name : file data hash ].
///
/// Used as an optimization to avoid redundantly parsing files that were saved by the editor.
#[derive(Resource, Clone)]
pub struct CobHashRegistry
{
    registry: Arc<Mutex<HashMap<String, CobFileHash>>>,
}

impl CobHashRegistry
{
    /// Returns `true` if the file's hash changed.
    pub(crate) fn try_refresh_file(&self, file: &String, hash: CobFileHash) -> bool
    {
        let Ok(mut registry) = self.registry.lock() else {
            warn_once!("CobHashRegistry's internal mutex is poisoned, which may reduce COB file loading perf \
                slightly; this warning only prints once");
            return true;
        };

        let Some(prev) = registry.insert(file.clone(), hash) else { return true };
        prev != hash
    }

    /// The saved hash is only changed if `old` matches the currently-saved hash.
    /// This is used to synchronize editor save operations with file refreshes that occur in the asset loader. It
    /// ensures if there is a file moving from the asset loader to the editor, that the editor's saved file will
    /// not be dropped and will instead 'flow through' to the editor in order to 'overwrite' that in-motion file in
    /// the cobweb backend (i.e. there will be a file parsing hiccup - this is a race condition we don't have
    /// a good solution for).
    ///
    /// Note that if there is a file between the save and asset loader, then the `new` hash inserted here will
    /// get refreshed away by that file, so the save will automatically flow through to eventually overwrite that
    /// file in the cobweb backend.
    pub(super) fn set_file_for_save(&self, file: &str, old: CobFileHash, new: CobFileHash)
    {
        let Ok(mut registry) = self.registry.lock() else {
            warn_once!("CobHashRegistry's internal mutex is poisoned, which may reduce COB file loading perf \
                slightly; this warning only prints once");
            return;
        };

        let Some(entry) = registry.get_mut(file) else {
            // This can happen if the editor creates a new file and then saves a change to it. If the new file has
            // not reached `try_refresh_file` yet, then it won't have a registry entry.
            registry.insert(file.to_string(), new);
            return;
        };

        if *entry != old {
            return;
        }

        *entry = new;
    }
}

impl Default for CobHashRegistry
{
    fn default() -> Self
    {
        Self { registry: Arc::new(Mutex::new(HashMap::default())) }
    }
}

//-------------------------------------------------------------------------------------------------------------------

const FILE_HASH_SEED: u64 = u64::from_le_bytes([b'C', b'O', b'B', b'_', b'H', b'A', b'S', b'H']);
const HASH_STATE: FixedState = FixedState::with_seed(FILE_HASH_SEED);

/// Cryptographically *insecure* hash of file data.
///
/// Hashes stored here are not guaranteed to be stable between platforms or `foldhash` versions.
///
/// We include the file length to reduce the chance of accidental collisions even further.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct CobFileHash(u64, usize);

impl CobFileHash
{
    pub(crate) fn new(file: &[u8]) -> Self
    {
        let mut hasher = HASH_STATE.build_hasher();
        hasher.write(file);
        Self(hasher.finish(), file.len())
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct CobHashRegistryPlugin;

impl Plugin for CobHashRegistryPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<CobHashRegistry>();
    }
}

//-------------------------------------------------------------------------------------------------------------------
