extern crate base58;
extern crate toml;

use p256::CompressedPoint;
use toml::Value;

use crate::encryption::{self, generate_key_pair};

const FILE_NAME: &str = "hafnium.toml";

fn read_config() -> toml::Value {
    let config = std::fs::read_to_string(FILE_NAME).unwrap();
    let value = toml::from_str(&config).unwrap();
    value
}

pub fn generate_key_pair_and_save() {
    if !std::path::Path::new(FILE_NAME).exists() {
        let private_key_raw = generate_key_pair();
        let private_key =
            base58::ToBase58::to_base58(private_key_raw.to_bytes().as_slice()).to_string();

        std::fs::write(
            FILE_NAME,
            format!(
                "private_key = \"{}\"
trusted_peers = []",
                private_key
            ),
        )
        .unwrap();
        println!("Key pair generated and saved");
    } else {
        println!("Key pair already exists");
    }
}

pub fn get_value(key: &str) -> toml::Value {
    let config = read_config();
    config.get(key).unwrap().clone()
}

pub fn get_public_key() -> String {
    let public_key = encryption::get_public_key();
    base58::ToBase58::to_base58(CompressedPoint::from(public_key).as_slice()).to_string()
}

pub fn set_value(key: &str, value: &str) {
    let mut config = read_config();
    config
        .as_table_mut()
        .unwrap()
        .insert(key.to_string(), toml::Value::String(value.to_string()));

    std::fs::write(FILE_NAME, toml::to_string(&config).unwrap()).unwrap();
}

pub fn insert_trusted_peer(peer: &str) {
    let mut config = read_config();

    let trusted_peers = config.get_mut("trusted_peers").unwrap().as_array_mut().unwrap();
    trusted_peers.push(Value::String(peer.to_string()));

    std::fs::write(FILE_NAME, toml::to_string(&config).unwrap()).unwrap();
}

pub fn get_trusted_peers() -> Vec<String> {
    let config = read_config();
    let trusted_peers = config.get("trusted_peers").unwrap().as_array().unwrap();
    trusted_peers.iter().map(|v| v.as_str().unwrap().to_string()).collect()
}

pub fn remove_trusted_peer(peer: &str) {
    let mut config = read_config();

    let trusted_peers = config.get_mut("trusted_peers").unwrap().as_array_mut().unwrap();
    trusted_peers.retain(|v| v.as_str().unwrap() != peer);

    std::fs::write(FILE_NAME, toml::to_string(&config).unwrap()).unwrap();
}