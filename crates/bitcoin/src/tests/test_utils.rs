use core::str::FromStr;
use std::sync::Arc;

use andromeda_common::{Network, ScriptType};
use bdk_chain::{tx_graph, BlockId, ConfirmationBlockTime};
use bdk_wallet::{Update, Wallet};
use bitcoin::{
    absolute,
    bip32::{DerivationPath, Xpriv},
    hashes::Hash,
    transaction, Address, Amount, BlockHash, NetworkKind, OutPoint, Transaction, TxIn, TxOut, Txid,
};

use crate::{account::Account, mnemonic::Mnemonic, storage::WalletStorage, KeychainKind};

pub struct TestUtils {}

impl TestUtils {
    /// Return a fake wallet that appears to be funded for testing.
    ///
    /// The funded wallet contains a tx with a 76_000 sats input and two outputs, one spending 25_000
    /// to a foreign address and one returning 50_000 back to the wallet. The remaining 1000
    /// sats are the transaction fee.
    pub fn get_funded_wallet_single(descriptor: &str) -> (Wallet, Txid) {
        Self::new_funded_wallet(descriptor, None)
    }

    /// Return a fake wallet that appears to be funded for testing.
    ///
    /// The funded wallet contains a tx with a 76_000 sats input and two outputs, one spending 25_000
    /// to a foreign address and one returning 50_000 back to the wallet. The remaining 1000
    /// sats are the transaction fee.
    pub fn get_funded_wallet(descriptor: &str, change_descriptor: &str) -> (Wallet, Txid) {
        Self::new_funded_wallet(descriptor, Some(change_descriptor))
    }

    fn new_funded_wallet(descriptor: &str, change_descriptor: Option<&str>) -> (Wallet, Txid) {
        let params = if let Some(change_desc) = change_descriptor {
            Wallet::create(descriptor.to_string(), change_desc.to_string())
        } else {
            Wallet::create_single(descriptor.to_string())
        };

        let mut wallet = params
            .network(Network::Regtest.into())
            .create_wallet_no_persist()
            .expect("descriptors must be valid");

        let receive_address = wallet.peek_address(KeychainKind::External, 0).address;
        let sendto_address = Address::from_str("bcrt1q3qtze4ys45tgdvguj66zrk4fu6hq3a3v9pfly5")
            .expect("address")
            .require_network(Network::Regtest.into())
            .unwrap();

        let tx0 = Transaction {
            output: vec![TxOut {
                value: Amount::from_sat(76_000),
                script_pubkey: receive_address.script_pubkey(),
            }],
            ..Self::new_tx(0)
        };

        let tx1 = Transaction {
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: tx0.compute_txid(),
                    vout: 0,
                },
                ..Default::default()
            }],
            output: vec![
                TxOut {
                    value: Amount::from_sat(50_000),
                    script_pubkey: receive_address.script_pubkey(),
                },
                TxOut {
                    value: Amount::from_sat(25_000),
                    script_pubkey: sendto_address.script_pubkey(),
                },
            ],
            ..Self::new_tx(0)
        };

        Self::insert_checkpoint(
            &mut wallet,
            BlockId {
                height: 42,
                hash: BlockHash::all_zeros(),
            },
        );
        Self::insert_checkpoint(
            &mut wallet,
            BlockId {
                height: 1_000,
                hash: BlockHash::all_zeros(),
            },
        );
        Self::insert_checkpoint(
            &mut wallet,
            BlockId {
                height: 2_000,
                hash: BlockHash::all_zeros(),
            },
        );

        Self::insert_tx(&mut wallet, tx0.clone());
        Self::insert_anchor(
            &mut wallet,
            tx0.compute_txid(),
            ConfirmationBlockTime {
                block_id: BlockId {
                    height: 1_000,
                    hash: BlockHash::all_zeros(),
                },
                confirmation_time: 100,
            },
        );

        Self::insert_tx(&mut wallet, tx1.clone());
        Self::insert_anchor(
            &mut wallet,
            tx1.compute_txid(),
            ConfirmationBlockTime {
                block_id: BlockId {
                    height: 2_000,
                    hash: BlockHash::all_zeros(),
                },
                confirmation_time: 200,
            },
        );

        (wallet, tx1.compute_txid())
    }

    /// A new empty transaction with the given locktime
    pub fn new_tx(locktime: u32) -> Transaction {
        Transaction {
            version: transaction::Version::ONE,
            lock_time: absolute::LockTime::from_consensus(locktime),
            input: vec![],
            output: vec![],
        }
    }

    /// Insert transaction
    pub fn insert_tx(wallet: &mut Wallet, tx: Transaction) {
        wallet
            .apply_update(Update {
                tx_update: bdk_chain::TxUpdate {
                    txs: vec![Arc::new(tx)],
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
    }

    /// Simulates confirming a tx with `txid` by applying an update to the wallet containing
    /// the given `anchor`. Note: to be considered confirmed the anchor block must exist in
    /// the current active chain.
    pub fn insert_anchor(wallet: &mut Wallet, txid: Txid, anchor: ConfirmationBlockTime) {
        wallet
            .apply_update(Update {
                tx_update: tx_graph::TxUpdate {
                    anchors: [(anchor, txid)].into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
    }

    /// Insert a checkpoint into the wallet. This can be used to extend the wallet's local chain
    /// or to insert a block that did not exist previously. Note that if replacing a block with
    /// a different one at the same height, then all later blocks are evicted as well.
    pub fn insert_checkpoint(wallet: &mut Wallet, block: BlockId) {
        let mut cp = wallet.latest_checkpoint();
        cp = cp.insert(block);
        wallet
            .apply_update(Update {
                chain: Some(cp),
                ..Default::default()
            })
            .unwrap();
    }

    pub fn create_test_account(script_type: ScriptType, derivation_path: &str) -> Account {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower".to_string(),
        )
        .unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();
        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();
        Account::new(
            master_secret_key,
            Network::Regtest,
            script_type,
            derivation_path,
            WalletStorage::memory_persist(),
        )
        .unwrap()
    }
}
