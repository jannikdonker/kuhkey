use std::fs::{self};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};

#[derive(Deserialize, Serialize)]
struct Entry {
    password: String,
    username: String,
    url: String,
    notes: String,
    created: u64,
    modified: u64,
    last_used: u64,
}

struct KdfParams {
    m_cost: u32,            // memory
    t_cost: u32,            // time (iterations)
    p_cost: u32,            // parallelism / lanes
}

// Bytes: 4 + 2 + 24 + 12 + 12
struct VaultHeader {
    magic: [u8; 4],         // 4 magic bytes: b"MUHV"
    version: u16,           // e.g. 1
    nonce: [u8; 24],        // 24 for XChaCha20 (12 for AES-GCM)
    salt: [u8; 12],         // 12 for Argon2
    kdf_params: KdfParams,  // 12 bytes (3 * u32)
}

const KEY_PATH: &str = "key";

fn main() {
    // let key = Aes256Gcm::generate_key(&mut OsRng);
    // let cipher = Aes256Gcm::new(&key);
    // fs::write(KEY_PATH, &key.as_slice());

    let cipher = Aes256Gcm::new_from_slice(fs::read(KEY_PATH).unwrap().as_slice()).unwrap();

    // let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let e1 = Entry {
        password: "hunter2".to_string(),
        username: "Peter".to_string(),
        url: "https://google.com".to_string(),
        notes: "".to_string(),
        created: 0,
        modified: 0,
        last_used: 0,
    };
    let e1_json = serde_json::to_string_pretty(&e1).unwrap();
    println!("Original text: {e1_json}");

    // let mut e1_json_enc = cipher.encrypt(&nonce, e1_json.as_bytes()).unwrap();
    // println!("Encrypted: {:02x?}", e1_json_enc);

    // let content = [nonce.as_slice(), &e1_json_enc].concat();
    // fs::write("content", &content);

    let raw = fs::read("content").unwrap();
    let (nonce_bytes, text_bytes) = raw.split_at(12);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let e1_json_dec = String::from_utf8(cipher.decrypt(&nonce, text_bytes).unwrap()).unwrap();
    // let content = fs::read("content").unwrap();
    // let e1_json_dec = String::from_utf8(cipher.decrypt(&nonce, content.as_slice()).unwrap()).unwrap();
    println!("Decrypted text: {e1_json_dec}");
}
