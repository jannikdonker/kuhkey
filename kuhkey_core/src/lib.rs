use anyhow::{Context, Ok, Result, bail};
use argon2::{
    Algorithm::{self},
    Argon2, Params, Version,
};
use std::path::{Path, PathBuf};
use web_time::{Duration, SystemTime, UNIX_EPOCH};

use chacha20poly1305::{
    KeyInit, XChaCha20Poly1305, XNonce,
    aead::{Aead, Payload, common::getrandom},
};
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use ::dirs::data_dir;

pub mod generator;

pub const APP_ID: &str = "de.kuhkey";

const MAGIC_BYTES: [u8; 4] = *b"KUHV";
const VAULT_VERSION: u32 = 1;
const NONCE_LEN: usize = 24; // 24 for XChaCha20 (12 for AES-GCM)
const SALT_LEN: usize = 16; // 16 for Argon2
const KEY_LEN: usize = 32;

/// MAGIC_BYTES + VERSION + UUID + NONCE + SALT + KdFParams + name length (u8).
/// The variable-length vault name follows directly after.
const HEADER_FIXED_LEN: usize = 4 + 4 + 16 + NONCE_LEN + SALT_LEN + 12 + 1;

/// The name length is stored as a u8
const MAX_VAULT_NAME_LEN: usize = u8::MAX as usize;

/// Refuse to run Argon2 with more than 1 GiB
const MAX_M_COST: u32 = 1024 * 1024;
/// Refuse to run Argon2 with more than 64 iterations
const MAX_T_COST: u32 = 64;
/// Refuse to run Argon2 with more than 16 lanes
const MAX_P_COST: u32 = 16;

/// A single credential entry stored in a vault, zeroized on drop.
#[derive(Clone, PartialEq, Zeroize, ZeroizeOnDrop)]
pub struct Entry {
    #[zeroize(skip)] // id doesnt need to be zeroized
    id: Uuid,
    name: String,
    password: String,
    username: String,
    url: String,
    notes: String,
    created: u64,
    modified: u64,
    last_used: u64,
}

impl Entry {
    /// Creates a new entry with a generated UUID, current timestamps for `created` and `modified`, and `last_used` set to the Unix epoch.
    pub fn new(name: &str, password: &str, username: &str, url: &str, notes: &str) -> Self {
        let timestamp = current_timestamp_secs();
        Entry {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            password: password.to_owned(),
            username: username.to_owned(),
            url: url.to_owned(),
            notes: notes.to_owned(),
            created: timestamp,
            modified: timestamp,
            last_used: 0,
        }
    }

    /// Returns the entry's unique identifier.
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    /// Returns the creation timestamp as a `SystemTime`.
    pub fn get_created(&self) -> SystemTime {
        systemtime_from_timestamp(self.created)
    }

    /// Returns the last modification timestamp as a `SystemTime`.
    pub fn get_modified(&self) -> SystemTime {
        systemtime_from_timestamp(self.modified)
    }

    /// Returns the last-used timestamp as a `SystemTime`.
    pub fn get_last_used(&self) -> SystemTime {
        systemtime_from_timestamp(self.last_used)
    }

    /// Returns the entry's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Returns the entry's password.
    pub fn get_password(&self) -> &str {
        &self.password
    }

    /// Returns the entry's username.
    pub fn get_username(&self) -> &str {
        &self.username
    }

    /// Returns the entry's URL.
    pub fn get_url(&self) -> &str {
        &self.url
    }

    /// Returns the entry's notes.
    pub fn get_notes(&self) -> &str {
        &self.notes
    }

    /// Replaces the entry's name, zeroizing the old value and updating `modified`.
    pub fn set_name(&mut self, new_name: &str) {
        self.name.zeroize();
        self.name.push_str(new_name);
        self.modified = current_timestamp_secs();
    }

    /// Replaces the entry's password, zeroizing the old value and updating `modified`.
    pub fn set_password(&mut self, new_password: &str) {
        self.password.zeroize();
        self.password.push_str(new_password);
        self.modified = current_timestamp_secs();
    }

    /// Replaces the entry's username, zeroizing the old value and updating `modified`.
    pub fn set_username(&mut self, new_username: &str) {
        self.username.zeroize();
        self.username.push_str(new_username);
        self.modified = current_timestamp_secs();
    }

    /// Replaces the entry's URL, zeroizing the old value and updating `modified`.
    pub fn set_url(&mut self, new_url: &str) {
        self.url.zeroize();
        self.url.push_str(new_url);
        self.modified = current_timestamp_secs();
    }

