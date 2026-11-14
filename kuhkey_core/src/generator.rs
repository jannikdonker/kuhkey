use std::{char, io::Error};

use chacha20poly1305::aead::common::getrandom;
use rand::Rng;
use rand_chacha::{ChaCha12Rng, rand_core::SeedableRng};

pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const NUMBERS: &[u8] = b"0123456789";
/// No quotes, backslash, backtick or space: those break when pasted into
/// shells, config files or badly written login forms.
pub const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.<>/?~";
/// Characters that are easy to mix up when read off a screen or typed by hand.
pub const LOOKALIKES: &[u8] = b"0Oo1Il";

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
    // Validate that the length fits all requirements
    let min_len = [opts.uppercase, opts.lowercase, opts.numbers, opts.symbols]
        .into_iter()
        .filter(|&enabled| enabled)
        .count();
    if opts.length < min_len {
        return Err(Error::new(
            std::io::ErrorKind::InvalidInput,
            "password length is smaller than the number of required character classes",
        ));
    }

    let seed = generate_random_seed()?;
    let mut rng = ChaCha12Rng::from_seed(seed);

    let mut charset: Vec<u8> = Vec::new();

    if opts.uppercase {
        charset.append(&mut UPPERCASE.to_vec());
    };
    if opts.lowercase {
        charset.append(&mut LOWERCASE.to_vec());
    };
    if opts.numbers {
        charset.append(&mut NUMBERS.to_vec());
    };
    if opts.symbols {
        charset.append(&mut SYMBOLS.to_vec());
    };
    if opts.exclude_lookalikes {
        charset.retain(|c| !LOOKALIKES.contains(c));
    };

    if charset.is_empty() {
        panic!()
        // TODO: impl error type
    };

    let mut pw = String::with_capacity(opts.length);
    loop {
        while pw.len() < opts.length {
            let mut buf = [0u8; 64];
            rng.fill_bytes(&mut buf);
            for &b in &buf {
                if charset.contains(&b) {
                    pw.push(b as char);
                };
                if pw.len() == opts.length {
                    break;
                };
            }
        }
        if password_has_reqs(&pw, opts) {
            return Ok(pw);
        }
        pw.clear();
    }
}

fn password_has_reqs(pw: &str, opts: &PasswordOptions) -> bool {
    let mut contains_upper = false;
    let mut contains_lower = false;
    let mut contains_numbers = false;
    let mut contains_symbols = false;
    let mut contains_lookalikes = false;

    for c in pw.as_bytes() {
        if UPPERCASE.contains(c) {
            contains_upper = true;
        };
        if LOWERCASE.contains(c) {
            contains_lower = true;
        };
        if NUMBERS.contains(c) {
            contains_numbers = true;
        };
        if SYMBOLS.contains(c) {
            contains_symbols = true;
        };
        if LOOKALIKES.contains(c) {
            contains_lookalikes = true;
        };
    }

    // 0 0 1
    // 0 1 0
    // 1 0 0
    // 1 1 1
    // XNOR or just ==
    return (opts.uppercase == contains_upper)
        && (opts.lowercase == contains_lower)
        && (opts.numbers == contains_numbers)
        && (opts.symbols == contains_symbols)
        && (!opts.exclude_lookalikes || !contains_lookalikes);
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
