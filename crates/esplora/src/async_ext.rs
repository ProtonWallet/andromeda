use async_trait::async_trait;
use bdk_core::{
    bitcoin::{BlockHash, OutPoint, ScriptBuf, Txid},
    collections::{BTreeMap, BTreeSet, HashSet},
    spk_client::{FullScanRequest, FullScanResult, SyncRequest, SyncResult},
    BlockId, CheckPoint, ConfirmationBlockTime, Indexed, TxUpdate,
};
use futures::{stream::FuturesOrdered, TryStreamExt};

use crate::{error::Error, insert_anchor_from_status, insert_prevouts, r#async::AsyncClient};

pub const MAX_SPKS_PER_REQUESTS: usize = 50;

/// Trait to extend the functionality of [`AsyncClient`].
///
/// Refer to [crate-level documentation](crate) for more.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait EsploraAsyncExt {
    /// Scan keychain scripts for transactions against Esplora, returning an
    /// update that can be applied to the receiving structures.
    ///
    /// `request` provides the data required to perform a script-pubkey-based
    /// full scan (see [`FullScanRequest`]). The full scan for each keychain
    /// (`K`) stops after a gap of `stop_gap` script pubkeys with no
    /// associated transactions. `parallel_requests` specifies the maximum
    /// number of HTTP requests to make in parallel.
    ///
    /// Refer to [crate-level docs](crate) for more.
    async fn full_scan<K: Ord + Clone + Send, R: Into<FullScanRequest<K>> + Send>(
        &self,
        request: R,
        stop_gap: usize,
    ) -> Result<FullScanResult<K>, Error>;

    /// Sync a set of scripts, txids, and/or outpoints against Esplora.
    ///
    /// `request` provides the data required to perform a script-pubkey-based
    /// sync (see [`SyncRequest`]). `parallel_requests` specifies the
    /// maximum number of HTTP requests to make in parallel.
    ///
    /// Refer to [crate-level docs](crate) for more.
    async fn sync<I: Send, R: Into<SyncRequest<I>> + Send>(
        &self,
        request: R,
        parallel_requests: usize,
    ) -> Result<SyncResult, Error>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl EsploraAsyncExt for AsyncClient {
    async fn full_scan<K: Ord + Clone + Send, R: Into<FullScanRequest<K>> + Send>(
        &self,
        request: R,
        stop_gap: usize,
    ) -> Result<FullScanResult<K>, Error> {
        let mut request = request.into();
        let keychains = request.keychains();

        let chain_tip = request.chain_tip();
        let latest_blocks = if chain_tip.is_some() {
            Some(fetch_latest_blocks(self).await?)
        } else {
            None
        };

        let mut tx_update = TxUpdate::<ConfirmationBlockTime>::default();
        let mut inserted_txs = HashSet::<Txid>::new();
        let mut last_active_indices = BTreeMap::<K, u32>::new();
        for keychain in keychains {
            let keychain_spks = request.iter_spks(keychain.clone());
            let (update, last_active_index) =
                fetch_txs_with_keychain_spks(self, &mut inserted_txs, keychain_spks, stop_gap).await?;
            tx_update.extend(update);
            if let Some(last_active_index) = last_active_index {
                last_active_indices.insert(keychain, last_active_index);
            }
        }

        let chain_update = match (chain_tip, latest_blocks) {
            (Some(chain_tip), Some(latest_blocks)) => {
                Some(chain_update(self, &latest_blocks, &chain_tip, &tx_update.anchors).await?)
            }
            _ => None,
        };

        Ok(FullScanResult {
            chain_update,
            tx_update,
            last_active_indices,
        })
    }

    async fn sync<I: Send, R: Into<SyncRequest<I>> + Send>(
        &self,
        request: R,
        parallel_requests: usize,
    ) -> Result<SyncResult, Error> {
        let mut request = request.into();

        let chain_tip = request.chain_tip();
        let latest_blocks = if chain_tip.is_some() {
            Some(fetch_latest_blocks(self).await?)
        } else {
            None
        };

        let mut tx_update = TxUpdate::<ConfirmationBlockTime>::default();
        let mut inserted_txs = HashSet::<Txid>::new();
        tx_update.extend(fetch_txs_with_spks(self, &mut inserted_txs, request.iter_spks()).await?);
        tx_update.extend(fetch_txs_with_txids(self, &mut inserted_txs, request.iter_txids(), parallel_requests).await?);
        tx_update.extend(
            fetch_txs_with_outpoints(self, &mut inserted_txs, request.iter_outpoints(), parallel_requests).await?,
        );

        let chain_update = match (chain_tip, latest_blocks) {
            (Some(chain_tip), Some(latest_blocks)) => {
                Some(chain_update(self, &latest_blocks, &chain_tip, &tx_update.anchors).await?)
            }
            _ => None,
        };

        Ok(SyncResult {
            chain_update,
            tx_update,
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

/// Used instead of [`esplora_client::BlockingClient::get_block_hash`].
///
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
async fn chain_update(
    client: &AsyncClient,
    latest_blocks: &BTreeMap<u32, BlockHash>,
    local_tip: &CheckPoint,
    anchors: &BTreeSet<(ConfirmationBlockTime, Txid)>,
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

    for (anchor, _txid) in anchors {
        let height = anchor.block_id.height;
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

/// Fetch transactions and associated [`ConfirmationBlockTime`]s by scanning
/// `keychain_spks` against Esplora.
///
/// `keychain_spks` is an *unbounded* indexed-[`ScriptBuf`] iterator that
/// represents scripts derived from a keychain. The scanning logic stops after a
/// `stop_gap` number of consecutive scripts with no transaction history is
/// reached. `parallel_requests` specifies the maximum number of HTTP requests
/// to make in parallel.
///
/// A [`TxGraph`] (containing the fetched transactions and anchors) and the last
/// active keychain index (if any) is returned. The last active keychain index
/// is the keychain's last script pubkey that contains a non-empty transaction
/// history.
///
/// Refer to [crate-level docs](crate) for more.
async fn fetch_txs_with_keychain_spks<I: Iterator<Item = Indexed<ScriptBuf>> + Send>(
    client: &AsyncClient,
    inserted_txs: &mut HashSet<Txid>,
    mut keychain_spks: I,
    stop_gap: usize,
) -> Result<(TxUpdate<ConfirmationBlockTime>, Option<u32>), Error> {
    let mut update = TxUpdate::<ConfirmationBlockTime>::default();

    let mut spks_to_fetch = Ord::min(stop_gap, MAX_SPKS_PER_REQUESTS);

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
        let req_spks = keychain_spks
            .by_ref()
            .take(spks_to_fetch)
            .collect::<Vec<(u32, ScriptBuf)>>();

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
                if inserted_txs.insert(tx.txid) {
                    update.txs.push(tx.to_tx().into());
                }
                insert_anchor_from_status(&mut update, tx.txid, tx.status);
                insert_prevouts(&mut update, tx.vin);
            }
        }
        let current_gap = last_index.expect("Should be set when handles is not empty") - last_active_index;
        let count_until_stop_gap = stop_gap.saturating_sub(current_gap as usize);
        if count_until_stop_gap == 0 {
            break;
        }
        spks_to_fetch = Ord::min(count_until_stop_gap, MAX_SPKS_PER_REQUESTS);
    }
    let last_active_index = u32::try_from(last_active_index).ok();
    Ok((update, last_active_index))
}

/// Fetch transactions and associated [`ConfirmationBlockTime`]s by scanning
/// `spks` against Esplora.
///
/// Unlike with [`EsploraAsyncExt::fetch_txs_with_keychain_spks`], `spks` must
/// be *bounded* as all contained scripts will be scanned. `parallel_requests`
/// specifies the maximum number of HTTP requests to make in parallel.
///
/// Refer to [crate-level docs](crate) for more.
async fn fetch_txs_with_spks<I: IntoIterator<Item = ScriptBuf> + Send>(
    client: &AsyncClient,
    inserted_txs: &mut HashSet<Txid>,
    spks: I,
) -> Result<TxUpdate<ConfirmationBlockTime>, Error>
where
    I::IntoIter: Send,
{
    fetch_txs_with_keychain_spks(
        client,
        inserted_txs,
        spks.into_iter().enumerate().map(|(i, spk)| (i as u32, spk)),
        usize::MAX,
    )
    .await
    .map(|(update, _)| update)
}

/// Fetch transactions and associated [`ConfirmationBlockTime`]s by scanning
/// `txids` against Esplora.
///
/// `parallel_requests` specifies the maximum number of HTTP requests to make in
/// parallel.
///
/// Refer to [crate-level docs](crate) for more.
async fn fetch_txs_with_txids<I: IntoIterator<Item = Txid> + Send>(
    client: &AsyncClient,
    inserted_txs: &mut HashSet<Txid>,
    txids: I,
    parallel_requests: usize,
) -> Result<TxUpdate<ConfirmationBlockTime>, Error>
where
    I::IntoIter: Send,
{
    let mut update = TxUpdate::<ConfirmationBlockTime>::default();
    // Only fetch for non-inserted txs.
    let mut txids = txids
        .into_iter()
        .filter(|txid| !inserted_txs.contains(txid))
        .collect::<Vec<Txid>>()
        .into_iter();
    loop {
        let handles = txids
            .by_ref()
            .take(parallel_requests)
            .map(|txid| async move { client.get_tx_info(&txid).await.map(|t| (txid, t)) })
            .collect::<FuturesOrdered<_>>();

        if handles.is_empty() {
            break;
        }

        for (txid, tx_info) in handles.try_collect::<Vec<_>>().await? {
            if let Some(tx_info) = tx_info {
                if inserted_txs.insert(txid) {
                    update.txs.push(tx_info.to_tx().into());
                }

                insert_anchor_from_status(&mut update, txid, tx_info.status);
                insert_prevouts(&mut update, tx_info.vin);
            }
        }
    }
    Ok(update)
}

/// Fetch transactions and [`ConfirmationBlockTime`]s that contain and spend the
/// provided `outpoints`.
///
/// `parallel_requests` specifies the maximum number of HTTP requests to make in
/// parallel.
///
/// Refer to [crate-level docs](crate) for more.
async fn fetch_txs_with_outpoints<I: IntoIterator<Item = OutPoint> + Send>(
    client: &AsyncClient,
    inserted_txs: &mut HashSet<Txid>,
    outpoints: I,
    parallel_requests: usize,
) -> Result<TxUpdate<ConfirmationBlockTime>, Error>
where
    I::IntoIter: Send,
{
    let outpoints = outpoints.into_iter().collect::<Vec<_>>();
    let mut update = TxUpdate::<ConfirmationBlockTime>::default();

    // make sure txs exists in graph and tx statuses are updated
    // TODO: We should maintain a tx cache (like we do with Electrum).
    update.extend(
        fetch_txs_with_txids(
            client,
            inserted_txs,
            outpoints.iter().copied().map(|op| op.txid),
            parallel_requests,
        )
        .await?,
    );

    // get outpoint spend-statuses
    let mut outpoints = outpoints.into_iter();
    let mut missing_txs = Vec::<Txid>::with_capacity(outpoints.len());
    loop {
        let handles = outpoints
            .by_ref()
            .take(parallel_requests)
            .map(|op| async move { client.get_output_status(&op.txid, op.vout as _).await })
            .collect::<FuturesOrdered<_>>();

        if handles.is_empty() {
            break;
        }

        for op_status in handles.try_collect::<Vec<_>>().await?.into_iter().flatten() {
            let spend_txid = match op_status.txid {
                Some(txid) => txid,
                None => continue,
            };
            if !inserted_txs.contains(&spend_txid) {
                missing_txs.push(spend_txid);
            }
            if let Some(spend_status) = op_status.status {
                insert_anchor_from_status(&mut update, spend_txid, spend_status);
            }
        }
    }

    update.extend(fetch_txs_with_txids(client, inserted_txs, missing_txs, parallel_requests).await?);
    Ok(update)
}
