use random_word::Lang;
use std::char;

use anyhow::{Result, bail};
use chacha20poly1305::aead::common::getrandom;
use rand::{Rng, RngExt};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordCase {
    Lower,       // kuhkey
    Capitalized, // Kuhkey
    Upper,       // KUHKEY
}

/// settings for the username gen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// maybe we delete debug afterwards
pub struct UsernameOptions {
    /// max len of the whole username, numbers included
    pub max_length: usize,
    /// how many words
    pub word_count: usize,
    /// casing of the words
    pub case: WordCase,
    /// put a "_" between the words
    pub seperator: bool,
    /// add random numbers to the username
    pub numbers: bool,
    /// true = front, false = end
    pub numbers_front: bool,
    /// how many numbers, only used if `numbers` is on
    pub numbers_len: usize,
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

impl Default for UsernameOptions {
    fn default() -> Self {
        Self {
            max_length: 20,
            word_count: 2,
            case: WordCase::Capitalized,
            seperator: false,
            numbers: true,
            numbers_front: false,
            numbers_len: 4,
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
    (opts.uppercase == contains_upper)
        && (opts.lowercase == contains_lower)
        && (opts.numbers == contains_numbers)
        && (opts.symbols == contains_symbols)
        && (!opts.exclude_lookalikes || !contains_lookalikes)
}

pub const USERNAME_SEPARATOR: char = '_';
pub const MIN_WORD_LENGTH: usize = 4;

/// gen a username from random words, optional with numbers in front or at the end
///
/// errors if `word_count` is 0, `numbers_len` is 0 with numbers on,
/// or `max_length` is too small for the words
pub fn generate_username(opts: &UsernameOptions) -> Result<String> {
    if opts.word_count < 1 {
        bail!("word_count must be higher or equal 1")
    }

    let mut budget = opts.max_length;
    if opts.numbers {
        budget = budget.saturating_sub(opts.numbers_len); // if negative (cant do this in usize) it will be 0 instead of underflowing 
    }

    // how long one word should be max
    let mut max_word_len = budget / opts.word_count;

    if opts.seperator && max_word_len > 0 {
        max_word_len -= 1; //so we have room for the "_" seperator
    }

    if opts.numbers && opts.numbers_len == 0 {
        bail!("numbers_len has to be at least 1")
    }

    if max_word_len < MIN_WORD_LENGTH {
        bail!("max_length is too small for {} words", opts.word_count);
    }

    let mut username = String::new();
    for i in 0..opts.word_count {
        if opts.seperator && i > 0 {
            username.push(USERNAME_SEPARATOR);
        }
        let word = pick_word(max_word_len);
        username.push_str(&apply_case(word, opts.case));
    }
    // put the numbers in front or at the end
    if opts.numbers {
        let nums = random_numbers(opts.numbers_len)?;
        if opts.numbers_front {
            username.insert_str(0, &nums);
        } else {
            username.push_str(&nums);
        }
    }
    Ok(username)
}

/// gen a string of `len` random digits, e.g. "4821"
// holy overpowerd
fn random_numbers(len: usize) -> Result<String> {
    let seed = generate_random_seed()?;
    let mut rng = ChaCha20Rng::from_seed(seed);

    let mut nums = String::new(); // we also could use with_capacity instead of new, however i think the speed difference isnt that big of a difference here
    for _ in 0..len {
        let digit: u8 = rng.random_range(0..10); // 0 to 9
        nums.push_str(&digit.to_string());
    }
    Ok(nums)
}
fn apply_case(word: &str, case: WordCase) -> String {
    match case {
        WordCase::Lower => word.to_lowercase(),
        WordCase::Upper => word.to_uppercase(),
        WordCase::Capitalized => {
            let lower = word.to_lowercase();
            let first = lower[..1].to_uppercase();
            let rest = &lower[1..];
            first + rest
        }
    }
}

fn pick_word(max_len: usize) -> &'static str {
    loop {
        let word = random_word::get(Lang::En);

        // falls die länge nicht passt holen wir ein neues word
        if word.len() < 3 || word.len() > max_len {
            continue;
        }

        return word;
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::{
        LOOKALIKES, LOWERCASE, NUMBERS, PasswordOptions, SYMBOLS, UPPERCASE, USERNAME_SEPARATOR,
        UsernameOptions, generate_password, generate_random_seed, generate_username,
        password_has_reqs, random_numbers,
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

    // username gen

    fn count_digits(s: &str) -> usize {
        s.bytes().filter(|c| NUMBERS.contains(c)).count()
    }

    fn with_numbers(front: bool, len: usize) -> UsernameOptions {
        UsernameOptions {
            numbers: true,
            numbers_front: front,
            numbers_len: len,
            ..Default::default()
        }
    }

    #[test]
    fn username_generation_succeeds() {
        let opts = UsernameOptions::default();
        assert!(generate_username(&opts).is_ok());
    }

    #[test]
    fn username_respects_max_length() {
        // numbers + separator both eat len, so test all combos
        for numbers in [false, true] {
            for seperator in [false, true] {
                let opts = UsernameOptions {
                    numbers,
                    seperator,
                    ..Default::default()
                };
                for _ in 0..RUNS {
                    let name = generate_username(&opts).unwrap();
                    assert!(name.len() <= opts.max_length, "{name} is too long");
                }
            }
        }
    }

    #[test]
    fn numbers_are_at_the_end() {
        let opts = with_numbers(false, 4);
        for _ in 0..RUNS {
            let name = generate_username(&opts).unwrap();
            let (words, nums) = name.split_at(name.len() - 4);
            assert_eq!(count_digits(nums), 4, "no numbers at the end of {name}");
            assert_eq!(count_digits(words), 0, "numbers in the words of {name}");
        }
    }

    #[test]
    fn numbers_are_in_front() {
        let opts = with_numbers(true, 4);
        for _ in 0..RUNS {
            let name = generate_username(&opts).unwrap();
            let (nums, words) = name.split_at(4);
            assert_eq!(count_digits(nums), 4, "no numbers in front of {name}");
            assert_eq!(count_digits(words), 0, "numbers in the words of {name}");
        }
    }

    #[test]
    fn numbers_respect_custom_length() {
        for len in [1, 3, 6] {
            let name = generate_username(&with_numbers(false, len)).unwrap();
            assert_eq!(
                count_digits(&name),
                len,
                "wrong amount of numbers in {name}"
            );
        }
    }

    #[test]
    fn no_numbers_when_off() {
        let opts = UsernameOptions {
            numbers: false,
            ..Default::default()
        };
        for _ in 0..RUNS {
            let name = generate_username(&opts).unwrap();
            assert_eq!(count_digits(&name), 0, "numbers in {name}");
        }
    }

    #[test]
    fn separator_is_between_words() {
        let opts = UsernameOptions {
            word_count: 3,
            seperator: true,
            max_length: 30,
            ..Default::default()
        };
        let name = generate_username(&opts).unwrap();
        let seps = name.chars().filter(|&c| c == USERNAME_SEPARATOR).count();
        assert_eq!(seps, 2, "wrong amount of separators in {name}");
    }

    #[test]
    fn zero_numbers_len_fails() {
        assert!(generate_username(&with_numbers(false, 0)).is_err());
    }

    #[test]
    fn zero_words_fails() {
        let opts = UsernameOptions {
            word_count: 0,
            ..Default::default()
        };
        assert!(generate_username(&opts).is_err());
    }

    #[test]
    fn too_small_max_length_fails() {
        let opts = UsernameOptions {
            max_length: 5,
            ..Default::default()
        };
        assert!(generate_username(&opts).is_err());
    }

    #[test]
    fn numbers_longer_than_max_length_fails() {
        // saturating_sub makes the budget 0, so this has to error and not panic
        let opts = UsernameOptions {
            max_length: 10,
            ..with_numbers(false, 50)
        };
        assert!(generate_username(&opts).is_err());
    }

    #[test]
    fn random_numbers_are_only_digits() {
        for len in [0, 1, 4, 32] {
            let nums = random_numbers(len).unwrap();
            assert_eq!(nums.len(), len);
            assert_eq!(count_digits(&nums), len, "non digit in {nums}");
        }
    }
}
