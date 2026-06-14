//! Deterministic RNG isolation for reproducible cluster state

use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Isolated deterministic random number generator
pub struct IsolatedRng {
    rng: ChaCha8Rng,
}

impl IsolatedRng {
    /// Create new deterministic RNG from seed
    pub fn new_deterministic(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    /// Get mutable reference to underlying RNG
    pub fn rng(&mut self) -> &mut ChaCha8Rng {
        &mut self.rng
    }
}
