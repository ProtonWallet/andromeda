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

#[cfg(test)]
pub mod tests {
    pub mod test_utils;
    pub mod utils;
}

// Define a type alias for the common result type used in this crate
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(feature = "sqlite")]
pub use bdk_wallet::rusqlite::Connection;
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

pub use andromeda_crypto::message_signature::SigningType;
