pub mod common;

pub mod onchain;

pub use bdk::{
    bitcoin::{
        bip32::{DerivationPath, ExtendedPrivKey},
        blockdata::locktime::absolute::{Height, LockTime, Time},
        psbt::PartiallySignedTransaction,
        Address, Network as BdkNetwork, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    chain::{ChainPosition, ConfirmationTime, ConfirmationTimeHeightAnchor},
    keys::{
        bip39::{Language as BdkLanguage, Mnemonic as BdkMnemonic, MnemonicWithPassphrase, WordCount},
        DerivableKey, ExtendedKey,
    },
    wallet::{tx_builder::ChangeSpendPolicy, AddressIndex, AddressInfo, Balance, ChangeSet},
    KeychainKind, LocalOutput, SignOptions,
};

pub use bdk_chain::{Append, PersistBackend};

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
