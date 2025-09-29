use hpke::rand_core::{CryptoRng, RngCore};

/// A cryptographically secure random number generator that uses getrandom.
#[derive(Clone, Copy, Debug, Default)]
pub struct GetRandomRng;

impl RngCore for GetRandomRng {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        getrandom::fill(dest).expect("getrandom failed")
    }
}

impl CryptoRng for GetRandomRng {}

/// Convenience function to create a new GetRandomRng instance.
pub fn rng() -> GetRandomRng {
    GetRandomRng
}
