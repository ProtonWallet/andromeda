use std::{cmp::Ordering, sync::Arc};

use andromeda_common::utils::now;
use async_std::sync::RwLockReadGuard;
use bdk_chain::tx_graph::TxNode;
use bdk_wallet::{
    bitcoin::{bip32::DerivationPath, Address, ScriptBuf, Sequence, TxIn, TxOut, Txid, Witness},
    chain::{ChainPosition, ConfirmationBlockTime},
    PersistedWallet, Wallet as BdkWallet, WalletPersister, WalletTx,
};
use bitcoin::Transaction;

use crate::{account::Account, account_trait::AccessWallet, error::Error, psbt::Psbt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionTime {
    Confirmed { confirmation_time: u64 },
    Unconfirmed { last_seen: u64 },
}

impl PartialOrd for TransactionTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            TransactionTime::Unconfirmed { .. } => match other {
                TransactionTime::Unconfirmed { .. } => Some(Ordering::Equal),
                TransactionTime::Confirmed { .. } => Some(Ordering::Greater),
            },
            TransactionTime::Confirmed {
                confirmation_time: confirmation_time_a,
            } => match other {
                TransactionTime::Unconfirmed { .. } => Some(Ordering::Less),
                TransactionTime::Confirmed {
                    confirmation_time: confirmation_time_b,
                } => Some(confirmation_time_a.cmp(confirmation_time_b)),
            },
        }
    }
}

impl Ord for TransactionTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

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
    /// Transaction size in vbytes.
    /// Can be used to compute feerate for transaction given an absolute fee
    /// amount
    pub vbytes_size: u64,
    /// If the transaction is confirmed, contains height and Unix timestamp of
    /// the block containing the transaction, unconfirmed transaction
    /// contains `None`.
    pub time: TransactionTime,
    /// List of transaction inputs.
    pub inputs: Vec<DetailledTxIn>,
    /// List of transaction outputs.
    pub outputs: Vec<DetailledTxOutput>,
    /// BIP44 Account to which the transaction is bound
    pub account_derivation_path: DerivationPath,
}

fn get_detailled_inputs(txins: Vec<TxIn>, wallet: &BdkWallet) -> Result<Vec<DetailledTxIn>, Error> {
    let inputs = txins
        .clone()
        .into_iter()
        .map(|input| DetailledTxIn::from_txin(input, wallet))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(inputs)
}

fn get_detailled_outputs(txout: Vec<TxOut>, wallet: &BdkWallet) -> Result<Vec<DetailledTxOutput>, Error> {
    let outputs = txout
        .clone()
        .into_iter()
        .map(|output| DetailledTxOutput::from_txout(output, wallet))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(outputs)
}

fn get_time(chain_position: Option<ChainPosition<ConfirmationBlockTime>>) -> TransactionTime {
    if let Some(chain_position) = chain_position {
        return match chain_position {
            ChainPosition::Confirmed { anchor, .. } => TransactionTime::Confirmed {
                confirmation_time: anchor.confirmation_time,
            },
            ChainPosition::Unconfirmed { last_seen } => TransactionTime::Unconfirmed {
                last_seen: last_seen.unwrap_or(now().as_secs()),
            },
        };
    }

    TransactionTime::Unconfirmed {
        last_seen: now().as_secs(),
    }
}

pub trait ToTransactionDetails<A> {
    fn to_transaction_details(&self, account: A) -> Result<TransactionDetails, Error>;
}

impl<'a, P> ToTransactionDetails<(&RwLockReadGuard<'a, PersistedWallet<P>>, DerivationPath)> for WalletTx<'a>
where
    P: WalletPersister,
{
    fn to_transaction_details(
        &self,
        (wallet_lock, account_derivation_path): (&RwLockReadGuard<'a, PersistedWallet<P>>, DerivationPath),
    ) -> Result<TransactionDetails, Error> {
        let (sent, received) = wallet_lock.sent_and_received(&self.tx_node.tx);

        let time = get_time(Some(self.chain_position));
        let outputs = get_detailled_outputs(self.tx_node.output.clone(), wallet_lock)?;
        let inputs = get_detailled_inputs(self.tx_node.input.clone(), wallet_lock)?;

        Ok(TransactionDetails {
            txid: self.tx_node.compute_txid(),

            received: received.to_sat(),
            sent: sent.to_sat(),
            fees: wallet_lock.calculate_fee(&self.tx_node.tx).ok().map(|a| a.to_sat()),

            vbytes_size: self.tx_node.weight().to_vbytes_ceil(),
            time,

            inputs,
            outputs,

            account_derivation_path,
        })
    }
}

impl<'a, P, A> ToTransactionDetails<(&RwLockReadGuard<'a, PersistedWallet<P>>, DerivationPath)>
    for TxNode<'_, Arc<Transaction>, A>
