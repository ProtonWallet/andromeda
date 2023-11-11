use bdk::bitcoin::address::{NetworkChecked, NetworkUnchecked};
use bdk::bitcoin::blockdata::script::ScriptBuf as BdkScriptBuf;
use bdk::bitcoin::consensus::Decodable;
use bdk::bitcoin::psbt::PartiallySignedTransaction as BdkPartiallySignedTransaction;
use bdk::bitcoin::Address as BdkAddress;
use bdk::bitcoin::Network;
use bdk::bitcoin::Transaction as BdkTransaction;
use bdk::Error as BdkError;

use std::io::Cursor;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// A Bitcoin script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script(pub(crate) BdkScriptBuf);

impl Script {
    pub fn new(raw_output_script: Vec<u8>) -> Self {
        let script: BdkScriptBuf = raw_output_script.into();
        Script(script)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl From<BdkScriptBuf> for Script {
    fn from(script: BdkScriptBuf) -> Self {
        Script(script)
    }
}

/// A Bitcoin address.
#[derive(Debug, PartialEq, Eq)]
pub struct Address {
    inner: BdkAddress<NetworkChecked>,
}

impl Address {
    pub fn new(address: String, network: Network) -> Result<Self, BdkError> {
        Ok(Address {
            inner: address
                .parse::<bdk::bitcoin::Address<NetworkUnchecked>>()
                .unwrap() // TODO 11: Handle error correctly by rethrowing it as a BdkError
                .require_network(network.into())
                .map_err(|e| BdkError::Generic(e.to_string()))?,
        })
    }

    /// alternative constructor
    // fn from_script(script: Arc<Script>, network: Network) -> Result<Self, BdkError> {
    //     BdkAddress::from_script(&script.inner, network)
    //         .map(|a| Address { inner: a })
    //         .map_err(|e| BdkError::Generic(e.to_string()))
    // }
    //
    // fn payload(&self) -> Payload {
    //     match &self.inner.payload.clone() {
    //         BdkPayload::PubkeyHash(pubkey_hash) => Payload::PubkeyHash {
    //             pubkey_hash: pubkey_hash.to_vec(),
    //         },
    //         BdkPayload::ScriptHash(script_hash) => Payload::ScriptHash {
    //             script_hash: script_hash.to_vec(),
    //         },
    //         BdkPayload::WitnessProgram { version, program } => Payload::WitnessProgram {
    //             version: *version,
    //             program: program.clone(),
    //         },
    //     }
    // }

    pub fn network(&self) -> Network {
        self.inner.network.into()
    }

    pub fn script_pubkey(&self) -> Arc<Script> {
        Arc::new(Script(self.inner.script_pubkey()))
    }

    pub fn to_qr_uri(&self) -> String {
        self.inner.to_qr_uri()
    }

    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}

impl From<Address> for BdkAddress {
    fn from(address: Address) -> Self {
        address.inner
    }
}

impl From<BdkAddress> for Address {
    fn from(address: BdkAddress) -> Self {
        Address { inner: address }
    }
}

/// A Bitcoin transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    inner: BdkTransaction,
}

impl Transaction {
    pub fn new(transaction_bytes: Vec<u8>) -> Result<Self, BdkError> {
        let mut decoder = Cursor::new(transaction_bytes);
        let tx: BdkTransaction =
            BdkTransaction::consensus_decode(&mut decoder).map_err(|e| BdkError::Generic(e.to_string()))?;
        Ok(Transaction { inner: tx })
    }

    pub fn txid(&self) -> String {
        self.inner.txid().to_string()
    }

    // fn weight(&self) -> u64 {
    //     self.inner.weight() as u64
    // }

    pub fn size(&self) -> u64 {
        self.inner.size() as u64
    }

    pub fn vsize(&self) -> u64 {
        self.inner.vsize() as u64
    }

    // fn serialize(&self) -> Vec<u8> {
    //     self.inner.serialize()
    // }

    pub fn is_coin_base(&self) -> bool {
        self.inner.is_coin_base()
    }

