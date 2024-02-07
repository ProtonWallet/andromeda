use andromeda_bitcoin::{
    account::{Account, ScriptType},
    bitcoin::Network,
    blockchain::Blockchain,
    wallet::{Wallet, WalletConfig},
    BdkMemoryDatabase, DerivationPath,
};
use std::{
    io::{self, Write},
    str::SplitWhitespace,
    sync::{Arc, Mutex, RwLock},
};
use tokio;

fn create_wallet(words: &mut SplitWhitespace<'_>) -> Result<Arc<Mutex<Wallet<BdkMemoryDatabase>>>, &'static str> {
    let (bip39, bip38, network) = words.fold((None, None, None), |acc, word| {
        let bip39 = if acc.0.is_none() {
            word.strip_prefix("--bip39=")
                .map(|word| word.split("_").collect::<Vec<_>>().join(" "))
        } else {
            acc.0
        };

        let bip38 = if acc.1.is_none() {
            word.strip_prefix("--bip38=").map(|s| s.to_string())
        } else {
            acc.1
        };

        let network = if acc.2.is_none() {
            word.strip_prefix("--network=")
        } else {
            acc.2
        };

        return (bip39, bip38, network);
    });

    let bip39 = bip39.ok_or("ERROR: createwallet requires BIP39 mnemonic")?;

    let network = network
        .map_or(Ok(Network::Testnet), |str| str.to_string().try_into())
        .map_err(|_| "ERROR: invalid network")?;

    let config = WalletConfig::new(network);

    let wallet = Wallet::new(bip39, bip38, config);

    wallet
        .map(|wallet| Arc::new(Mutex::new(wallet)))
        .map_err(|_| "ERROR: could not create wallet")
}

fn require_wallet(
    wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>,
) -> Result<Arc<Mutex<Wallet<BdkMemoryDatabase>>>, &'static str> {
    wallet.ok_or("ERROR: you need to create a wallet first. use onchain:wallet command")
}

fn require_derivation_arg(words: &SplitWhitespace<'_>) -> Result<DerivationPath, &'static str> {
    let derivation_path = words.clone().fold(None, |acc, word| {
        let derivation_path = if acc.is_none() {
            word.strip_prefix("--derivationPath=")
        } else {
            acc
        };

        return derivation_path;
    });

    let derivation_path = derivation_path
        .ok_or("ERROR: derivation path is required")?
        .parse::<DerivationPath>()
        .map_err(|_| "ERROR: could not parse derivation path")?;

    Ok(derivation_path)
}

fn require_account_lock(
    wallet: Arc<Mutex<Wallet<BdkMemoryDatabase>>>,
    derivation_path: &DerivationPath,
) -> Result<Arc<RwLock<Account<BdkMemoryDatabase>>>, &'static str> {
    let mut lock = wallet.lock().unwrap();
    let account = lock.get_account(derivation_path).ok_or("ERROR: account not found")?;

    Ok(account.clone())
}

async fn get_wallet_balance(wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>) -> Result<(), &'static str> {
    let wallet = require_wallet(wallet)?;

    let lock = wallet.lock().unwrap();
    let balance = lock.get_balance().await.unwrap();

    println!("\nBALANCE");
    println!("confirmed: {}", balance.confirmed);
    println!("trusted_spendable: {}", balance.get_spendable());
    println!("trusted_pending: {}", balance.trusted_pending);
    println!("untrusted_pending: {}", balance.untrusted_pending);

    Ok(())
}

fn add_account(
    words: &mut SplitWhitespace<'_>,
    wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>,
) -> Result<DerivationPath, &'static str> {
    let wallet = require_wallet(wallet)?;

    let (script_type, account_index) = words.fold((None, None), |acc, word| {
        let script_type = if acc.0.is_none() {
            word.strip_prefix("--scriptType=")
                .map(|word| word.split("_").collect::<Vec<_>>().join(" "))
        } else {
            acc.0
        };

        let account_index = if acc.1.is_none() {
            word.strip_prefix("--accountIndex=")
        } else {
            acc.1
        };

        return (script_type, account_index);
    });

    let script_type = match script_type {
        None => Ok(ScriptType::NativeSegwit),
        Some(str) => str.try_into().map_err(|_| "ERROR:invalid script type"),
    }?;

    let account_index = match account_index {
        None => Ok(0),
        Some(index) => index.parse::<u32>().map_err(|_| "ERROR:invalid index"),
    }?;

    let mut lock = wallet.lock().unwrap();
    let derivation_path = lock.add_account(script_type, account_index, BdkMemoryDatabase::new());

    derivation_path.map_err(|_| "ERROR: could not add account")
}

async fn sync_account(
    words: &mut SplitWhitespace<'_>,
    wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>,
) -> Result<DerivationPath, &'static str> {
    let wallet = require_wallet(wallet)?;

    let derivation_path = require_derivation_arg(words)?;
    let account = require_account_lock(wallet, &derivation_path)?;

    let chain = Blockchain::new(None, None);

    let read_lock = account.read().unwrap();
    chain.full_sync(read_lock.get_wallet()).await.unwrap();
    drop(read_lock);

    Ok(derivation_path)
}

