use anyhow::{Result, Error};
use hmac::{Hmac, Mac};
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{KeyIvInit, BlockDecryptMut};
use sha2::{Digest};
use crate::types::Aes128Key;

const MAC_SIZE: usize = 32;

pub struct SubKeys {
    cipher: Aes128Key,
    mac: Aes128Key,
}

impl SubKeys {
    pub fn new(key: Aes128Key) -> Self {
        let mut hasher = sha2::Sha256::new();
        hasher.update(key);
        let hash = hasher.finalize();
        let mut cipher = [0; 16];
        let mut mac = [0; 16];
        cipher.copy_from_slice(&hash[..16]);
        mac.copy_from_slice(&hash[16..]);
        SubKeys { cipher, mac }
    }
}

pub fn create_user_passphrase_key(passphrase: &str, salt: &Aes128Key) -> Aes128Key {
    let mut hasher = sha2::Sha256::new();
    hasher.update(passphrase);
    let user_passphrase_hash = hasher.finalize();
    let user_passphrase_key_source = bcrypt::bcrypt(
        8,
        salt[..16].try_into().unwrap(), // should never fail because of check in fetch salt
        &user_passphrase_hash
    );
    let mut user_passphrase_key = [0; 16];
    user_passphrase_key.copy_from_slice(&user_passphrase_key_source[..16]);
    user_passphrase_key
}

pub fn decrypt_key(key: &Aes128Key, message: &Aes128Key) -> Aes128Key {
    use aes::cipher::{KeyInit, BlockDecrypt};
    let mut output = [0; 16];
    output.copy_from_slice(message);
    let cipher = aes::Aes128::new(key.into());
    cipher.decrypt_block(output.as_mut().into());
    for byte in &mut output {
        *byte ^= 0x88;
    }
    output
}

pub fn decrypt_with_mac(sub_keys: &SubKeys, message: &[u8]) -> Result<Vec<u8>> {
    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

    if message.len() < MAC_SIZE || message.len() % 16 != 1 {
        return Err(Error::msg("message length is wrong"));
    }
    let mut mac = Hmac::<sha2::Sha256>::new_from_slice(&sub_keys.mac)?;
    let message_without_mac = &message[1..message.len() - MAC_SIZE];
    mac.update(message_without_mac);
    if mac.verify_slice(&message[message.len() - MAC_SIZE..]).is_err() {
        return Err(Error::msg("message could not be verified"));
    }

    let msg = Aes128CbcDec::new(sub_keys.cipher[..].into(), message_without_mac[..16].into())
        .decrypt_padded_vec_mut::<Pkcs7>(&message_without_mac[16..])?;

    Ok(msg)
}