    pub fn is_explicitly_rbf(&self) -> bool {
        self.inner.is_explicitly_rbf()
    }

    pub fn is_lock_time_enabled(&self) -> bool {
        self.inner.is_lock_time_enabled()
    }

    pub fn version(&self) -> i32 {
        self.inner.version
    }

    // fn lock_time(&self) -> u32 {
    //     self.inner.lock_time.0
    // }

    // fn input(&self) -> Vec<TxIn> {
    //     self.inner.input.iter().map(|x| x.into()).collect()
    // }
    //
    // fn output(&self) -> Vec<TxOut> {
    //     self.inner.output.iter().map(|x| x.into()).collect()
    // }
    pub fn get_inner(&self) -> BdkTransaction {
        self.inner.clone()
    }
}

impl From<BdkTransaction> for Transaction {
    fn from(tx: BdkTransaction) -> Self {
        Transaction { inner: tx }
    }
}

pub struct PartiallySignedTransaction {
    pub inner: Mutex<BdkPartiallySignedTransaction>,
}

impl PartiallySignedTransaction {
    pub fn new(psbt_base64: String) -> Result<Self, BdkError> {
        let psbt: BdkPartiallySignedTransaction =
            BdkPartiallySignedTransaction::from_str(&psbt_base64).map_err(|e| BdkError::Generic(e.to_string()))?;

        Ok(PartiallySignedTransaction {
            inner: Mutex::new(psbt),
        })
    }

    pub fn serialize(&self) -> String {
        let psbt = self.inner.lock().unwrap().clone();
        psbt.to_string()
    }

    // pub(crate) fn txid(&self) -> String {
    //     let tx = self.inner.lock().unwrap().clone().extract_tx();
    //     let txid = tx.txid();
    //     txid.to_hex()
    // }

    /// Return the transaction.
    pub fn extract_tx(&self) -> Arc<Transaction> {
        let tx = self.inner.lock().unwrap().clone().extract_tx();
        Arc::new(tx.into())
    }

    // /// Combines this PartiallySignedTransaction with other PSBT as described by BIP 174.
    // ///
    // /// In accordance with BIP 174 this function is commutative i.e., `A.combine(B) == B.combine(A)`
    // pub(crate) fn combine(
    //     &self,
    //     other: Arc<PartiallySignedTransaction>,
    // ) -> Result<Arc<PartiallySignedTransaction>, BdkError> {
    //     let other_psbt = other.inner.lock().unwrap().clone();
    //     let mut original_psbt = self.inner.lock().unwrap().clone();
    //
    //     original_psbt.combine(other_psbt)?;
    //     Ok(Arc::new(PartiallySignedTransaction {
    //         inner: Mutex::new(original_psbt),
    //     }))
    // }

    // /// The total transaction fee amount, sum of input amounts minus sum of output amounts, in Sats.
    // /// If the PSBT is missing a TxOut for an input returns None.
    // pub(crate) fn fee_amount(&self) -> Option<u64> {
    //     self.inner.lock().unwrap().fee_amount()
    // }

    // /// The transaction's fee rate. This value will only be accurate if calculated AFTER the
    // /// `PartiallySignedTransaction` is finalized and all witness/signature data is added to the
    // /// transaction.
    // /// If the PSBT is missing a TxOut for an input returns None.
    // pub(crate) fn fee_rate(&self) -> Option<Arc<FeeRate>> {
    //     self.inner.lock().unwrap().fee_rate().map(Arc::new)
    // }

    // /// Serialize the PSBT data structure as a String of JSON.
    // pub(crate) fn json_serialize(&self) -> String {
    //     let psbt = self.inner.lock().unwrap();
    //     serde_json::to_string(psbt.deref()).unwrap()
    // }
}

impl From<BdkPartiallySignedTransaction> for PartiallySignedTransaction {
    fn from(psbt: BdkPartiallySignedTransaction) -> Self {
        PartiallySignedTransaction {
            inner: Mutex::new(psbt),
        }
    }
}
