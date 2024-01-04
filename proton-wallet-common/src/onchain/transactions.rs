use bdk::Wallet;
use bdk_chain::{tx_graph::CanonicalTx, ChainPosition, ConfirmationTimeAnchor};
use miniscript::bitcoin::{
    bip32::DerivationPath, psbt::PartiallySignedTransaction, Address, ScriptBuf, Transaction, TxIn, Txid,
};

use crate::common::error::Error;

#[derive(Clone, Debug)]
pub struct SimpleTransaction {
    pub txid: Txid,
    pub value: i64,
    pub fees: Option<u64>,
    pub time: TransactionTime,
    pub account_key: Option<DerivationPath>,
}

impl SimpleTransaction {
    pub fn from_can_tx<Storage>(
        can_tx: &CanonicalTx<'_, Transaction, ConfirmationTimeAnchor>,
        wallet: &Wallet<Storage>,
        account_key: Option<DerivationPath>,
    ) -> Self {
        let (sent, received) = wallet.spk_index().sent_and_received(can_tx.tx_node.tx);

        SimpleTransaction {
            account_key,
            txid: can_tx.tx_node.txid,
            value: received as i64 - sent as i64,
            fees: wallet.calculate_fee(can_tx.tx_node.tx).ok(),
            time: match can_tx.chain_position {
                ChainPosition::Confirmed(anchor) => TransactionTime::Confirmed {
                    confirmation_time: anchor.confirmation_time,
                },
                ChainPosition::Unconfirmed(last_seen) => TransactionTime::Unconfirmed { last_seen },
            },
        }
    }

    pub fn get_time(&self) -> u64 {
        match self.time {
            TransactionTime::Confirmed { confirmation_time } => confirmation_time,
            TransactionTime::Unconfirmed { last_seen } => last_seen,
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

#[derive(Clone, Debug)]
pub struct DetailledTransaction {
    pub txid: Txid,
    pub value: i64,
    pub fees: Option<u64>,
    pub time: Option<TransactionTime>,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<DetailledTxOutput>,
}

impl DetailledTransaction {
    pub fn from_psbt<Storage>(psbt: &PartiallySignedTransaction, wallet: &Wallet<Storage>) -> Result<Self, Error> {
        let tx = psbt.clone().extract_tx();

        let (sent, received) = wallet.spk_index().sent_and_received(&tx);
        let fees = wallet.calculate_fee(&tx).ok();

        let outputs: Vec<DetailledTxOutput> = tx
            .output
            .clone()
            .into_iter()
            .map(|output| {
                let tx = DetailledTxOutput {
                    value: output.value,
                    address: Address::from_script(&output.script_pubkey, wallet.network())
                        .map_err(|_| Error::CannotCreateAddressFromScript)?,
                    is_mine: wallet.is_mine(&output.script_pubkey),
                    script_pubkey: output.script_pubkey,
                };

                Ok(tx)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let tx = DetailledTransaction {
            txid: tx.txid(),
            value: received as i64 - sent as i64,
            fees,
            time: None,
            inputs: tx.input.clone(),
            outputs,
        };

        Ok(tx)
    }

    pub fn from_can_tx<Storage>(
        can_tx: &CanonicalTx<'_, Transaction, ConfirmationTimeAnchor>,
        wallet: &Wallet<Storage>,
    ) -> Result<Self, Error> {
        let (sent, received) = wallet.spk_index().sent_and_received(can_tx.tx_node.tx);
        let fees = wallet.calculate_fee(can_tx.tx_node.tx).ok();

        let outputs: Vec<DetailledTxOutput> = can_tx
            .tx_node
            .output
            .clone()
            .into_iter()
            .map(|output| {
                let tx = DetailledTxOutput {
                    value: output.value,
                    address: Address::from_script(&output.script_pubkey, wallet.network())
                        .map_err(|_| Error::CannotCreateAddressFromScript)?,
                    is_mine: wallet.is_mine(&output.script_pubkey),
                    script_pubkey: output.script_pubkey,
                };

                Ok(tx)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let tx = DetailledTransaction {
            txid: can_tx.tx_node.txid(),
            value: received as i64 - sent as i64,
            fees,
            time: Some(match can_tx.chain_position {
                ChainPosition::Confirmed(anchor) => TransactionTime::Confirmed {
                    confirmation_time: anchor.confirmation_time,
                },
                ChainPosition::Unconfirmed(last_seen) => TransactionTime::Unconfirmed { last_seen },
            }),
            inputs: can_tx.tx_node.input.clone(),
            outputs,
        };

        Ok(tx)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TransactionTime {
    Confirmed { confirmation_time: u64 },
    Unconfirmed { last_seen: u64 },
}

pub fn get_tx_time(can_tx: &CanonicalTx<'_, Transaction, ConfirmationTimeAnchor>) -> u64 {
    match can_tx.chain_position {
        ChainPosition::Confirmed(anchor) => anchor.confirmation_time,
        ChainPosition::Unconfirmed(last_seen) => last_seen,
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