    /// Replaces the entry's notes, zeroizing the old value and updating `modified`.
    pub fn set_notes(&mut self, new_notes: &str) {
        self.notes.zeroize();
        self.notes.push_str(new_notes);
        self.modified = current_timestamp_secs();
    }

    /// Updates the `last_used` timestamp to the current time.
    pub fn set_last_used(&mut self) {
        self.last_used = current_timestamp_secs();
    }

    fn get_byte_size(&self) -> usize {
        16 // uuid
        + 5 * 4 // len prefix for the strings so we know how many bytes to grab
        + self.name.len()
        + self.password.len()
        + self.username.len()
        + self.url.len()
        + self.notes.len()
        + 3 * 8 // timestamps
    }

    fn to_bytes(&self) -> Vec<u8> {
        let capacity = self.get_byte_size();
        let mut bytes = Vec::with_capacity(capacity);
        bytes.extend_from_slice(self.id.as_bytes());
        for s in [
            &self.name,
            &self.password,
            &self.username,
            &self.url,
            &self.notes,
        ] {
            let len: u32 = s.len().try_into().expect("entry field too long");
            bytes.extend_from_slice(&len.to_le_bytes());
            bytes.extend_from_slice(s.as_bytes());
        }
        bytes.extend_from_slice(&self.created.to_le_bytes());
        bytes.extend_from_slice(&self.modified.to_le_bytes());
        bytes.extend_from_slice(&self.last_used.to_le_bytes());
        bytes
    }

    fn from_bytes(b: &[u8]) -> Result<Self> {
        let Some((id, mut rest)) = b.split_first_chunk::<16>() else {
            bail!("entry too short for id");
        };

        // Fill an Entry in place so an early return drops (and zeroizes) the
        // fields parsed so far instead of leaving plain Strings behind.
        let mut entry = Entry {
            id: Uuid::from_bytes(*id),
            name: String::new(),
            password: String::new(),
            username: String::new(),
            url: String::new(),
            notes: String::new(),
            created: 0,
            modified: 0,
            last_used: 0,
        };
        for field in [
            &mut entry.name,
            &mut entry.password,
            &mut entry.username,
            &mut entry.url,
            &mut entry.notes,
        ] {
            let Some((len_bytes, after_len)) = rest.split_first_chunk::<4>() else {
                bail!("entry too short for string length");
            };
            let len = u32::from_le_bytes(*len_bytes) as usize;
            if after_len.len() < len {
                bail!("expected length of entry doesnt match actual length");
            }
            let (str_bytes, after_str) = after_len.split_at(len);
            rest = after_str;
            field.push_str(std::str::from_utf8(str_bytes)?);
        }

        if rest.len() < 8 * 3 {
            bail!("no enough remaining bytes for create, modified, last_used timestamps")
        }

        // safe because we check the length in the if above
        let (created, rest) = rest.split_first_chunk::<8>().unwrap();
        let (modified, rest) = rest.split_first_chunk::<8>().unwrap();
        let (last_used, _) = rest.split_first_chunk::<8>().unwrap();

        entry.created = u64::from_le_bytes(*created);
        entry.modified = u64::from_le_bytes(*modified);
        entry.last_used = u64::from_le_bytes(*last_used);
        Ok(entry)
    }
}

fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn systemtime_from_timestamp(secs: u64) -> SystemTime {
    UNIX_EPOCH
        .checked_add(Duration::from_secs(secs))
        .unwrap_or(UNIX_EPOCH)
}

#[derive(Copy, Clone)]
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

// Bytes: 4 + 4 + 16 + 24 + 16 + 12 + 1 = 77, followed by 0..=255 bytes of name
// The header is stored in plaintext (but authenticated), so the name is readable without the password.
#[derive(Clone)]
struct VaultHeader {
    magic: [u8; 4],
    version: u32,
    id: Uuid,
    nonce: [u8; NONCE_LEN],
    salt: [u8; SALT_LEN],
    kdf_params: KdfParams, // 12 bytes (3 * u32)
    name: String,          // u8 length prefix + utf-8 bytes
}

