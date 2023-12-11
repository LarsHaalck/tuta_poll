use crate::types::Aes128Key;
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use anyhow::{bail, Error, Result};
use hmac::{Hmac, Mac};
use rsa::RsaPrivateKey;
use sha2::Digest;
// use num_bigint_dig::ToPrimitive;
use num_traits::cast::{ToPrimitive, FromPrimitive};
use tracing::warn;

const MAC_SIZE: usize = 32;
const RSA_KEY_LENGTH_BITS: usize = 2048;

pub struct SubKeys {
    cipher: Aes128Key,
    mac: Option<Aes128Key>,
}

impl SubKeys {
    pub fn new(key: Aes128Key, use_mac: bool) -> Self {
        if use_mac {
            let mut hasher = sha2::Sha256::new();
            hasher.update(key);
            let hash = hasher.finalize();
            let mut cipher = [0; 16];
            let mut mac = [0; 16];
            cipher.copy_from_slice(&hash[..16]);
            mac.copy_from_slice(&hash[16..]);
            SubKeys {
                cipher,
                mac: Some(mac),
            }
        } else {
            SubKeys {
                cipher: key,
                mac: None,
            }
        }
    }
}

pub fn create_user_passphrase_key(passphrase: &str, salt: &Aes128Key) -> Aes128Key {
    let mut hasher = sha2::Sha256::new();
    hasher.update(passphrase);
    let user_passphrase_hash = hasher.finalize();
    let user_passphrase_key_source = bcrypt::bcrypt(8, *salt, &user_passphrase_hash);
    let mut user_passphrase_key = [0; 16];
    user_passphrase_key.copy_from_slice(&user_passphrase_key_source[..16]);
    user_passphrase_key
}

pub fn decrypt_key(key: &Aes128Key, message: &Aes128Key) -> Aes128Key {
    use aes::cipher::{BlockDecrypt, KeyInit};
    let mut output = [0; 16];
    output.copy_from_slice(message);
    let cipher = aes::Aes128::new(key.into());
    cipher.decrypt_block(output.as_mut().into());
    for byte in &mut output {
        *byte ^= 0x88;
    }
    output
}

pub fn aes_decrypt(key: &Aes128Key, message: &[u8]) -> Result<Vec<u8>> {
    let use_mac = message.len() % 2 == 1;
    let sub_keys = SubKeys::new(*key, use_mac);

    let message_without_mac;
    if use_mac {
        message_without_mac = &message[1..message.len() - MAC_SIZE];
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(&sub_keys.mac.unwrap())?;
        mac.update(message_without_mac);
        if mac
            .verify_slice(&message[message.len() - MAC_SIZE..])
            .is_err()
        {
            return Err(Error::msg("message could not be verified"));
        }
    } else {
        message_without_mac = &message;
    }

    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;
    let msg = Aes128CbcDec::new(sub_keys.cipher[..].into(), message_without_mac[..16].into())
        .decrypt_padded_vec_mut::<Pkcs7>(&message_without_mac[16..])?;

    Ok(msg)
}

pub fn decrypt_rsa_key(key: &Aes128Key, message: &[u8]) -> Result<RsaPrivateKey> {
    let decrypt = aes_decrypt(key, message)?;
    u8_to_key(&decrypt)
}

fn u8_to_key(enc_key: &[u8]) -> Result<RsaPrivateKey> {
    let mut key = Vec::new();
    let mut pos: usize = 0;
    while pos < enc_key.len() {
        let next_param_len =
            u16::from_be_bytes(enc_key[pos..pos + 2].try_into().unwrap()) as usize / 2;
        pos += 2;
        let value = rsa::BigUint::from_bytes_be(&enc_key[pos..pos + next_param_len]);
        key.push(value);
        pos += next_param_len;
    }

    if key.len() != 1 && key.len() != 7 {
        bail!("invalid key params");
    }

    let bit_length = key[0].bits();
    if bit_length < RSA_KEY_LENGTH_BITS - 1 || bit_length > RSA_KEY_LENGTH_BITS {
        bail!(
            "invalid key length, expected: around {}, but was: {}",
            RSA_KEY_LENGTH_BITS,
            bit_length
        )
    }

    Ok(RsaPrivateKey::from_components(
        key[0].clone(),
        rsa::BigUint::from_u32(65537).unwrap(),
        key[1].clone(),
        key[2..4].to_vec(),
    )?)
}

pub fn rsa_decrypt(key: &RsaPrivateKey, message: &[u8]) -> Result<Vec<u8>> {
    let padding = rsa::Oaep::new::<sha2::Sha256>();
    Ok(key.decrypt(padding, message)?)
}
