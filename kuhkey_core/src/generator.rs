use std::io::Error;

use chacha20poly1305::aead::common::getrandom;
use rand::distr::{Alphanumeric, SampleString};
use rand_chacha::{ChaCha12Rng, rand_core::SeedableRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PasswordOptions {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub numbers: bool,
    pub symbols: bool,
    /// Leave out the characters in [`LOOKALIKES`].
    pub exclude_lookalikes: bool,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            numbers: true,
            symbols: true,
            exclude_lookalikes: false,
        }
    }
}

fn generate_random_seed() -> Result<[u8; 32], getrandom::Error> {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf)?;
    Ok(buf)
}

pub fn generate_password(opts: &PasswordOptions) -> Result<String, Error> {
    let seed = generate_random_seed()?;

    let mut rng = ChaCha12Rng::from_seed(seed);
    Ok(Alphanumeric.sample_string(&mut rng, opts.length))
}

#[cfg(test)]
mod tests {
    use crate::generator::{PasswordOptions, generate_password, generate_random_seed};

    #[test]
    fn seed_generation_succeeds() {
        assert!(generate_random_seed().is_ok());
    }

    #[test]
    fn seed_is_not_all_zeros() {
        let seed = generate_random_seed().unwrap();
        assert_ne!(seed, [0u8; 32]);
    }

    #[test]
    fn seeds_are_unique() {
        let a = generate_random_seed().unwrap();
        let b = generate_random_seed().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn password_generation_succeeds() {
        let opts = PasswordOptions::default();
        assert!(generate_password(&opts).is_ok());
    }

    #[test]
    fn password_is_correct_length() {
        let opts = PasswordOptions::default();
        assert!(generate_password(&opts).unwrap().len().eq(&20));
    }
}
