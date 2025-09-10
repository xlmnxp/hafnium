use base58::FromBase58;
use p256::{ecdh::diffie_hellman, elliptic_curve::{ecdh::SharedSecret, PublicKey}, FieldBytes, NistP256, NonZeroScalar};
use rand_core::OsRng;
use std::convert::TryInto;
use crate::config_manager;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit}, XChaCha20Poly1305, Key, XNonce
};

pub struct EphemeralSecret {
    scalar: NonZeroScalar,
}

impl EphemeralSecret {
    fn new() -> Self {
        let scalar = NonZeroScalar::random(&mut OsRng);
        EphemeralSecret { scalar }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        FieldBytes::from(&self.scalar).as_slice().try_into().expect("Cannot convert scalar to array")
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let scalar = NonZeroScalar::from_repr(*FieldBytes::from_slice(&bytes)).expect("Cannot convert array to scalar");
        EphemeralSecret { scalar }
    }

    fn public_key(&self) -> PublicKey<NistP256> {
        PublicKey::from_secret_scalar(&self.scalar)
    }

    fn diffie_hellman(&self, public_key: &PublicKey<NistP256>) -> SharedSecret<NistP256> {
        diffie_hellman(self.scalar, public_key.as_affine())
    }
}

pub fn generate_key_pair() -> EphemeralSecret {
    EphemeralSecret::new()
}

pub fn get_private_key() -> EphemeralSecret {
    let private_key_base58 = config_manager::get_value("private_key");
    let private_key_bytes = private_key_base58.as_str().unwrap().from_base58().unwrap();
    EphemeralSecret::from_bytes(private_key_bytes.as_slice())
}

pub fn get_public_key() -> PublicKey<NistP256> {
    get_private_key().public_key()
}

pub fn get_shared_secret(public_key: &PublicKey<NistP256>) -> [u8; 32] {
    get_private_key().diffie_hellman(&public_key).raw_secret_bytes().as_slice().try_into().expect("Cannot convert shared secret to array")
}

pub async fn encrypt(data: &[u8], shared_secret: &[u8; 32]) -> anyhow::Result<Vec<u8>, anyhow::Error> {
    let key = Key::from_slice(shared_secret);

    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng); // 192-bit nonce for XChaCha20Poly1305
    Ok([nonce.as_slice(), &cipher.encrypt(&nonce, data).unwrap()].concat())
}

pub async fn decrypt(data: &[u8], shared_secret: &[u8; 32]) -> anyhow::Result<Vec<u8>, anyhow::Error> {
    let key = Key::from_slice(shared_secret);

    let cipher = XChaCha20Poly1305::new(&key);
    let nonce = XNonce::from_slice(&data[0..24]);
    Ok(cipher.decrypt(&nonce, &data[24..]).unwrap())
}