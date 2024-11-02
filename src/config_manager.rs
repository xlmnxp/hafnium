extern crate toml;
extern crate base58;

use p256::CompressedPoint;

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
        let private_key = base58::ToBase58::to_base58(private_key_raw.to_bytes().as_slice()).to_string();

        std::fs::write(FILE_NAME, format!("private_key = \"{}\"", private_key)).unwrap();
        println!("Key pair generated and saved");
    } else {
        println!("Key pair already exists");
    }
}

pub fn get_value(key: &str) -> toml::Value {
    let config = read_config();
    let value = config.get(key).unwrap();
    value.clone()
}

pub fn get_public_key() -> String {
    let public_key = encryption::get_public_key();
    base58::ToBase58::to_base58(CompressedPoint::from(public_key).as_slice()).to_string()
}

pub fn set_value(key: &str, value: &str) {
    let mut config = read_config();
    config.as_table_mut().unwrap().insert(key.to_string(), toml::Value::String(value.to_string()));

    std::fs::write(FILE_NAME, toml::to_string(&config).unwrap()).unwrap();
}