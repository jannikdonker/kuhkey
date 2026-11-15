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
