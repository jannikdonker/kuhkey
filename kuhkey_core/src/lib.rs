#![allow(dead_code)]
#![allow(unused)]
#![allow(private_interfaces)]

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use chacha20poly1305::{
    ChaCha20Poly1305, ChaChaPoly1305, XChaCha20Poly1305, aead::common::getrandom,
};
use dirs::data_dir;
use uuid::Uuid;
use zeroize::Zeroizing;

pub mod generator;

pub const APP_ID: &str = "de.edah.kuhkey";

const MAGIC_BYTES: [u8; 4] = *b"KUHV";
const VAULT_VERSION: u32 = 1;
const NONCE_LEN: usize = 24; // 24 for XChaCha20 (12 for AES-GCM)
const SALT_LEN: usize = 16; // 16 for Argon2
const KEY_LEN: usize = 32;

/// MAGIC_BYTES + VERSION + NONCE + SALT + KdFParams
pub const HEADER_LEN: usize = 4 + 4 + NONCE_LEN + SALT_LEN + 12;

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

impl Entry {
    fn new(password: &str, username: &str, url: &str, notes: &str) -> Self {
        let timestamp = current_timestamp();
        Entry {
            id: Uuid::new_v4().as_bytes().to_owned(),
            password: password.to_owned(),
            username: username.to_owned(),
            url: url.to_owned(),
            notes: notes.to_owned(),
            created: timestamp,
            modified: timestamp,
            last_used: 0,
        }
    }
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Copy, Clone)]
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
    fn to_bytes(self) -> [u8; 12] {
        [
            self.m_cost.to_le_bytes(),
            self.t_cost.to_le_bytes(),
            self.p_cost.to_le_bytes(),
        ]
        .as_flattened()
        .try_into()
        .expect("Couldn't convert KdfParams to bytes")
    }

    fn from_bytes(b: &[u8; 12]) -> Self {
        // unwrap is save here because we know the length
        let (m, rest) = b.split_first_chunk::<4>().unwrap();
        let (t, rest) = rest.split_first_chunk::<4>().unwrap();
        let (p, _) = rest.split_first_chunk::<4>().unwrap();

        Self {
            m_cost: u32::from_le_bytes(*m),
            t_cost: u32::from_le_bytes(*t),
            p_cost: u32::from_le_bytes(*p),
        }
    }
}

// Bytes: 4 + 4 + 24 + 16 + 12 = 60
struct VaultHeader {
    magic: [u8; 4],
    version: u32,
    nonce: [u8; NONCE_LEN],
    salt: [u8; SALT_LEN],
    kdf_params: KdfParams, // 12 bytes (3 * u32)
}

impl VaultHeader {
    fn to_bytes(&self) -> [u8; HEADER_LEN] {
        [
            &self.magic as &[u8],
            &self.version.to_le_bytes(),
            &self.nonce,
            &self.salt,
            &self.kdf_params.to_bytes(),
        ]
        .concat()
        .try_into()
        // should never fail
        .expect("Couldn't convert VaultHeader to bytes")
    }

    fn from_bytes(b: &[u8; HEADER_LEN]) -> Result<Self> {
        // unwrap is save here because we know the HEADER_LENGTH
        let (magic, rest) = b.split_first_chunk::<4>().unwrap();
        let (version, rest) = rest.split_first_chunk::<4>().unwrap();
        let (nonce, rest) = rest.split_first_chunk::<NONCE_LEN>().unwrap();
        let (salt, rest) = rest.split_first_chunk::<SALT_LEN>().unwrap();
        let (kdf, _) = rest.split_first_chunk::<12>().unwrap();

        if *magic != MAGIC_BYTES {
            bail!("bad magic bytes");
        }
        let version = u32::from_le_bytes(*version);
        if version != VAULT_VERSION {
            bail!("unsupported vault version");
        }
        let kdf_params = KdfParams::from_bytes(kdf);
        if kdf_params.m_cost > MAX_M_COST {
            bail!("m_cost too large");
        }

        Ok(Self {
            magic: *magic,
            version,
            nonce: *nonce,
            salt: *salt,
            kdf_params,
        })
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

    pub fn from_bytes(b: &[u8]) -> Result<Self> {
        let Some((header, payload)) = b.split_first_chunk::<HEADER_LEN>() else {
            bail!("vault file too short");
        };

        Ok(Self {
            header: VaultHeader::from_bytes(header)?,
            payload: payload.to_vec(),
        })
    }
}

fn derive_key(password: &[u8], salt: &[u8], p: KdfParams) -> Result<Zeroizing<[u8; KEY_LEN]>> {
    todo!()
}

/// Encrypts `entries` into a new vault with a fresh salt and nonce.
/// Call this again on every save: the nonce must never be reused with the same key.
pub fn create_vault(password: &[u8], entries: &[Entry], kdf_params: KdfParams) -> Result<Vault> {
    todo!()
}

/// Decrypts a vault. Fails with `VaultError::Decrypt` on a wrong password or a modified file.
pub fn open_vault(password: &[u8], vault: &Vault) -> Result<Vec<Entry>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header() -> VaultHeader {
        VaultHeader {
            magic: MAGIC_BYTES,
            version: VAULT_VERSION,
            nonce: [7; NONCE_LEN],
            salt: [9; SALT_LEN],
            kdf_params: KdfParams {
                m_cost: 1024,
                t_cost: 5,
                p_cost: 6,
            },
        }
    }

