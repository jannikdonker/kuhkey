use chacha20poly1305::{
    ChaCha20Poly1305, ChaChaPoly1305, XChaCha20Poly1305, aead::common::getrandom,
};
use rand_core::Rng;
use serde::{Deserialize, Serialize};
use std::io::Error;

use zeroize::Zeroizing;

mod generator;

const MAGIC_BYTES: [u8; 4] = *b"KUHV";
const VAULT_VERSION: u16 = 1;
const NONCE_LEN: usize = 24; // 24 for XChaCha20 (12 for AES-GCM)
const SALT_LEN: usize = 16; // 16 for Argon2
const KEY_LEN: usize = 32;

/// MAGIC_BYTES + VERSION + NONCE + SALT + KdFParams
pub const HEADER_LEN: usize = 4 + 2 + NONCE_LEN + SALT_LEN + 12;

/// Refuse to run Argon2 with more than 1 GiB
const MAX_M_COST: u32 = 1024 * 1024;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct Entry {
    id: [u8; 16],
    password: String,
    username: String,
    url: String,
    notes: String,
    created: u64,
    modified: u64,
    last_used: u64,
}

struct KdfParams {
    m_cost: u32, // memory in KiB
    t_cost: u32, // time (iterations)
    p_cost: u32, // parallelism / lanes
}

impl Default for KdfParams {
    /// 64 MiB, 3 iterations, 4 lanes
    fn default() -> Self {
        Self {
            m_cost: 64 * 1024,
            t_cost: 3,
            p_cost: 4,
        }
    }
}

impl KdfParams {
    pub fn to_bytes(self) -> [u8; 12] {
        todo!()
    }
}

// Bytes: 4 + 2 + 24 + 12 + 12 = 54
struct VaultHeader {
    magic: [u8; 4],
    version: u32,
    nonce: [u8; NONCE_LEN],
    salt: [u8; SALT_LEN],
    kdf_params: KdfParams, // 12 bytes (3 * u32)
}

impl VaultHeader {
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        todo!()
    }

    pub fn from_bytes(b: &[u8; HEADER_LEN]) -> Result<Self, Error> {
        todo!()
    }
}

struct Vault {
    header: VaultHeader,
    payload: Vec<u8>,
}

impl Vault {
    pub fn to_bytes(&self) -> Vec<u8> {
        [&self.header.to_bytes()[..], &self.payload].concat()
    }

    pub fn from_bytes(b: &[u8]) -> Result<Self, Error> {
        todo!()
    }
}

fn derive_key(
    password: &[u8],
    salt: &[u8],
    p: KdfParams,
) -> Result<Zeroizing<[u8; KEY_LEN]>, Error> {
    todo!()
}

/// Encrypts `entries` into a new vault with a fresh salt and nonce.
/// Call this again on every save: the nonce must never be reused with the same key.
pub fn create_vault(
    password: &[u8],
    entries: &[Entry],
    kdf_params: KdfParams,
) -> Result<Vault, Error> {
    todo!()
}

/// Decrypts a vault. Fails with `VaultError::Decrypt` on a wrong password or a modified file.
pub fn open_vault(password: &[u8], vault: &Vault) -> Result<Vec<Entry>, Error> {
    todo!()
}
