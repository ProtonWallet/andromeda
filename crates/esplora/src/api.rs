//! structs from the esplora API
//!
//! see: <https://github.com/Blockstream/esplora/blob/master/API.md>

use std::str::FromStr;

use andromeda_api::transaction::ApiTransactionStatus;
pub use bitcoin::{
    consensus::{deserialize, serialize},
    hashes::hex::FromHex,
    BlockHash, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness,
};
use bitcoin::{transaction::Version, Amount};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PrevOut {
    pub value: u64,
    pub scriptpubkey: ScriptBuf,
}

impl From<andromeda_api::address::ApiVout> for PrevOut {
    fn from(api_vout: andromeda_api::address::ApiVout) -> Self {
        PrevOut {
            value: api_vout.Value,
            scriptpubkey: ScriptBuf::from_hex(&api_vout.ScriptPubKey).unwrap(),
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Vin {
    pub txid: Txid,
    pub vout: u32,
    // None if coinbase
    pub prevout: Option<PrevOut>,
    pub scriptsig: ScriptBuf,
    #[serde(deserialize_with = "deserialize_witness", default)]
    pub witness: Vec<Vec<u8>>,
    pub sequence: u32,
    pub is_coinbase: bool,
}

impl From<andromeda_api::address::ApiVin> for Vin {
    fn from(api_vin: andromeda_api::address::ApiVin) -> Self {
        Vin {
            txid: Txid::from_str(&api_vin.TransactionID).unwrap(),
            vout: api_vin.Vout,
            prevout: Some(api_vin.Prevout.into()),
            scriptsig: ScriptBuf::from_hex(&api_vin.ScriptSig).unwrap(),
            witness: match api_vin.Witness {
                Some(witnesses) => witnesses.into_iter().map(|s| s.into_bytes()).collect(),
                None => Vec::new(),
            },
            sequence: api_vin.Sequence,
            is_coinbase: api_vin.IsCoinbase != 0,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Vout {
    pub value: u64,
    pub scriptpubkey: ScriptBuf,
}

impl From<andromeda_api::address::ApiVout> for Vout {
    fn from(api_vout: andromeda_api::address::ApiVout) -> Self {
        Vout {
            value: api_vout.Value,
            scriptpubkey: ScriptBuf::from_hex(&api_vout.ScriptPubKey).unwrap(),
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TxStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
    pub block_hash: Option<BlockHash>,
    pub block_time: Option<u64>,
}

impl From<ApiTransactionStatus> for TxStatus {
    fn from(transaction_status: ApiTransactionStatus) -> Self {
        TxStatus {
            confirmed: transaction_status.IsConfirmed != 0,
            block_height: transaction_status.BlockHeight,
            block_hash: transaction_status.BlockHash.and_then(|b| BlockHash::from_str(&b).ok()),
            block_time: transaction_status.BlockTime,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MerkleProof {
    pub block_height: u32,
    pub merkle: Vec<Txid>,
    pub pos: usize,
}

impl From<andromeda_api::transaction::TransactionMerkleProof> for MerkleProof {
    fn from(transaction_merkle_proof: andromeda_api::transaction::TransactionMerkleProof) -> Self {
        MerkleProof {
            block_height: transaction_merkle_proof.BlockHeight,
            merkle: transaction_merkle_proof
                .Merkle
                .into_iter()
                .map(|hash| Txid::from_str(&hash).unwrap())
                .collect(),
            pos: transaction_merkle_proof.Position as usize,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct OutputStatus {
    pub spent: bool,
    pub txid: Option<Txid>,
    pub vin: Option<u64>,
    pub status: Option<TxStatus>,
}

impl From<andromeda_api::transaction::OutpointSpendingStatus> for OutputStatus {
    fn from(outpoint_spending_status: andromeda_api::transaction::OutpointSpendingStatus) -> Self {
        OutputStatus {
            spent: outpoint_spending_status.IsSpent != 0,
            txid: outpoint_spending_status
                .TransactionID
                .map(|t| Txid::from_str(&t).unwrap()),
            vin: outpoint_spending_status.Vin,
            status: outpoint_spending_status.TransactionStatus.map(|s| s.into()),
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockStatus {
    pub in_best_chain: bool,
    pub height: Option<u32>,
    pub next_best: Option<BlockHash>,
}

impl From<andromeda_api::block::BlockStatus> for BlockStatus {
    fn from(block_status: andromeda_api::block::BlockStatus) -> Self {
        BlockStatus {
            in_best_chain: block_status.IsInBestChain != 0,
            height: Some(block_status.BlockHeight),
            next_best: Some(BlockHash::from_str(&block_status.NextBest).unwrap()),
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Tx {
    pub txid: Txid,
    pub version: i32,
    pub locktime: u32,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub status: TxStatus,
    pub fee: u64,
}

impl From<andromeda_api::address::ApiTx> for Tx {
    fn from(tx: andromeda_api::address::ApiTx) -> Self {
        Tx {
            txid: Txid::from_str(&tx.TransactionID).unwrap(),
            version: tx.Version,
            locktime: tx.Locktime,
            vin: tx.Vin.unwrap().into_iter().map(|vin| vin.into()).collect(),
            vout: tx.Vout.unwrap().into_iter().map(|vout| vout.into()).collect(),
            status: tx.TransactionStatus.into(),
            fee: tx.Fee,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockTime {
    pub timestamp: u64,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct BlockSummary {
    pub id: BlockHash,
    #[serde(flatten)]
    pub time: BlockTime,
    /// Hash of the previous block, will be `None` for the genesis block.
    pub previousblockhash: Option<bitcoin::BlockHash>,
    pub merkle_root: bitcoin::hash_types::TxMerkleNode,
}

impl From<andromeda_api::block::ApiBlock> for BlockSummary {
    fn from(block: andromeda_api::block::ApiBlock) -> Self {
        BlockSummary {
            id: BlockHash::from_str(&block.ID).unwrap(),
            time: BlockTime {
                timestamp: block.Timestamp,
                height: block.BlockHeight,
            },
            previousblockhash: block
                .PreviousBlockHash
                .map(|value| BlockHash::from_str(&value).unwrap()),
            merkle_root: bitcoin::hash_types::TxMerkleNode::from_str(&block.MerkleRoot).unwrap(),
        }
    }
}

impl Tx {
    pub fn to_tx(&self) -> Transaction {
        Transaction {
            version: Version(self.version),
            lock_time: bitcoin::absolute::LockTime::from_consensus(self.locktime),
            input: self
                .vin
                .iter()
                .cloned()
                .map(|vin| TxIn {
                    previous_output: OutPoint {
                        txid: vin.txid,
                        vout: vin.vout,
                    },
                    script_sig: vin.scriptsig,
                    sequence: bitcoin::Sequence(vin.sequence),
                    witness: Witness::from_slice(&vin.witness),
                })
                .collect(),
            output: self
                .vout
                .iter()
                .cloned()
                .map(|vout| TxOut {
                    value: Amount::from_sat(vout.value),
                    script_pubkey: vout.scriptpubkey,
                })
                .collect(),
        }
    }

    pub fn confirmation_time(&self) -> Option<BlockTime> {
        match self.status {
            TxStatus {
                confirmed: true,
                block_height: Some(height),
                block_time: Some(timestamp),
                ..
            } => Some(BlockTime { timestamp, height }),
            _ => None,
        }
    }

    pub fn previous_outputs(&self) -> Vec<Option<TxOut>> {
        self.vin
            .iter()
            .cloned()
            .map(|vin| {
                vin.prevout.map(|po| TxOut {
                    script_pubkey: po.scriptpubkey,
                    value: Amount::from_sat(po.value),
                })
            })
            .collect()
    }
}

fn deserialize_witness<'de, D>(d: D) -> Result<Vec<Vec<u8>>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let list = Vec::<String>::deserialize(d)?;
    list.into_iter()
        .map(|hex_str| Vec::<u8>::from_hex(&hex_str))
        .collect::<Result<Vec<Vec<u8>>, _>>()
        .map_err(serde::de::Error::custom)
}
