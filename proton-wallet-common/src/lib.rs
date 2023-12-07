mod keys;

// pub mod
pub mod account;
pub mod bitcoin;
pub mod client;
pub mod error;
pub mod mnemonic;
pub mod transaction_builder;
pub mod wallet;

pub use bdk::{
    bitcoin::{
        bip32::{DerivationPath, ExtendedPrivKey},
        blockdata::locktime::absolute::{Height, LockTime, Time},
        psbt::PartiallySignedTransaction,
        Address, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    chain::{ChainPosition, ConfirmationTime, ConfirmationTimeAnchor},
    keys::{
        bip39::{Language, Mnemonic, MnemonicWithPassphrase, WordCount},
        DerivableKey, ExtendedKey,
    },
    wallet::{tx_builder::ChangeSpendPolicy, AddressIndex, AddressInfo, Balance},
    KeychainKind, LocalUtxo, SignOptions,
};

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
