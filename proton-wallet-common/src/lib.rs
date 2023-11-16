mod keys;

// pub mod
pub mod account;
pub mod bitcoin;
pub mod error;
pub mod mnemonic;
pub mod transaction_builder;
pub mod wallet;

pub use bdk::{
    bitcoin::{
        bip32::DerivationPath,
        blockdata::locktime::absolute::{Height, LockTime, Time},
        Transaction,
    },
    keys::bip39::{Language, WordCount},
    wallet::{AddressIndex, AddressInfo, Balance},
    KeychainKind,
};

pub use keys::new_master_private_key;

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