impl VaultHeader {
    fn new(name: &str, kdf_params: KdfParams) -> Result<Self> {
        if name.len() > MAX_VAULT_NAME_LEN {
            bail!("vault name too long")
        }

        let mut salt = [0u8; SALT_LEN];
        let mut nonce = [0u8; NONCE_LEN];
        getrandom::fill(&mut salt)?;
        getrandom::fill(&mut nonce)?;

        Ok(Self {
            magic: MAGIC_BYTES,
            version: VAULT_VERSION,
            id: Uuid::new_v4(),
            nonce,
            salt,
            kdf_params,
            name: name.to_owned(),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        // should never fail, the name length is checked in new() and set_name()
        let name_len: u8 = self.name.len().try_into().unwrap();

        let mut bytes = Vec::with_capacity(HEADER_FIXED_LEN + self.name.len());
        bytes.extend_from_slice(&self.magic);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(self.id.as_bytes());
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.kdf_params.to_bytes());
        bytes.push(name_len);
        bytes.extend_from_slice(self.name.as_bytes());
        bytes
    }

    /// Parses the header at the start of `b` and returns it together with the remaining bytes (the payload).
    fn from_bytes(b: &[u8]) -> Result<(Self, &[u8])> {
        let Some((fixed, rest)) = b.split_first_chunk::<HEADER_FIXED_LEN>() else {
            bail!("vault file too short");
        };

        // unwrap is save here because we know the HEADER_FIXED_LEN
        let (magic, fixed) = fixed.split_first_chunk::<4>().unwrap();
        let (version, fixed) = fixed.split_first_chunk::<4>().unwrap();
        let (id, fixed) = fixed.split_first_chunk::<16>().unwrap();
        let (nonce, fixed) = fixed.split_first_chunk::<NONCE_LEN>().unwrap();
        let (salt, fixed) = fixed.split_first_chunk::<SALT_LEN>().unwrap();
        let (kdf, fixed) = fixed.split_first_chunk::<12>().unwrap();
        let (name_len, _) = fixed.split_first_chunk::<1>().unwrap();

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
        if kdf_params.t_cost > MAX_T_COST {
            bail!("t_cost too large");
        }
        if kdf_params.p_cost > MAX_P_COST {
            bail!("p_cost too large");
        }

        let name_len = u8::from_le_bytes(*name_len) as usize;
        let Some((name, payload)) = rest.split_at_checked(name_len) else {
            bail!("vault file too short for vault name");
        };
        let name = std::str::from_utf8(name)
            .context("vault name is not valid utf-8")?
            .to_owned();

        let header = Self {
            magic: *magic,
            version,
            id: Uuid::from_bytes(*id),
            nonce: *nonce,
            salt: *salt,
            kdf_params,
            name,
        };
        Ok((header, payload))
    }
}

/// An encrypted vault containing a plaintext header and an authenticated ciphertext payload.
pub struct VaultLocked {
    header: VaultHeader,
    payload: Vec<u8>,
}

impl VaultLocked {
    /// Returns the vault's name.
    pub fn get_name(&self) -> &str {
        &self.header.name
    }

    /// Returns the vault's unique identifier.
    pub fn get_id(&self) -> &Uuid {
        &self.header.id
    }

    /// Serializes the locked vault (header followed by encrypted payload) into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Parses a locked vault from its serialized byte representation.
    pub fn from_bytes(b: &[u8]) -> Result<Self> {
        let (header, payload) = VaultHeader::from_bytes(b)?;

        Ok(Self {
            header,
            payload: payload.to_vec(),
        })
    }

    /// Writes the locked vault to `path`, or to `<data_dir>/<APP_ID>/<id>.barn` if `path` is `None`, via a temporary file, returning the written path.
    pub fn write_to_disk(&self, path: Option<&Path>) -> Result<PathBuf> {
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => data_dir()
                .context("could not find data directory")?
                .join(APP_ID)
                .join(format!("{}.barn", self.header.id)),
        };
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }

        let tmp_path = path.with_added_extension("tmp");

        std::fs::write(&tmp_path, self.to_bytes())?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(path)
    }

    /// Reads a locked vault from `path`.
    pub fn read_from_disk(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes)
    }
}

/// A decrypted vault holding its header and plaintext entries in memory.
pub struct VaultUnlocked {
    header: VaultHeader,
    entries: Vec<Entry>,
}

impl VaultUnlocked {
    /// Creates a new empty unlocked vault with the given name and default KDF parameters.
    /// Fails if `name` exceeds `MAX_VAULT_NAME_LEN` bytes.
    pub fn new(name: &str) -> Result<Self> {
        Ok(Self {
            header: VaultHeader::new(name, KdfParams::default())?,
            entries: Vec::new(),
        })
    }

    /// Returns the vault's name.
    pub fn get_name(&self) -> &str {
        &self.header.name
    }

