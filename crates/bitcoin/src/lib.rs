//! Bitcoin wallet functionality for Andromeda
//!
//! This crate provides Bitcoin wallet operations including account management,
//! transaction handling, and message signing.

// Core modules
pub mod account;
pub mod account_sweeper;
pub mod account_syncer;
pub mod account_trait;
pub mod address;
pub mod bdk_wallet_ext;
pub mod bdk_wallet_secp_ext;
pub mod bitcoin_signed_message;
pub mod blockchain_client;
pub mod error;
pub mod message_signer;
pub mod mnemonic;
pub mod paper_account;
pub mod payment_link;
pub mod psbt;
pub mod storage;
pub mod transaction_builder;
pub mod transactions;
pub mod utils;
pub mod wallet;

// Re-export commonly used types
pub use andromeda_crypto::message_signature::SigningType;
pub use utils::{SortOrder, TransactionFilter};

#[cfg(test)]
pub mod tests {
    pub mod test_utils;
    pub mod utils;
}

// Type alias for common result type
type Result<T> = std::result::Result<T, error::Error>;

// SQLite feature
#[cfg(feature = "sqlite")]
pub use bdk_wallet::rusqlite::Connection;

// Re-export BDK types
#[doc(hidden)]
pub use bdk_wallet::{
    bitcoin::{
        bip32::{ChildNumber, DerivationPath, Xpriv},
        block::Header as BlockHeader,
        blockdata::{
            constants::genesis_block,
            locktime::absolute::{Height, LockTime, Time},
        },
        consensus::Params as ConsensusParams,
        secp256k1::Secp256k1,
        Address, Amount, BlockHash, Network as BdkNetwork, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
        Witness,
    },
    chain::ConfirmationBlockTime,
    keys::{
        bip39::{Language as BdkLanguage, Mnemonic as BdkMnemonic, MnemonicWithPassphrase, WordCount},
        DerivableKey, ExtendedKey,
    },
    tx_builder::ChangeSpendPolicy,
    AddressInfo, Balance, ChangeSet, KeychainKind, LocalOutput, SignOptions,
};