where
    P: WalletPersister,
{
    fn to_transaction_details(
        &self,
        (wallet_lock, account_derivation_path): (&RwLockReadGuard<'a, PersistedWallet<P>>, DerivationPath),
    ) -> Result<TransactionDetails, Error> {
        let (sent, received) = wallet_lock.sent_and_received(&self.tx);
        let tx = wallet_lock
            .tx_graph()
            .list_canonical_txs(wallet_lock.local_chain(), wallet_lock.local_chain().tip().block_id())
            .find(|tx| tx.tx_node.txid == self.compute_txid());
        let time = match tx {
            Some(tx) => get_time(Some(tx.chain_position)),
            None => get_time(None),
        };

        let outputs = get_detailled_outputs(self.output.clone(), wallet_lock)?;
        let inputs = get_detailled_inputs(self.input.clone(), wallet_lock)?;

        Ok(TransactionDetails {
            txid: self.compute_txid(),

            received: received.to_sat(),
            sent: sent.to_sat(),
            fees: wallet_lock.calculate_fee(&self.tx).ok().map(|a| a.to_sat()),

            vbytes_size: self.weight().to_vbytes_ceil(),
            time,

            inputs,
            outputs,

            account_derivation_path,
        })
    }
}

impl TransactionDetails {
    pub async fn from_psbt(psbt: &Psbt, account: Arc<Account>) -> Result<Self, Error> {
        let tx = psbt.extract_tx()?;

        let wallet_lock = account.get_wallet().await;

        let outputs = get_detailled_outputs(tx.output.clone(), &wallet_lock)?;
        let inputs = get_detailled_inputs(tx.input.clone(), &wallet_lock)?;

        let (sent, received) = wallet_lock.sent_and_received(&tx);

        let tx = TransactionDetails {
            txid: tx.compute_txid(),
            received: received.to_sat(),
            sent: sent.to_sat(),

            fees: wallet_lock.calculate_fee(&tx).ok().map(|a| a.to_sat()),
            vbytes_size: tx.weight().to_vbytes_ceil(),

            time: TransactionTime::Unconfirmed {
                last_seen: now().as_secs(),
            },

            inputs,
            outputs,

            account_derivation_path: account.get_derivation_path(),
        };

        Ok(tx)
    }

    pub fn get_time(&self) -> u64 {
        match self.time {
            TransactionTime::Confirmed { confirmation_time } => confirmation_time,
            TransactionTime::Unconfirmed { last_seen } => last_seen,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DetailledTxIn {
    pub previous_output: Option<DetailledTxOutput>, // Remove option when we know why some utxo are not found
    pub script_sig: ScriptBuf,
    pub sequence: Sequence,
    pub witness: Witness,
}

impl DetailledTxIn {
    pub fn from_txin(input: TxIn, wallet: &BdkWallet) -> Result<DetailledTxIn, Error> {
        let utxo = wallet.get_utxo(input.previous_output);

        Ok(DetailledTxIn {
            previous_output: utxo.and_then(|utxo| DetailledTxOutput::from_txout(utxo.txout, wallet).ok()),
            script_sig: input.script_sig,
            sequence: input.sequence,
            witness: input.witness,
        })
    }
}

#[derive(Clone, Debug)]
pub struct DetailledTxOutput {
    pub value: u64,
    pub address: Option<Address>,
    pub script_pubkey: ScriptBuf,
    pub is_mine: bool,
}

impl DetailledTxOutput {
    pub fn from_txout(output: TxOut, wallet: &BdkWallet) -> Result<DetailledTxOutput, Error> {
        Ok(DetailledTxOutput {
            value: output.value.to_sat(),
            is_mine: wallet.is_mine(output.script_pubkey.clone()),
            address: Address::from_script(output.script_pubkey.as_script(), wallet.network()).ok(),
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
