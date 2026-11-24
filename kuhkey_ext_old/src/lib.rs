use kuhkey_core::generator::{self, PasswordOptions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn greet(name: String) -> String {
    format!("Hello {name}, this comes from Rust 🐄")
}

/// Uses the already implemented gen logic to create password from the extetion

/// if something goes wrong the error sents the error to js, so i can be displayed
#[wasm_bindgen]
pub fn generate_password(
    length: usize,
    uppercase: bool,
    lowercase: bool,
    numbers: bool,
    symbols: bool,
    exclude_lookalikes: bool,
) -> Result<String, String> {
    let options = PasswordOptions {
        length,
        uppercase,
        lowercase,
        numbers,
        symbols,
        exclude_lookalikes,
    };

    match generator::generate_password(&options) {
        Ok(password) => Ok(password),
        Err(error) => Err(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{generate_password, greet};

    #[test]
    fn greet_includes_name() {
        assert!(greet("kuh".to_string()).contains("kuh"));
    }

    #[test]
    fn ext_password_generation_succeeds() {
        let pw = generate_password(20, true, true, true, true, false).unwrap();
        assert_eq!(pw.len(), 20);
    }

    #[test]
    fn ext_error_comes_back_as_string() {
        // js only gets a string, so make sure the msg makes it through
        let err = generate_password(20, false, false, false, false, false).unwrap_err();
        assert!(!err.is_empty());
    }

    #[test]
    fn ext_too_short_fails() {
        assert!(generate_password(2, true, true, true, true, false).is_err());
    }
}
