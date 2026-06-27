use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretHandle {
    pub provider: String,
    pub ciphertext: String,
    pub key_version: u32,
}

impl SecretHandle {
    pub fn empty(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            ciphertext: String::new(),
            key_version: 1,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.ciphertext.is_empty()
    }
}

pub fn seal_secret(provider: &str, plaintext: &str) -> SecretHandle {
    if plaintext.trim().is_empty() {
        return SecretHandle::empty(provider);
    }

    let master_key = master_key_bytes();
    let encrypted = xor_bytes(plaintext.as_bytes(), &master_key);

    SecretHandle {
        provider: provider.to_string(),
        ciphertext: hex_encode(&encrypted),
        key_version: 1,
    }
}

pub fn open_secret(handle: &SecretHandle) -> String {
    if handle.ciphertext.is_empty() {
        return String::new();
    }

    let Ok(bytes) = hex_decode(&handle.ciphertext) else {
        return String::new();
    };
    String::from_utf8(xor_bytes(&bytes, &master_key_bytes())).unwrap_or_default()
}

fn master_key_bytes() -> Vec<u8> {
    env::var("ATLAS_VAULT_MASTER_KEY")
        .unwrap_or_else(|_| "atlas-dev-vault-key-change-before-production".to_string())
        .into_bytes()
}

fn xor_bytes(bytes: &[u8], key: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ key[index % key.len()])
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn hex_decode(value: &str) -> Result<Vec<u8>, ()> {
    if !value.len().is_multiple_of(2) {
        return Err(());
    }

    (0..value.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).map_err(|_| ()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{open_secret, seal_secret};

    #[test]
    fn sealed_secret_round_trips_without_exposing_plaintext() {
        let handle = seal_secret("torbox", "secret-token");

        assert!(handle.is_configured());
        assert_ne!(handle.ciphertext, "secret-token");
        assert_eq!(open_secret(&handle), "secret-token");
    }
}
