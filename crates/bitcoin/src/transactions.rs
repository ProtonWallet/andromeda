use bdk::{
    database::BatchDatabase, psbt::PsbtUtils, BlockTime, TransactionDetails as BdkTransactionDetails,
    Wallet as BdkWallet,
};
use bitcoin::{bip32::DerivationPath, psbt::PartiallySignedTransaction, TxIn, TxOut, Txid};
use miniscript::bitcoin::{Address, ScriptBuf};

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct TransactionDetails {
    /// Transaction id
    pub txid: Txid,
    /// Received value (sats)
    /// Sum of owned outputs of this transaction.
    pub received: u64,
    /// Sent value (sats)
    /// Sum of owned inputs of this transaction.
    pub sent: u64,
    /// Fee value (sats) if confirmed.
    /// The availability of the fee depends on the backend. It's never `None`
    /// with an Electrum Server backend, but it could be `None` with a
    /// Bitcoin RPC node without txindex that receive funds while offline.
    pub fees: Option<u64>,
    /// If the transaction is confirmed, contains height and Unix timestamp of
    /// the block containing the transaction, unconfirmed transaction
    /// contains `None`.
    pub confirmation_time: Option<BlockTime>,
    /// List of transaction inputs.
    pub inputs: Vec<TxIn>,
    /// List of transaction outputs.
    pub outputs: Vec<DetailledTxOutput>,
}

impl TransactionDetails {
    pub fn from_bdk<Storage>(value: BdkTransactionDetails, wallet: &BdkWallet<Storage>) -> Result<Self, Error>
    where
        Storage: BatchDatabase,
    {
        Ok(Self {
            txid: value.txid,
            received: value.received,
            sent: value.sent,
            fees: value.fee,
            confirmation_time: value.confirmation_time,
            inputs: value.transaction.clone().map_or(Vec::new(), |tx| tx.input),
            outputs: value.transaction.clone().map_or(Ok(Vec::new()), |tx| {
                tx.output
                    .into_iter()
                    .map(|output| DetailledTxOutput::from_txout(output, wallet))
                    .collect::<Result<Vec<_>, _>>()
            })?,
        })
    }
}

impl TransactionDetails {
    pub fn from_psbt<Storage>(psbt: &PartiallySignedTransaction, wallet: &BdkWallet<Storage>) -> Result<Self, Error>
    where
        Storage: BatchDatabase,
    {
        let tx = psbt.clone().extract_tx();

        let outputs: Vec<DetailledTxOutput> = tx
            .output
            .clone()
            .into_iter()
            .map(|output| DetailledTxOutput::from_txout(output, wallet))
            .collect::<Result<Vec<_>, _>>()?;

        let tx = TransactionDetails {
            txid: tx.txid(),
            received: 0u64,
            sent: 0u64,
            fees: psbt.fee_amount(),
            confirmation_time: None,
            inputs: tx.input.clone(),
            outputs,
        };

        Ok(tx)
    }
}

#[derive(Clone, Debug)]
pub struct SimpleTransaction {
    /// Transaction id
    pub txid: Txid,
    /// Received value (sats)
    /// Sum of owned outputs of this transaction.
    pub received: u64,
    /// Sent value (sats)
    /// Sum of owned inputs of this transaction.
    pub sent: u64,
    /// Fee value (sats) if confirmed.
    /// The availability of the fee depends on the backend. It's never `None`
    /// with an Electrum Server backend, but it could be `None` with a
    /// Bitcoin RPC node without txindex that receive funds while offline.
    pub fees: Option<u64>,
    /// If the transaction is confirmed, contains height and Unix timestamp of
    /// the block containing the transaction, unconfirmed transaction
    /// contains `None`.
    pub confirmation_time: Option<BlockTime>,
    /// Derivation of the account linked to the transaction
    pub account_key: Option<DerivationPath>,
}

impl SimpleTransaction {
    pub fn from_detailled_tx(detailled_tx: BdkTransactionDetails, account_key: Option<DerivationPath>) -> Self {
        SimpleTransaction {
            account_key,
            txid: detailled_tx.txid,
            received: detailled_tx.received,
            sent: detailled_tx.sent,
            fees: detailled_tx.fee,
            confirmation_time: detailled_tx.confirmation_time,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DetailledTxOutput {
    pub value: u64,
    pub address: Address,
    pub script_pubkey: ScriptBuf,
    pub is_mine: bool,
}

impl DetailledTxOutput {
    pub fn from_txout<Storage>(output: TxOut, wallet: &BdkWallet<Storage>) -> Result<DetailledTxOutput, Error>
    where
        Storage: BatchDatabase,
    {
        Ok(DetailledTxOutput {
            value: output.value,
            is_mine: wallet.is_mine(&output.script_pubkey)?,
            address: Address::from_script(&output.script_pubkey, wallet.network())?,
            script_pubkey: output.script_pubkey,
        })
    }
}

pub struct Pagination {
    pub skip: usize,
    pub take: usize,
}

impl Pagination {
    pub fn new(skip: usize, take: usize) -> Self {
        Pagination { skip, take }
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination::new(0, usize::MAX)
    }
}
