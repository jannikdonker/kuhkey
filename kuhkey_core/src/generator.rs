use std::char;

use anyhow::{Result, bail};
use chacha20poly1305::aead::common::getrandom;
use rand::Rng;
use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};

pub const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub const NUMBERS: &[u8] = b"0123456789";
/// No quotes, backslash, backtick or space: those break when pasted into
/// shells, config files or badly written login forms.
pub const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{};:,.<>/?~";
/// Characters that are easy to mix up when read off a screen or typed by hand.
pub const LOOKALIKES: &[u8] = b"0Oo1Il";
pub const MAXIMUM_PASSWORD_LENGTH: usize = 256;

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

fn generate_random_seed() -> Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf)?;
    Ok(buf)
}

pub fn generate_password(opts: &PasswordOptions) -> Result<String> {
    // Validate that the length fits all requirements
    let min_len = [opts.uppercase, opts.lowercase, opts.numbers, opts.symbols]
        .into_iter()
        .filter(|&enabled| enabled)
        .count();
    if opts.length < min_len {
        bail!("password length is smaller than the number of required character classes")
    }
    if opts.length > MAXIMUM_PASSWORD_LENGTH {
        bail!("password length is larger than the maximum length of {MAXIMUM_PASSWORD_LENGTH}")
    }

    let seed = generate_random_seed()?;
    let mut rng = ChaCha20Rng::from_seed(seed);

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
        bail!("charset is empty")
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
    use crate::generator::{
        LOOKALIKES, LOWERCASE, NUMBERS, PasswordOptions, SYMBOLS, UPPERCASE, generate_password,
        generate_random_seed, password_has_reqs,
    };

    // gen is random, so run each check a bunch of times
    const RUNS: usize = 100;

    fn has_any(pw: &str, set: &[u8]) -> bool {
        pw.bytes().any(|c| set.contains(&c))
    }

    fn only_class(upper: bool, lower: bool, numbers: bool, symbols: bool) -> PasswordOptions {
        PasswordOptions {
            length: 20,
            uppercase: upper,
            lowercase: lower,
            numbers,
            symbols,
            exclude_lookalikes: false,
        }
    }

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

    #[test]
    fn password_respects_custom_length() {
        for length in [4, 8, 64, 256] {
            let opts = PasswordOptions {
                length,
                ..Default::default()
            };
            assert_eq!(generate_password(&opts).unwrap().len(), length);
        }
    }

    #[test]
    fn passwords_are_unique() {
        let opts = PasswordOptions::default();
        let a = generate_password(&opts).unwrap();
        let b = generate_password(&opts).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn password_contains_every_enabled_class() {
        // length 4 = exactly one char per class, worst case for the retry loop
        let opts = PasswordOptions {
            length: 4,
            ..Default::default()
        };
        for _ in 0..RUNS {
            let pw = generate_password(&opts).unwrap();
            assert!(has_any(&pw, UPPERCASE), "no upper in {pw}");
            assert!(has_any(&pw, LOWERCASE), "no lower in {pw}");
            assert!(has_any(&pw, NUMBERS), "no number in {pw}");
            assert!(has_any(&pw, SYMBOLS), "no symbol in {pw}");
        }
    }

    #[test]
    fn password_only_uses_known_chars() {
        let opts = PasswordOptions::default();
        for _ in 0..RUNS {
            let pw = generate_password(&opts).unwrap();
            for c in pw.bytes() {
                assert!(
                    UPPERCASE.contains(&c)
                        || LOWERCASE.contains(&c)
                        || NUMBERS.contains(&c)
                        || SYMBOLS.contains(&c),
                    "unexpected char {:?} in {pw}",
                    c as char
                );
            }
        }
    }

    #[test]
    fn disabled_classes_are_left_out() {
        let cases = [
            (only_class(true, false, false, false), UPPERCASE),
            (only_class(false, true, false, false), LOWERCASE),
            (only_class(false, false, true, false), NUMBERS),
            (only_class(false, false, false, true), SYMBOLS),
        ];
        for (opts, allowed) in cases {
            for _ in 0..RUNS {
                let pw = generate_password(&opts).unwrap();
                assert!(pw.bytes().all(|c| allowed.contains(&c)), "bad char in {pw}");
            }
        }
    }

    #[test]
    fn lookalikes_are_excluded() {
        let opts = PasswordOptions {
            exclude_lookalikes: true,
            ..Default::default()
        };
        for _ in 0..RUNS {
            let pw = generate_password(&opts).unwrap();
            assert!(!has_any(&pw, LOOKALIKES), "lookalike in {pw}");
        }
    }

    #[test]
    fn numbers_only_without_lookalikes_works() {
        // 0 and 1 are gone, so only 2-9 should be left
        let opts = PasswordOptions {
            exclude_lookalikes: true,
            ..only_class(false, false, true, false)
        };
        let pw = generate_password(&opts).unwrap();
        assert!(
            pw.bytes().all(|c| b"23456789".contains(&c)),
            "bad char in {pw}"
        );
    }

    #[test]
    fn length_shorter_than_classes_fails() {
        let opts = PasswordOptions {
            length: 3,
            ..Default::default()
        };
        assert!(generate_password(&opts).is_err());
    }

    #[test]
    fn zero_length_fails() {
        let opts = PasswordOptions {
            length: 0,
            ..Default::default()
        };
        assert!(generate_password(&opts).is_err());
    }

    #[test]
    fn all_classes_disabled_fails() {
        let opts = only_class(false, false, false, false);
        assert!(generate_password(&opts).is_err());
    }

    #[test]
    fn reqs_accepts_matching_password() {
        let opts = PasswordOptions::default();
        assert!(password_has_reqs("Ab3!", &opts));
    }

    #[test]
    fn reqs_rejects_missing_class() {
        let opts = PasswordOptions::default();
        assert!(!password_has_reqs("Ab3x", &opts)); // no symbol
        assert!(!password_has_reqs("ab3!", &opts)); // no upper
    }

    #[test]
    fn reqs_rejects_disabled_class() {
        let opts = only_class(true, true, false, false);
        assert!(!password_has_reqs("Ab3", &opts));
    }

    #[test]
    fn reqs_rejects_lookalikes_when_excluded() {
        let opts = PasswordOptions {
            exclude_lookalikes: true,
            ..Default::default()
        };
        assert!(!password_has_reqs("Ab1!", &opts));
        assert!(password_has_reqs("Ab3!", &opts));
    }
}
