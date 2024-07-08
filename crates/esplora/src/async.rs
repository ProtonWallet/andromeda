// Bitcoin Dev Kit
// Written in 2020 by Alekos Filini <alekos.filini@gmail.com>
//
// Copyright (c) 2020-2021 Bitcoin Dev Kit Developers
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

//! Esplora by way of `reqwest` HTTP client.

use std::{collections::HashMap, str::FromStr};

use andromeda_api::{
    address::{AddressClient, ScriptHashTransactionsPayload},
    block::BlockClient,
    transaction::{ExchangeRateOrTransactionTime, TransactionClient},
    ProtonWalletApiClient,
};
use bitcoin::{
    block::Header as BlockHeader,
    consensus::{deserialize, serialize},
    hashes::{hex::FromHex, sha256, Hash},
    hex::DisplayHex,
    Block, BlockHash, MerkleBlock, Script, ScriptBuf, Transaction, Txid,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace};

use crate::{BlockStatus, BlockSummary, Error, MerkleProof, OutputStatus, Tx, TxStatus};

pub struct AsyncClient {
    transaction: TransactionClient,
    address: AddressClient,
    block: BlockClient,
}

const TRANSACTIONS_PER_PAGE: u32 = 25;

impl AsyncClient {
    /// build an async client from the base url and [`Client`]
    pub fn from_client(api_client: ProtonWalletApiClient) -> Self {
        let clients = api_client.clients();

        let transaction = clients.transaction;
        let address = clients.address;
        let block = clients.block;

        AsyncClient {
            transaction,
            address,
            block,
        }
    }

    /// Get a [`Transaction`] option given its [`Txid`]
    pub async fn get_tx(&self, txid: &Txid) -> Result<Option<Transaction>, Error> {
        let tx = self.transaction.get_raw_transaction(txid.to_string()).await?;
        Ok(Some(tx))
    }

    /// Get a [`Transaction`] given its [`Txid`].
    pub async fn get_tx_no_opt(&self, txid: &Txid) -> Result<Transaction, Error> {
        match self.get_tx(txid).await {
            Ok(Some(tx)) => Ok(tx),
            Ok(None) => Err(Error::TransactionNotFound(*txid)),
            Err(e) => Err(e),
        }
    }

    /// Get a [`Txid`] of a transaction given its index in a block with a given
    /// hash.
    pub async fn get_txid_at_block_index(&self, block_hash: &BlockHash, index: usize) -> Result<Option<Txid>, Error> {
        let txid = self.block.get_txid_at_block_index(block_hash, index).await?;
        Ok(Some(Txid::from_str(&txid)?))
    }

    /// Get the status of a [`Transaction`] given its [`Txid`].
    pub async fn get_tx_status(&self, txid: &Txid) -> Result<TxStatus, Error> {
        let status = self.transaction.get_transaction_status(txid.to_string()).await?;
        Ok(status.into())
    }