    /// Returns the vault's unique identifier.
    pub fn get_id(&self) -> &Uuid {
        &self.header.id
    }

    /// Updates the vault's name. Fails if `new_name` exceeds `MAX_VAULT_NAME_LEN` bytes.
    pub fn set_name(&mut self, new_name: &str) -> Result<()> {
        if new_name.len() > MAX_VAULT_NAME_LEN {
            bail!("vault name to long")
        }
        self.header.name.clear();
        self.header.name.push_str(new_name);
        Ok(())
    }

    /// Return all entries of a vault as a slice.
    pub fn get_entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Return an entry ref based on its Uuid.
    pub fn get_entry(&self, id: &Uuid) -> Option<&Entry> {
        self.entries.iter().find(|e| e.get_id() == id)
    }

    /// Return an entry mutable ref based on its Uuid.
    pub fn get_entry_mut(&mut self, id: &Uuid) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|e| e.get_id() == id)
    }

    /// Add an entry to the unlocked vault.
    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Remove an entry from the unlocked vault.
    /// Returns None if the entry is not in the vault.
    pub fn remove_entry(&mut self, id: &Uuid) -> Option<Entry> {
        let index = self.entries.iter().position(|e| e.get_id() == id)?;
        Some(self.entries.remove(index))
    }
}

fn derive_key(password: &[u8], salt: &[u8], p: KdfParams) -> Result<Zeroizing<[u8; KEY_LEN]>> {
    let params = Params::new(p.m_cost, p.t_cost, p.p_cost, Some(KEY_LEN))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = Zeroizing::new([0u8; KEY_LEN]);
    argon2.hash_password_into(password, salt, &mut key[..])?;
    Ok(key)
}

/// Encrypts the entries into a new locked vault.
/// A fresh salt and nonce are generated on every call, so a nonce is never reused with the same key.
pub fn lock_vault(password: &[u8], vault_unlocked: &VaultUnlocked) -> Result<VaultLocked> {
    // Keep id, name and kdf params of the vault, only salt and nonce are renewed
    let mut header = vault_unlocked.header.clone();
    getrandom::fill(&mut header.salt)?;
    getrandom::fill(&mut header.nonce)?;

    let key = derive_key(password, &header.salt, header.kdf_params)?;
    let cipher = XChaCha20Poly1305::new_from_slice(&key[..])?;
    drop(key); // explicit drop

    // Each entry is prefixed with its u32 length so unlock_vault can split them again.
    // Capacity is reserved up front so no unzeroized copies are left behind by reallocations.
    let entries = &vault_unlocked.entries;
    let capacity = entries.iter().map(|e| 4 + e.get_byte_size()).sum();
    let mut plaintext = Zeroizing::new(Vec::with_capacity(capacity));
    for entry in entries {
        let bytes = Zeroizing::new(entry.to_bytes());
        let len: u32 = bytes.len().try_into()?;
        plaintext.extend_from_slice(&len.to_le_bytes());
        plaintext.extend_from_slice(&bytes);
    }

    // The header is authenticated as associated data, so tampering with it makes decryption fail
    let header_bytes = header.to_bytes();
    let payload = cipher.encrypt(
        &XNonce::from(header.nonce),
        Payload {
            msg: &plaintext,
            aad: &header_bytes,
        },
    )?;

    Ok(VaultLocked { header, payload })
}