async fn get_account_balance(
    words: &mut SplitWhitespace<'_>,
    wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>,
) -> Result<(), &'static str> {
    let wallet = require_wallet(wallet)?;

    let derivation_path = require_derivation_arg(&words)?;
    let account = require_account_lock(wallet, &derivation_path)?;

    let account_lock = account.read().unwrap();
    let balance = account_lock.get_balance().map_err(|_| "Cannot get balance")?;

    println!("\nBALANCE");
    println!("confirmed: {}", balance.confirmed);
    println!("trusted_spendable: {}", balance.get_spendable());
    println!("trusted_pending: {}", balance.trusted_pending);
    println!("untrusted_pending: {}", balance.untrusted_pending);

    Ok(())
}

async fn get_account_transactions(
    words: &mut SplitWhitespace<'_>,
    wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>,
) -> Result<(), &'static str> {
    let wallet = require_wallet(wallet)?;

    let derivation_path = require_derivation_arg(&words)?;
    let account = require_account_lock(wallet, &derivation_path)?;

    let account_lock = account.read().unwrap();

    println!("\nTRANSACTIONS");
    account_lock
        .get_transactions(None, true)
        .map_err(|_| "Cannot get transactions")?
        .into_iter()
        .for_each(|simple_tx| {
            println!(
                "txid: {:?} | time {} | sent: {} sats | received: {} sats | fees: {} ",
                simple_tx.txid,
                simple_tx
                    .confirmation_time
                    .map_or("Unconfirmed".to_string(), |t| t.height.to_string()),
                simple_tx.sent,
                simple_tx.received,
                match simple_tx.fees {
                    Some(fees) => format!("{} sats", fees),
                    None => "None".to_string(),
                },
            );
        });

    Ok(())
}

async fn get_account_utxos(
    words: &mut SplitWhitespace<'_>,
    wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>>,
) -> Result<(), &'static str> {
    let wallet = require_wallet(wallet)?;

    let derivation_path = require_derivation_arg(&words)?;
    let account = require_account_lock(wallet, &derivation_path)?;

    let account_lock = account.read().unwrap();

    println!("\nUTXOs");
    account_lock
        .get_utxos()
        .map_err(|_| "Cannot get utxos")?
        .into_iter()
        .for_each(|utxo| {
            println!(
                "outpoint {} | keychain {:?} | value {} | spent {}",
                utxo.outpoint, utxo.keychain, utxo.txout.value, utxo.is_spent
            );
        });

    Ok(())
}

async fn poll_for_user_input() {
    println!("Proton Wallet CLI launched. Enter \"help\" to view available commands. Press Ctrl-D to quit.");

    let mut onchain_wallet: Option<Arc<Mutex<Wallet<BdkMemoryDatabase>>>> = None;

    loop {
        print!("> ");
        io::stdout().flush().unwrap(); // Without flushing, the `>` doesn't print
        let mut line = String::new();
        if let Err(e) = io::stdin().read_line(&mut line) {
            break println!("ERROR: {}", e);
        }

        if line.len() == 0 {
            // We hit EOF / Ctrl-D
            break;
        }

        let mut words = line.split_whitespace();
        if let Some(word) = words.next() {
            match word {
                "onchain:wallet" => {
                    let wallet = create_wallet(&mut words);

                    match wallet {
                        Err(err) => println!("{:?}", err),
                        Ok(wallet) => {
                            onchain_wallet = Some(wallet.clone());

                            let created = wallet.lock().unwrap();
                            println!(
                                "INFO: wallet was succesfully created. fingerprint: {}. network: {}",
                                created.get_fingerprint(),
                                created.get_network().to_string()
                            );
                        }
                    }
                }
                "onchain:wallet:balance" => match get_wallet_balance(onchain_wallet.clone()).await {
                    Err(err) => println!("{:?}", err),
                    Ok(_) => {}
                },
                "onchain:account" => match add_account(&mut words, onchain_wallet.clone()) {
                    Err(err) => println!("{:?}", err),
                    Ok(derivation_path) => {
                        println!(
                            "INFO: account was succesfully added to wallet. derivation path: {}",
                            derivation_path.to_string()
                        );
                    }
                },
                "onchain:account:sync" => match sync_account(&mut words, onchain_wallet.clone()).await {
                    Err(err) => println!("{:?}", err),
                    Ok(derivation_path) => {
                        println!("INFO: account synced. {}", &derivation_path.to_string())
                    }
                },
                "onchain:account:balance" => match get_account_balance(&mut words, onchain_wallet.clone()).await {
                    Err(err) => println!("{:?}", err),
                    Ok(_) => {}
                },
                "onchain:account:transactions" => {
                    match get_account_transactions(&mut words, onchain_wallet.clone()).await {
                        Err(err) => println!("{:?}", err),
                        Ok(_) => {}
                    }
                }
                "onchain:account:utxos" => match get_account_utxos(&mut words, onchain_wallet.clone()).await {
                    Err(err) => println!("{:?}", err),
                    Ok(_) => {}
                },
                _ => println!("Unknown command. See `\"help\" for available commands."),
            }
        }
    }
}

#[tokio::main]
async fn main() {
    poll_for_user_input().await;
}
