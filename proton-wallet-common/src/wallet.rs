use crate::account::{Account, AccountConfig, SupportedBIPs};
use crate::bitcoin::Network;
use crate::error::Error;
use crate::mnemonic::Mnemonic;

use bdk::wallet::Balance as BdkBalance;
use bdk::wallet::Update as BdkUpdate;
use bdk::{descriptor, Wallet as BdkWallet};
use miniscript::bitcoin::bip32::{DerivationPath, ExtendedPrivKey};

use std::collections::HashMap;

use std::io::Write;

use bdk_esplora::{esplora_client, EsploraAsyncExt};

const STOP_GAP: usize = 50;
const PARALLEL_REQUESTS: usize = 5;

#[derive(Debug)]
pub struct Wallet {
    mprv: ExtendedPrivKey,
    accounts: HashMap<DerivationPath, Account>,
    config: WalletConfig,
}

#[derive(Debug)]
pub struct WalletConfig {
    pub network: Network,
}

impl Wallet {
    pub fn new(bip39_mnemonic: String, bip38_passphrase: Option<String>, config: WalletConfig) -> Result<Self, Error> {
        let mnemonic = Mnemonic::from_string(bip39_mnemonic).unwrap();
        let mprv = ExtendedPrivKey::new_master(
            config.network.into(),
            &mnemonic.inner().to_seed(match bip38_passphrase {
                Some(bip38_passphrase) => bip38_passphrase,
                None => "".to_string(),
            }),
        )
        .unwrap();

        Ok(Wallet {
            mprv,
            accounts: HashMap::new(),
            config,
        })
    }

    pub fn add_account(&mut self, bip: SupportedBIPs, account_index: u32) {
        let account = Account::new(
            self.mprv,
            AccountConfig {
                bip,
                account_index,
                network: self.config.network,
            },
        )
        .unwrap();

        self.accounts.insert(account.derivation_path(), account);
    }

    pub async fn get_balance(&self) -> Result<BdkBalance, Error> {
        let cloned = self.accounts.clone();
        let mut iter = cloned.values();

        let mut balance = BdkBalance {
            untrusted_pending: 0,
            confirmed: 0,
            immature: 0,
            trusted_pending: 0,
        };

        while let Some(account) = iter.next() {
            let account_balance = account.clone().get_balance().await?;

            balance.untrusted_pending += account_balance.untrusted_pending;
            balance.confirmed += account_balance.confirmed;
            balance.immature += account_balance.immature;
            balance.trusted_pending += account_balance.trusted_pending;
        }

        Ok(balance)
    }

    pub fn get_wallet(&self) -> BdkWallet {
        let external_descriptor = descriptor!(wpkh((self.mprv, Vec::new().into()))).unwrap();

        BdkWallet::new_no_persist(external_descriptor, None, self.config.network.into())
            .map_err(|e| println!("error {}", e))
            .unwrap()
    }
}

pub async fn sync(mut wallet: BdkWallet) -> Result<BdkWallet, Error> {
    print!("Syncing...");
    // Scanning the blockchain
    let esplora_url = "https://mempool.space/testnet/api"; //TODO: make this dynamic
    let client = esplora_client::Builder::new(esplora_url)
        .build_async()
        .map_err(|_| Error::Generic {
            msg: "Could not create client".to_string(),
        })?;

    let checkpoint = wallet.latest_checkpoint();
    let keychain_spks = wallet
        .spks_of_all_keychains()
        .into_iter()
        .map(|(k, k_spks)| {
            let mut once = Some(());
            let mut stdout = std::io::stdout();
            let k_spks = k_spks
                .inspect(move |(spk_i, _)| match once.take() {
                    Some(_) => print!("\nScanning keychain [{:?}]", k),
                    None => print!(" {:<3}", spk_i),
                })
                .inspect(move |_| stdout.flush().expect("must flush"));
            (k, k_spks)
        })
        .collect();

    let (update_graph, last_active_indices) = client
        .scan_txs_with_keychains(keychain_spks, None, None, STOP_GAP, PARALLEL_REQUESTS)
        .await
        .map_err(|_| Error::Generic {
            msg: "Could not scan".to_string(),
        })?;

    let missing_heights = update_graph.missing_heights(wallet.local_chain());
    let chain_update = client
        .update_local_chain(checkpoint, missing_heights)
        .await
        .map_err(|_| Error::Generic {
            msg: "Could not update chain locally".to_string(),
        })?;

    let update = BdkUpdate {
        last_active_indices,
        graph: update_graph,
        chain: Some(chain_update),
    };

    wallet.apply_update(update).map_err(|_| Error::Generic {
        msg: "Couldn't apply wallet sync update".to_string(),
    })?;

    wallet.commit().map_err(|_| Error::Generic {
        msg: "Couldn't commit wallet sync update".to_string(),
    })?;

    Ok(wallet)
}