/// Decrypts a locked vault. Fails on a wrong password or a modified vault.
pub fn unlock_vault(password: &[u8], vault_locked: &VaultLocked) -> Result<VaultUnlocked> {
    let header = &vault_locked.header;
    let key = derive_key(password, &header.salt, header.kdf_params)?;
    let cipher = XChaCha20Poly1305::new_from_slice(&key[..])?;
    drop(key); // explicit drop

    let header_bytes = header.to_bytes();
    let plaintext = Zeroizing::new(
        cipher
            .decrypt(
                &XNonce::from(header.nonce),
                Payload {
                    msg: &vault_locked.payload,
                    aad: &header_bytes,
                },
            )
            .context("wrong password or corrupted vault")?,
    );

    // Read back the u32 length-prefixed entries written by lock_vault
    let mut entries = Vec::new();
    let mut rest = &plaintext[..];
    while !rest.is_empty() {
        let Some((len, after_len)) = rest.split_first_chunk::<4>() else {
            bail!("vault too short for entry length");
        };
        let len = u32::from_le_bytes(*len) as usize;
        let Some((entry, after_entry)) = after_len.split_at_checked(len) else {
            bail!("entry length exceeds vault size");
        };
        entries.push(Entry::from_bytes(entry)?);
        rest = after_entry;
    }

    Ok(VaultUnlocked {
        header: header.clone(),
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_timestamp_roundtrips_normal_values() {
        let t = systemtime_from_timestamp(1_700_000_000);
        assert_eq!(
            t.duration_since(UNIX_EPOCH).unwrap().as_secs(),
            1_700_000_000
        );
    }

    #[test]
    fn from_timestamp_does_not_panic_on_overflow() {
        assert_eq!(systemtime_from_timestamp(u64::MAX), UNIX_EPOCH);
    }

    fn header() -> VaultHeader {
        VaultHeader {
            magic: MAGIC_BYTES,
            version: VAULT_VERSION,
            id: Uuid::new_v4(),
            nonce: [7; NONCE_LEN],
            salt: [9; SALT_LEN],
            kdf_params: KdfParams {
                m_cost: 1024,
                t_cost: 5,
                p_cost: 6,
            },
            name: "Personal".to_owned(),
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
        assert_eq!(bytes.len(), HEADER_FIXED_LEN + "Personal".len());
        assert_eq!(&bytes[..4], b"KUHV");
        assert_eq!(bytes[HEADER_FIXED_LEN - 1], 8);
        assert_eq!(&bytes[HEADER_FIXED_LEN..], b"Personal");
    }

    #[test]
    fn header_roundtrip() {
        let h = header();
        let bytes = h.to_bytes();
        let (r, rest) = VaultHeader::from_bytes(&bytes).unwrap();
        assert_eq!(r.magic, h.magic);
        assert_eq!(r.version, h.version);
        assert_eq!(r.nonce, h.nonce);
        assert_eq!(r.salt, h.salt);
        assert_eq!(r.kdf_params.m_cost, h.kdf_params.m_cost);
        assert_eq!(r.name, h.name);
        assert!(rest.is_empty());
    }

    #[test]
    fn header_empty_name_roundtrip() {
        let mut h = header();
        h.name.clear();
        let bytes = h.to_bytes();
        assert_eq!(bytes.len(), HEADER_FIXED_LEN);
        let (r, _) = VaultHeader::from_bytes(&bytes).unwrap();
        assert_eq!(r.name, "");
    }

    #[test]
    fn header_name_with_multibyte_chars_roundtrip() {
        let mut h = header();
        h.name = "Kühe 🐄".to_owned();
        let (r, _) = VaultHeader::from_bytes(&h.to_bytes()).unwrap();
        assert_eq!(r.name, "Kühe 🐄");
    }

    #[test]
    fn header_rejects_name_longer_than_file() {
        let bytes = header().to_bytes();
        assert!(VaultHeader::from_bytes(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn header_rejects_invalid_utf8_name() {
        let mut bytes = header().to_bytes();
        bytes[HEADER_FIXED_LEN] = 0xFF;
        assert!(VaultHeader::from_bytes(&bytes).is_err());
    }

    #[test]
    fn header_new_accepts_max_name_len() {
        let name = "a".repeat(MAX_VAULT_NAME_LEN);
        assert!(VaultHeader::new(&name, KdfParams::default()).is_ok());
    }

    #[test]
    fn header_new_rejects_too_long_name() {
        let name = "a".repeat(MAX_VAULT_NAME_LEN + 1);
        assert!(VaultHeader::new(&name, KdfParams::default()).is_err());
    }

    #[test]
    fn set_name_rejects_too_long_name_and_keeps_old_one() {
        let mut v = VaultUnlocked::new("Personal").unwrap();
        assert!(v.set_name(&"a".repeat(MAX_VAULT_NAME_LEN + 1)).is_err());
        assert_eq!(v.get_name(), "Personal");
        v.set_name("Work").unwrap();
        assert_eq!(v.get_name(), "Work");
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
    fn header_rejects_too_large_t_cost() {
        let mut h = header();
        h.kdf_params.t_cost = MAX_T_COST + 1;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_err());
    }

    #[test]
    fn header_accepts_max_t_cost() {
        let mut h = header();
        h.kdf_params.t_cost = MAX_T_COST;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_ok());
    }

    #[test]
    fn header_rejects_too_large_p_cost() {
        let mut h = header();
        h.kdf_params.p_cost = MAX_P_COST + 1;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_err());
    }

    #[test]
    fn header_accepts_max_p_cost() {
        let mut h = header();
        h.kdf_params.p_cost = MAX_P_COST;
        assert!(VaultHeader::from_bytes(&h.to_bytes()).is_ok());
    }

    #[test]
    fn vault_roundtrip() {
        let h = header();
        let expected_len = HEADER_FIXED_LEN + h.name.len() + 5;
        let v = VaultLocked {
            header: h,
            payload: vec![1, 2, 3, 4, 5],
        };
        let bytes = v.to_bytes();
        assert_eq!(bytes.len(), expected_len);
        let r = VaultLocked::from_bytes(&bytes).unwrap();
        assert_eq!(r.payload, v.payload);
        assert_eq!(r.get_name(), "Personal");
        assert_eq!(r.header.nonce, v.header.nonce);
        assert_eq!(r.header.salt, v.header.salt);
    }

    #[test]
    fn vault_empty_payload() {
        let h = header();
        let v = VaultLocked {
            header: h,
            payload: vec![],
        };
        let r = VaultLocked::from_bytes(&v.to_bytes()).unwrap();
        assert!(r.payload.is_empty());
    }

    #[test]
    fn vault_too_short() {
        assert!(VaultLocked::from_bytes(&[]).is_err());
        assert!(VaultLocked::from_bytes(&[0; HEADER_FIXED_LEN - 1]).is_err());
    }

    #[test]
    fn vault_bad_header_propagates() {
        assert!(VaultLocked::from_bytes(&[0; HEADER_FIXED_LEN]).is_err());
    }

    fn unlocked_vault() -> VaultUnlocked {
        let mut h = header();
        h.kdf_params = KdfParams {
            m_cost: 8,
            t_cost: 1,
            p_cost: 1,
        };
        VaultUnlocked {
            header: h,
            entries: vec![
                Entry::new("Reddit", "hunter2", "muh", "kuh.de", ""),
                Entry::new("Facebook", "pw", "", "", "notes"),
            ],
        }
    }

    #[test]
    fn lock_unlock_roundtrip() {
        let v = unlocked_vault();
        let locked = lock_vault(b"master", &v).unwrap();
        let r = unlock_vault(b"master", &locked).unwrap();
        assert_eq!(r.entries.len(), v.entries.len());
        for (a, b) in r.entries.iter().zip(&v.entries) {
            assert_eq!(a.get_name(), b.get_name());
            assert_eq!(a.to_bytes(), b.to_bytes());
        }
    }

    #[test]
    fn lock_unlock_roundtrip_through_bytes() {
        let v = unlocked_vault();
        let bytes = lock_vault(b"master", &v).unwrap().to_bytes();
        let locked = VaultLocked::from_bytes(&bytes).unwrap();
        assert_eq!(locked.get_name(), "Personal");
        let r = unlock_vault(b"master", &locked).unwrap();
        assert_eq!(r.get_name(), "Personal");
        assert_eq!(r.entries[0].to_bytes(), v.entries[0].to_bytes());
    }

    #[test]
    fn lock_unlock_empty_vault() {
        let mut v = unlocked_vault();
        v.entries.clear();
        let locked = lock_vault(b"master", &v).unwrap();
        assert!(unlock_vault(b"master", &locked).unwrap().entries.is_empty());
    }

    #[test]
    fn unlock_rejects_wrong_password() {
        let locked = lock_vault(b"master", &unlocked_vault()).unwrap();
        assert!(unlock_vault(b"wrong", &locked).is_err());
    }

    #[test]
    fn unlock_rejects_modified_payload() {
        let mut locked = lock_vault(b"master", &unlocked_vault()).unwrap();
        locked.payload[0] ^= 1;
        assert!(unlock_vault(b"master", &locked).is_err());
    }

    #[test]
    fn lock_vault_uses_fresh_salt_and_nonce() {
        let v = unlocked_vault();
        let a = lock_vault(b"master", &v).unwrap();
        let b = lock_vault(b"master", &v).unwrap();
        assert_ne!(a.header.salt, b.header.salt);
        assert_ne!(a.header.nonce, b.header.nonce);
        assert_ne!(a.payload, b.payload);
    }

    #[test]
    fn unlock_rejects_modified_header() {
        let mut locked = lock_vault(b"master", &unlocked_vault()).unwrap();
        locked.header.version += 1;
        assert!(unlock_vault(b"master", &locked).is_err());
    }

    #[test]
    fn unlock_rejects_modified_name() {
        let mut locked = lock_vault(b"master", &unlocked_vault()).unwrap();
        locked.header.name = "Evil".to_owned();
        assert!(unlock_vault(b"master", &locked).is_err());
    }
}
