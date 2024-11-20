use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct DemoRng
{
    rng: ChaCha8Rng,
}

impl DemoRng
{
    pub fn new(seed: u64) -> Self
    {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        Self { rng }
    }

    pub fn rng(&mut self) -> &mut ChaCha8Rng
    {
        &mut self.rng
    }
}

//-------------------------------------------------------------------------------------------------------------------