    #[deprecated(
        since = "0.2.0",
        note = "Deprecated to improve alignment with Esplora API. Users should use `get_block_hash` and `get_header_by_hash` methods directly."
    )]
    /// Get a [`BlockHeader`] given a particular block height.
    pub async fn get_header(&self, block_height: u32) -> Result<BlockHeader, Error> {
        let block_hash = self.get_block_hash(block_height).await?;
        self.get_header_by_hash(&block_hash).await
    }

    /// Get a [`BlockHeader`] given a particular block hash.
    pub async fn get_header_by_hash(&self, block_hash: &BlockHash) -> Result<BlockHeader, Error> {
        let header = self.block.get_header_by_hash(block_hash).await?;
        Ok(header)
    }

    /// Get the [`BlockStatus`] given a particular [`BlockHash`].
    pub async fn get_block_status(&self, block_hash: &BlockHash) -> Result<BlockStatus, Error> {
        let block_status = self.block.get_block_status(block_hash).await?;
        Ok(block_status.into())
    }

    /// Get a [`Block`] given a particular [`BlockHash`].
    pub async fn get_block_by_hash(&self, block_hash: &BlockHash) -> Result<Option<Block>, Error> {
        let block = self.block.get_block_by_hash(block_hash).await?;
        Ok(Some(block))
    }

    /// Get a merkle inclusion proof for a [`Transaction`] with the given
    /// [`Txid`].
    pub async fn get_merkle_proof(&self, tx_hash: &Txid) -> Result<Option<MerkleProof>, Error> {
        let merkle_proof = self
            .transaction
            .get_transaction_merkle_proof(tx_hash.to_string())
            .await?;

        Ok(Some(merkle_proof.into()))
    }

    /// Get a [`MerkleBlock`] inclusion proof for a [`Transaction`] with the
    /// given [`Txid`].
    pub async fn get_merkle_block(&self, tx_hash: &Txid) -> Result<Option<MerkleBlock>, Error> {
        let merkle_block = self
            .transaction
            .get_transaction_merkle_block_proof(tx_hash.to_string())
            .await?;

        let block = deserialize(&Vec::from_hex(&merkle_block)?)?;
        Ok(Some(block))
    }

    /// Get the spending status of an output given a [`Txid`] and the output
    /// index.
    pub async fn get_output_status(&self, txid: &Txid, index: u64) -> Result<Option<OutputStatus>, Error> {
        let output_status = self
            .transaction
            .get_outpoint_spending_status(txid.to_string(), index)
            .await?;

        Ok(Some(output_status.into()))
    }

    /// Broadcast a [`Transaction`] to Esplora
    pub async fn broadcast(
        &self,
        transaction: &Transaction,
        wallet_id: String,
        wallet_account_id: String,
        label: Option<String>,
        exchange_rate_or_transaction_time: ExchangeRateOrTransactionTime,
        address_id: Option<String>,
        subject: Option<String>,
        body: Option<String>,
    ) -> Result<(), Error> {
        self.transaction
            .broadcast_raw_transaction(
                serialize(transaction).to_lower_hex_string(),
                wallet_id,
                wallet_account_id,
                label,
                exchange_rate_or_transaction_time,
                address_id,
                subject,
                body,
            )
            .await?;

        Ok(())
    }

    /// Get the current height of the blockchain tip
    pub async fn get_height(&self) -> Result<u32, Error> {
        Ok(self.block.get_tip_height().await?)
    }

    /// Get the [`BlockHash`] of the current blockchain tip.
    pub async fn get_tip_hash(&self) -> Result<BlockHash, Error> {
        Ok(self.block.get_tip_hash().await?)
    }

    /// Get the [`BlockHash`] of a specific block height
    pub async fn get_block_hash(&self, block_height: u32) -> Result<BlockHash, Error> {
        Ok(self.block.get_block_hash(block_height).await?)
    }

    /// Get confirmed transaction history for the specified address/scripthash,
    /// sorted with newest first. Returns 25 transactions per page.
    /// More can be requested by specifying the last txid seen by the previous
    /// query.
    pub async fn scripthash_txs(&self, script: &Script, last_seen: Option<Txid>) -> Result<Vec<Tx>, Error> {
        let script_hash = sha256::Hash::hash(script.as_bytes());

        let txs = match last_seen {
            Some(last_seen) => {
                self.address
                    .get_scripthash_transactions_at_transaction_id(script_hash.to_string(), last_seen.to_string())
                    .await?
            }
            None => {
                self.address
                    .get_scripthash_transactions(script_hash.to_string())
                    .await?
            }
        };

        Ok(txs.into_iter().map(|tx| tx.into()).collect())
    }

    pub async fn many_scripthash_txs(
        &self,
        scripts: Vec<(u32, ScriptBuf)>,
    ) -> Result<HashMap<String, (u32, Vec<Tx>)>, Error> {
        let mut txs_by_spk_map: HashMap<String, (u32, Vec<Tx>)> = scripts
            .into_iter()
            .map(|(spk_index, spk)| (sha256::Hash::hash(spk.as_bytes()).to_string(), (spk_index, Vec::new())))
            .collect::<HashMap<_, _>>();

        let mut remaining_spks_to_fetch = txs_by_spk_map
            .iter()
            .map(|(spk, (_spk_index, txs))| ScriptHashTransactionsPayload {
                ScriptHash: spk.clone(),
                TransactionID: txs.last().map(|v| v.txid.to_string()),
            })
            .collect::<Vec<_>>();

        loop {
            let fetched_txs_by_spk = self
                .address
                .get_scripthashes_transactions(remaining_spks_to_fetch.clone())
                .await?;

            let mut new_remaining_spks_to_fetch = Vec::<ScriptHashTransactionsPayload>::new();
            fetched_txs_by_spk.iter().for_each(|(spk, fetched_txs)| {
                // Extends txs vectors with newly fetched transations
                let (_index, txs) = txs_by_spk_map.get_mut(spk).expect("Should be in the init hashmap");
                txs.extend(fetched_txs.clone().into_iter().map(|tx| tx.into()));

                // Refetch spk with txid as anchor if we hit the max items per page
                if fetched_txs.len() as u32 >= TRANSACTIONS_PER_PAGE {
                    new_remaining_spks_to_fetch.push(ScriptHashTransactionsPayload {
                        ScriptHash: spk.clone(),
                        TransactionID: fetched_txs.last().map(|v| v.TransactionID.clone()),
                    });
                }
            });

            if new_remaining_spks_to_fetch.is_empty() {
                break;
            }

            remaining_spks_to_fetch = new_remaining_spks_to_fetch;
        }

        Ok(txs_by_spk_map)
    }

    /// Get an map where the key is the confirmation target (in number of
    /// blocks) and the value is the estimated feerate (in sat/vB).
    pub async fn get_fee_estimates(&self) -> Result<HashMap<String, f64>, Error> {
        Ok(self.transaction.get_fee_estimates().await?)
    }

    /// Gets some recent block summaries starting at the tip or at `height` if
    /// provided.
    ///
    /// The maximum number of summaries returned depends on the backend itself:
    /// esplora returns `10` while [mempool.space](https://mempool.space/docs/api) returns `15`.
    pub async fn get_blocks(&self, height: Option<u32>) -> Result<Vec<BlockSummary>, Error> {
        let blocks = self.block.get_blocks(height).await?;
        Ok(blocks.into_iter().map(|block| block.into()).collect())
    }
}
