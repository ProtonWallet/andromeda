// pub mod
pub mod keys;
pub mod mnemonic;
// pub mod transaction;
pub mod bitcoin;
pub mod descriptor;
pub mod account;
pub mod error;
pub mod wallet;

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// pub use bdk::keys::bip39::WordCount;
// pub use bdk::KeychainKind;

pub use crate::bitcoin::Address;
pub use crate::bitcoin::PartiallySignedTransaction;
pub use crate::bitcoin::Script;
pub use crate::bitcoin::Transaction;
pub use crate::descriptor::Descriptor;
pub use self::keys::DescriptorPublicKey;
pub use crate::keys::DescriptorSecretKey;
pub use crate::wallet::TxBuilder;
pub use crate::wallet::Update;
pub use crate::wallet::Wallet;

pub use bdk::bitcoin::blockdata::locktime::absolute::Height;
pub use bdk::bitcoin::blockdata::locktime::absolute::LockTime;
pub use bdk::bitcoin::blockdata::locktime::absolute::Time;
pub use bdk::keys::bip39::WordCount;
pub use bdk::wallet::AddressIndex as BdkAddressIndex;
pub use bdk::wallet::AddressInfo as BdkAddressInfo;
pub use bdk::bitcoin::Network as BdkNetwork;
pub use bdk::wallet::Balance as BdkBalance;
pub use bdk::Error as BdkError;
pub use bdk::KeychainKind;
pub use bdk::Wallet as BdkWallet;
use std::sync::Arc;

/// A output script and an amount of satoshis.
// pub struct ScriptAmount {
//     pub script: Arc<Script>,
//     pub amount: u64,
// }

/// A derived address and the index it was found at.
pub struct AddressInfo {
    /// Child index of this address.
    pub index: u32,
    /// Address.
    pub address: Arc<Address>,
    /// Type of keychain.
    pub keychain: KeychainKind,
}

impl From<BdkAddressInfo> for AddressInfo {
    fn from(address_info: BdkAddressInfo) -> Self {
        AddressInfo {
            index: address_info.index,
            address: Arc::new(address_info.address.into()),
            keychain: address_info.keychain,
        }
    }
}

/// The address index selection strategy to use to derived an address from the wallet's external
/// descriptor.
pub enum AddressIndex {
    /// Return a new address after incrementing the current descriptor index.
    New,
    /// Return the address for the current descriptor index if it has not been used in a received
    /// transaction. Otherwise return a new address as with AddressIndex::New.
    /// Use with caution, if the wallet has not yet detected an address has been used it could
    /// return an already used address. This function is primarily meant for situations where the
    /// caller is untrusted; for example when deriving donation addresses on-demand for a public
    /// web page.
    LastUnused,
    /// Return the address for a specific descriptor index. Does not change the current descriptor
    /// index used by `AddressIndex::New` and `AddressIndex::LastUsed`.
    /// Use with caution, if an index is given that is less than the current descriptor index
    /// then the returned address may have already been used.
    Peek { index: u32 },
}

impl From<AddressIndex> for BdkAddressIndex {
    fn from(address_index: AddressIndex) -> Self {
        match address_index {
            AddressIndex::New => BdkAddressIndex::New,
            AddressIndex::LastUnused => BdkAddressIndex::LastUnused,
            AddressIndex::Peek { index } => BdkAddressIndex::Peek(index),
        }
    }
}

// TODO 9: Peek is not correctly implemented
impl From<&AddressIndex> for BdkAddressIndex {
    fn from(address_index: &AddressIndex) -> Self {
        match address_index {
            AddressIndex::New => BdkAddressIndex::New,
            AddressIndex::LastUnused => BdkAddressIndex::LastUnused,
            AddressIndex::Peek { index } => BdkAddressIndex::Peek(*index),
        }
    }
}

impl From<BdkAddressIndex> for AddressIndex {
    fn from(address_index: BdkAddressIndex) -> Self {
        match address_index {
            BdkAddressIndex::New => AddressIndex::New,
            BdkAddressIndex::LastUnused => AddressIndex::LastUnused,
            _ => panic!("Mmmm not working"),
        }
    }
}

impl From<&BdkAddressIndex> for AddressIndex {
    fn from(address_index: &BdkAddressIndex) -> Self {
        match address_index {
            BdkAddressIndex::New => AddressIndex::New,
            BdkAddressIndex::LastUnused => AddressIndex::LastUnused,
            _ => panic!("Mmmm not working"),
        }
    }
}

// /// A wallet transaction
// #[derive(Debug, Clone, PartialEq, Eq, Default)]
// pub struct TransactionDetails {
//     pub transaction: Option<Arc<Transaction>>,
//     /// Transaction id.
//     pub txid: String,
//     /// Received value (sats)
//     /// Sum of owned outputs of this transaction.
//     pub received: u64,
//     /// Sent value (sats)
//     /// Sum of owned inputs of this transaction.
//     pub sent: u64,
//     /// Fee value (sats) if confirmed.
//     /// The availability of the fee depends on the backend. It's never None with an Electrum
//     /// Server backend, but it could be None with a Bitcoin RPC node without txindex that receive
//     /// funds while offline.
//     pub fee: Option<u64>,
//     /// If the transaction is confirmed, contains height and timestamp of the block containing the
//     /// transaction, unconfirmed transaction contains `None`.
//     pub confirmation_time: Option<BlockTime>,
// }

//
// impl From<BdkTransactionDetails> for TransactionDetails {
//     fn from(tx_details: BdkTransactionDetails) -> Self {
//         let optional_tx: Option<Arc<Transaction>> =
//             tx_details.transaction.map(|tx| Arc::new(tx.into()));
//
//         TransactionDetails {
//             transaction: optional_tx,
//             fee: tx_details.fee,
//             txid: tx_details.txid.to_string(),
//             received: tx_details.received,
//             sent: tx_details.sent,
//             confirmation_time: tx_details.confirmation_time,
//         }
//     }
// }
//
// /// A reference to a transaction output.
// #[derive(Clone, Debug, PartialEq, Eq, Hash)]
// pub struct OutPoint {
//     /// The referenced transaction's txid.
//     txid: String,
//     /// The index of the referenced output in its transaction's vout.
//     vout: u32,
// }
//
// impl From<&OutPoint> for BdkOutPoint {
//     fn from(outpoint: &OutPoint) -> Self {
//         BdkOutPoint {
//             txid: Txid::from_str(&outpoint.txid).unwrap(),
//             vout: outpoint.vout,
//         }
//     }
// }

pub struct Balance {
    pub inner: BdkBalance,
}

impl Balance {
    /// All coinbase outputs not yet matured.
    pub fn immature(&self) -> u64 {
        self.inner.immature
    }

    /// Unconfirmed UTXOs generated by a wallet tx.
    pub fn trusted_pending(&self) -> u64 {
        self.inner.trusted_pending
    }

    /// Unconfirmed UTXOs received from an external wallet.
    pub fn untrusted_pending(&self) -> u64 {
        self.inner.untrusted_pending
    }

    /// Confirmed and immediately spendable balance.
    pub fn confirmed(&self) -> u64 {
        self.inner.confirmed
    }

    /// Get sum of trusted_pending and confirmed coins.
    pub fn trusted_spendable(&self) -> u64 {
        self.inner.trusted_spendable()
    }

    /// Get the whole balance visible to the wallet.
    pub fn total(&self) -> u64 {
        self.inner.total()
    }
}