    #[test]
    fn kdf_params_to_bytes_is_little_endian() {
        let b = KdfParams {
            m_cost: 0x01020304,
            t_cost: 5,
            p_cost: 6,
        }
        .to_bytes();
        assert_eq!(b, [4, 3, 2, 1, 5, 0, 0, 0, 6, 0, 0, 0]);
    }

    #[test]
    fn kdf_params_roundtrip() {
        let p = KdfParams {
            m_cost: 123_456,
            t_cost: 7,
            p_cost: 8,
        };
        let r = KdfParams::from_bytes(&p.to_bytes());
        assert_eq!((r.m_cost, r.t_cost, r.p_cost), (123_456, 7, 8));
    }

    #[test]
    fn kdf_params_default_roundtrip() {
        let d = KdfParams::default();
        let r = KdfParams::from_bytes(&d.to_bytes());
        assert_eq!(
            (r.m_cost, r.t_cost, r.p_cost),
            (d.m_cost, d.t_cost, d.p_cost)
        );
    }

    #[test]
    fn header_to_bytes_layout() {
        let bytes = header().to_bytes();
        assert_eq!(bytes.len(), HEADER_LEN);
        assert_eq!(&bytes[..4], b"KUHV");
    }

    #[test]
    fn header_roundtrip() {
        let h = header();
        let r = VaultHeader::from_bytes(&h.to_bytes()).unwrap();
        assert_eq!(r.magic, h.magic);
        assert_eq!(r.version, h.version);
        assert_eq!(r.nonce, h.nonce);
        assert_eq!(r.salt, h.salt);
        assert_eq!(r.kdf_params.m_cost, h.kdf_params.m_cost);
    }

    #[test]
    fn header_rejects_bad_magic() {
        let mut h = header();
        h.magic = *b"XXXX";
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_err());
    }

    #[test]
    fn header_rejects_bad_version() {
        let mut h = header();
        h.version = VAULT_VERSION + 1;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_err());
    }

    #[test]
    fn header_rejects_too_large_m_cost() {
        let mut h = header();
        h.kdf_params.m_cost = MAX_M_COST + 1;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_err());
    }

    #[test]
    fn header_accepts_max_m_cost() {
        let mut h = header();
        h.kdf_params.m_cost = MAX_M_COST;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_ok());
    }

    #[test]
    fn vault_roundtrip() {
        let h = header();
        let v = Vault {
            header: h,
            payload: vec![1, 2, 3, 4, 5],
        };
        let bytes = v.to_bytes();
        assert_eq!(bytes.len(), HEADER_LEN + 5);
        let r = Vault::from_bytes(&bytes).unwrap();
        assert_eq!(r.payload, v.payload);
        assert_eq!(r.header.nonce, v.header.nonce);
        assert_eq!(r.header.salt, v.header.salt);
    }

    #[test]
    fn vault_empty_payload() {
        let h = header();
        let v = Vault {
            header: h,
            payload: vec![],
        };
        let r = Vault::from_bytes(&v.to_bytes()).unwrap();
        assert!(r.payload.is_empty());
    }

    #[test]
    fn vault_too_short() {
        assert!(Vault::from_bytes(&[]).is_err());
        assert!(Vault::from_bytes(&[0; HEADER_LEN - 1]).is_err());
    }

    #[test]
    fn vault_bad_header_propagates() {
        assert!(Vault::from_bytes(&[0; HEADER_LEN]).is_err());
    }
}
