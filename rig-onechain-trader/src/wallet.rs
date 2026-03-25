use fastcrypto::ed25519::Ed25519KeyPair;
use fastcrypto::traits::KeyPair;
use onechain_sdk::types::base_types::SuiAddress;

pub fn get_public_key(keypair: &Ed25519KeyPair) -> SuiAddress {
    SuiAddress::from(&keypair.public())
} 