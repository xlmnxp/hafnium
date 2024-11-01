use base58::FromBase58;
use p256::{ecdh::diffie_hellman, elliptic_curve::{ecdh::SharedSecret, PublicKey}, FieldBytes, NistP256, NonZeroScalar};
use rand_core::OsRng;
use std::convert::TryInto;
use crate::config_manager;

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

pub fn get_public_key() -> PublicKey<NistP256> {
    let private_key_base58 = config_manager::get_value("private_key");
    let private_key_bytes = private_key_base58.as_str().unwrap().from_base58().unwrap();
    let private_key: EphemeralSecret = EphemeralSecret::from_bytes(private_key_bytes.as_slice());
    private_key.public_key()
}

pub fn get_shared_secret(private_key: &EphemeralSecret, public_key: &PublicKey<NistP256>) -> [u8; 32] {
    private_key.diffie_hellman(&public_key).raw_secret_bytes().as_slice().try_into().expect("Cannot convert shared secret to array")
}