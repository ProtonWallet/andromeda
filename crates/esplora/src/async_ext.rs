use std::collections::BTreeSet;

use async_trait::async_trait;
use bdk_chain::{
    bitcoin::{BlockHash, OutPoint, ScriptBuf, TxOut, Txid},
    collections::BTreeMap,
    local_chain::CheckPoint,
    spk_client::{FullScanRequest, FullScanResult, SyncRequest, SyncResult},
    Anchor, BlockId, ConfirmationTimeHeightAnchor, Indexed, TxGraph,
};
use bitcoin::Amount;
use futures::{stream::FuturesOrdered, TryStreamExt};

use super::{error::Error, AsyncClient};
use crate::TxStatus;

pub const MAX_SPKS_PER_REQUESTS: usize = 50;

/// Refer to [crate-level documentation] for more.
///
/// [crate-level documentation]: crate
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait EsploraAsyncExt {
    /// Scan keychain scripts for transactions against Esplora, returning an
    /// update that can be applied to the receiving structures.
    ///
    /// * `local_tip`: the previously seen tip from [`LocalChain::tip`].
    /// * `keychain_spks`: keychains that we want to scan transactions for
    ///
    /// The full scan for each keychain stops after a gap of `stop_gap` script
    /// pubkeys with no associated transactions.
    ///
    /// ## Note
    ///
    /// `stop_gap` is defined as "the maximum number of consecutive unused
    /// addresses". For example, with a `stop_gap` of  3, `full_scan` will
    /// keep scanning until it encounters 3 consecutive script pubkeys with
    /// no associated transactions.
    ///
    /// This follows the same approach as other Bitcoin-related software,
    /// such as [Electrum](https://electrum.readthedocs.io/en/latest/faq.html#what-is-the-gap-limit),
    /// [BTCPay Server](https://docs.btcpayserver.org/FAQ/Wallet/#the-gap-limit-problem),
    /// and [Sparrow](https://www.sparrowwallet.com/docs/faq.html#ive-restored-my-wallet-but-some-of-my-funds-are-missing).
    ///
    /// A `stop_gap` of 0 will be treated as a `stop_gap` of 1.
    ///
    /// [`LocalChain::tip`]: bdk_chain::local_chain::LocalChain::tip
    async fn full_scan<K: Ord + Clone + Send>(
        &self,
        request: FullScanRequest<K>,
        stop_gap: usize,
    ) -> Result<FullScanResult<K>, Error>;

    /// Sync a set of scripts with the blockchain (via an Esplora client) for
    /// the data specified and return a [`TxGraph`].
    ///
    /// * `local_tip`: the previously seen tip from [`LocalChain::tip`].
    /// * `misc_spks`: scripts that we want to sync transactions for
    /// * `txids`: transactions for which we want updated
    ///   [`ConfirmationTimeHeightAnchor`]s
    /// * `outpoints`: transactions associated with these outpoints (residing,
    ///   spending) that we want to include in the update
    ///
    /// If the scripts to sync are unknown, such as when restoring or importing
    /// a keychain that may include scripts that have been used, use
    /// [`full_scan`] with the keychain.
    ///
    /// [`LocalChain::tip`]: bdk_chain::local_chain::LocalChain::tip
    /// [`full_scan`]: EsploraAsyncExt::full_scan
    async fn sync(&self, request: SyncRequest, parallel_requests: usize) -> Result<SyncResult, Error>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl EsploraAsyncExt for AsyncClient {
    async fn full_scan<K: Ord + Clone + Send>(
        &self,
        request: FullScanRequest<K>,
        stop_gap: usize,
    ) -> Result<FullScanResult<K>, Error> {
        let latest_blocks = fetch_latest_blocks(self).await?;
        let (graph_update, last_active_indices) =
            full_scan_for_index_and_graph(self, request.spks_by_keychain, stop_gap).await?;

        let chain_update = chain_update(self, &latest_blocks, &request.chain_tip, graph_update.all_anchors()).await?;
        Ok(FullScanResult {
            chain_update,
            graph_update,
            last_active_indices,
        })
    }

    async fn sync(&self, request: SyncRequest, parallel_requests: usize) -> Result<SyncResult, Error> {
        let latest_blocks = fetch_latest_blocks(self).await?;
        let graph_update =
            sync_for_index_and_graph(self, request.spks, request.txids, request.outpoints, parallel_requests).await?;
        let chain_update = chain_update(self, &latest_blocks, &request.chain_tip, graph_update.all_anchors()).await?;
        Ok(SyncResult {
            chain_update,
            graph_update,
        })
    }
}

/// Fetch latest blocks from Esplora in an atomic call.
///
/// We want to do this before fetching transactions and anchors as we cannot
/// fetch latest blocks AND transactions atomically, and the checkpoint tip is
/// used to determine last-scanned block (for block-based chain-sources).
/// Therefore it's better to be conservative when setting the tip (use
/// an earlier tip rather than a later tip) otherwise the caller may
/// accidentally skip blocks when alternating between chain-sources.
async fn fetch_latest_blocks(client: &AsyncClient) -> Result<BTreeMap<u32, BlockHash>, Error> {
    Ok(client
        .get_blocks(None)
        .await?
        .into_iter()
        .map(|b| (b.time.height, b.id))
        .collect())
}

/// This first checks the previously fetched `latest_blocks` before fetching
/// from Esplora again.
async fn fetch_block(
    client: &AsyncClient,
    latest_blocks: &BTreeMap<u32, BlockHash>,
    height: u32,
) -> Result<Option<BlockHash>, Error> {
    if let Some(&hash) = latest_blocks.get(&height) {
        return Ok(Some(hash));
    }

    // We avoid fetching blocks higher than previously fetched `latest_blocks` as
    // the local chain tip is used to signal for the last-synced-up-to-height.
    let &tip_height = latest_blocks.keys().last().expect("must have atleast one entry");
    if height > tip_height {
        return Ok(None);
    }

    Ok(Some(client.get_block_hash(height).await?))
}

/// Create the [`local_chain::Update`].
///
/// We want to have a corresponding checkpoint per anchor height. However,
/// checkpoints fetched should not surpass `latest_blocks`.
async fn chain_update<A: Anchor>(
    client: &AsyncClient,
    latest_blocks: &BTreeMap<u32, BlockHash>,
    local_tip: &CheckPoint,
    anchors: &BTreeSet<(A, Txid)>,
) -> Result<CheckPoint, Error> {
    let mut point_of_agreement = None;
    let mut conflicts = vec![];
    for local_cp in local_tip.iter() {
        let remote_hash = match fetch_block(client, latest_blocks, local_cp.height()).await? {
            Some(hash) => hash,
            None => continue,
        };
        if remote_hash == local_cp.hash() {
            point_of_agreement = Some(local_cp.clone());
            break;
        } else {
            // it is not strictly necessary to include all the conflicted heights (we do
            // need the first one) but it seems prudent to make sure the updated
            // chain's heights are a superset of the existing chain after
            // update.
            conflicts.push(BlockId {
                height: local_cp.height(),
                hash: remote_hash,
            });
        }
    }

    let mut tip = point_of_agreement.expect("remote esplora should have same genesis block");

    tip = tip.extend(conflicts.into_iter().rev()).expect("evicted are in order");

    for anchor in anchors {
        let height = anchor.0.anchor_block().height;
        if tip.get(height).is_none() {
            let hash = match fetch_block(client, latest_blocks, height).await? {
                Some(hash) => hash,
                None => continue,
            };
            tip = tip.insert(BlockId { height, hash });
        }
    }

    // insert the most recent blocks at the tip to make sure we update the tip and
    // make the update robust.
    for (&height, &hash) in latest_blocks.iter() {
        tip = tip.insert(BlockId { height, hash });
    }

    Ok(tip)
}

/// This performs a full scan to get an update for the [`TxGraph`] and
/// [`KeychainTxOutIndex`](bdk_chain::keychain::KeychainTxOutIndex).
async fn full_scan_for_index_and_graph<K: Ord + Clone + Send>(
    client: &AsyncClient,
    keychain_spks: BTreeMap<K, impl IntoIterator<IntoIter = impl Iterator<Item = Indexed<ScriptBuf>> + Send> + Send>,
    stop_gap: usize,
) -> Result<(TxGraph<ConfirmationTimeHeightAnchor>, BTreeMap<K, u32>), Error> {
    let mut graph = TxGraph::<ConfirmationTimeHeightAnchor>::default();
    let mut last_active_indexes = BTreeMap::<K, u32>::new();

    for (keychain, spks) in keychain_spks {
        let mut spks_to_fetch = Ord::min(stop_gap, MAX_SPKS_PER_REQUESTS);

        let mut spks = spks.into_iter();

        let mut last_index: Option<i32> = None;

        // last_active_index is -1 by default, so that if we don't find any
        // active index, we'll still have a gap taking into account first index
        // (at index 0)
        //
        // Example:
        // - let's say that my stop_gap is 10, last_index is 9 (we fetched first 10
        //   indexes) and we didn't find any active index
        // - then we have: count_until_stop_gap = stop_gap -
        // (last_index - last_active_index) = 10 - (9 - (-1)) = 10 - 10 = 0
        //
        // On the other hand, it doesn't affect other result since
        // last_active_index will be then set to the index
        let mut last_active_index = -1;

        loop {
            let req_spks = spks.by_ref().take(spks_to_fetch).collect::<Vec<(u32, ScriptBuf)>>();

            if req_spks.is_empty() {
                break;
            }

            let handles: std::collections::HashMap<String, (u32, Vec<crate::Tx>)> =
                client.many_scripthash_txs(req_spks).await?;

            if handles.is_empty() {
                break;
            }

            let mut sorted_handles = handles.values().collect::<Vec<_>>();
            sorted_handles.sort_by(|(a_index, _), (b_index, _)| a_index.partial_cmp(b_index).unwrap());

            for (index, txs) in sorted_handles.iter() {
                let index = *index as i32;
                let txs = txs.clone();

                last_index = Some(index);
                if !txs.is_empty() {
                    last_active_index = index;
                }

                for tx in txs {
                    let _ = graph.insert_tx(tx.to_tx());
                    if let Some(anchor) = anchor_from_status(&tx.status) {
                        let _ = graph.insert_anchor(tx.txid, anchor);
                    }

                    let previous_outputs = tx.vin.iter().filter_map(|vin| {
                        let prevout = vin.prevout.as_ref()?;
                        Some((
                            OutPoint {
                                txid: vin.txid,
                                vout: vin.vout,
                            },
                            TxOut {
                                script_pubkey: prevout.scriptpubkey.clone(),
                                value: Amount::from_sat(prevout.value),
                            },
                        ))
                    });

                    for (outpoint, txout) in previous_outputs {
                        let _ = graph.insert_txout(outpoint, txout);
                    }
                }
            }

            let current_gap = last_index.expect("Should be set when handles is not empty") - last_active_index;
            let count_until_stop_gap = stop_gap.saturating_sub(current_gap as usize);

            if count_until_stop_gap == 0 {
                break;
            }

            spks_to_fetch = Ord::min(count_until_stop_gap, MAX_SPKS_PER_REQUESTS);
        }

        if last_active_index > -1 {
            last_active_indexes.insert(keychain, last_active_index.try_into().unwrap_or(0));
        }
    }

    Ok((graph, last_active_indexes))
}

async fn sync_for_index_and_graph(
    client: &AsyncClient,
    misc_spks: impl IntoIterator<IntoIter = impl Iterator<Item = ScriptBuf> + Send> + Send,
    txids: impl IntoIterator<IntoIter = impl Iterator<Item = Txid> + Send> + Send,
    outpoints: impl IntoIterator<IntoIter = impl Iterator<Item = OutPoint> + Send> + Send,
    parallel_requests: usize,
) -> Result<TxGraph<ConfirmationTimeHeightAnchor>, Error> {
    let mut graph = full_scan_for_index_and_graph(
        client,
        [((), misc_spks.into_iter().enumerate().map(|(i, spk)| (i as u32, spk)))].into(),
        usize::MAX,
    )
    .await
    .map(|(g, _)| g)?;

    let mut txids = txids.into_iter();
    loop {
        let handles = txids
            .by_ref()
            .take(parallel_requests)
            .filter(|&txid| graph.get_tx(txid).is_none())
            .map(|txid| async move { client.get_tx_status(&txid).await.map(|s| (txid, s)) })
            .collect::<FuturesOrdered<_>>();

        if handles.is_empty() {
            break;
        }

        for (txid, status) in handles.try_collect::<Vec<(Txid, TxStatus)>>().await? {
            if let Some(anchor) = anchor_from_status(&status) {
                let _ = graph.insert_anchor(txid, anchor);
            }
        }
    }

    for op in outpoints.into_iter() {
        if graph.get_tx(op.txid).is_none() {
            if let Some(tx) = client.get_tx(&op.txid).await? {
                let _ = graph.insert_tx(tx);
            }
            let status = client.get_tx_status(&op.txid).await?;
            if let Some(anchor) = anchor_from_status(&status) {
                let _ = graph.insert_anchor(op.txid, anchor);
            }
        }

        if let Some(op_status) = client.get_output_status(&op.txid, op.vout as _).await? {
            if let Some(txid) = op_status.txid {
                if graph.get_tx(txid).is_none() {
                    if let Some(tx) = client.get_tx(&txid).await? {
                        let _ = graph.insert_tx(tx);
                    }
                    let status = client.get_tx_status(&txid).await?;
                    if let Some(anchor) = anchor_from_status(&status) {
                        let _ = graph.insert_anchor(txid, anchor);
                    }
                }
            }
        }
    }

    Ok(graph)
}

fn anchor_from_status(status: &TxStatus) -> Option<ConfirmationTimeHeightAnchor> {
    if let TxStatus {
        block_height: Some(height),
        block_hash: Some(hash),
        block_time: Some(time),
        ..
    } = status.clone()
    {
        Some(ConfirmationTimeHeightAnchor {
            anchor_block: BlockId { height, hash },
            confirmation_height: height,
            confirmation_time: time,
        })
    } else {
        None
    }
}
