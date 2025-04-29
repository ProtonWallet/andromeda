///! [`AccountStatementGenerator`] provides functionality to generate account statements in the Typst format.
///! It allows users to create detailed statements for their Bitcoin accounts, including
///! transaction history, balances, and total account value.
use crate::error::Error;
use andromeda_api::exchange_rate::ApiExchangeRate;
use andromeda_bitcoin::account::Account;
use andromeda_common::{pdf_generator::PDFGenerator, BITCOIN};
use bdk_chain::Balance as BdkBalance;
use bitcoin::Amount;
use chrono::{Datelike, TimeZone, Utc};
use std::sync::Arc;
#[derive(Clone)]
pub struct AccountStatementGenerator {
    accounts: Vec<Arc<Account>>,
    account_names: Vec<String>,
    exchange_rate: Option<ApiExchangeRate>,
}

impl AccountStatementGenerator {
    /// Creates a new instance of `AccountStatementGenerator`.
    pub fn new(
        accounts: Vec<Arc<Account>>,
        account_names: Vec<String>,
        exchange_rate: Option<ApiExchangeRate>,
    ) -> Self {
        Self {
            accounts,
            account_names,
            exchange_rate: exchange_rate.clone(),
        }
    }

    /// Adds a new account and its name to the generator.
    pub fn add_account(&mut self, account: Arc<Account>, account_name: String) {
        self.accounts.push(account);
        self.account_names.push(account_name);
    }
}

impl AccountStatementGenerator {
    /// convert btc amount to fiat string
    pub fn convert_btc_to_fiat_string(&self, btc_amoount: f64, exchange_rate: &Option<ApiExchangeRate>) -> String {
        let amount_in_fiat_string = match exchange_rate {
            Some(ref rate) => {
                let value = (rate.ExchangeRate as f64) * btc_amoount / (rate.Cents as f64);
                format!("{:.2} {}", value, rate.FiatCurrency)
            }
            None => "".to_string(),
        };

        amount_in_fiat_string
    }

    pub async fn generate_csv(&self, export_time: u64) -> Result<String, Error> {
        let mut csv_text = String::new();
        csv_text.push_str("Type,Buy,Cur.,Sell,Cur.,Fee,Cur.,Exchange,Group,Comment,Date\n");

        for (acc, acc_name) in self.accounts.iter().zip(self.account_names.iter()) {
            let result = acc.get_last_block_id_before_time(export_time).await;

            // Get account balance and transactions
            if let Some(block_id) = result {
                let transactions = acc.get_transactions_at_block(block_id).await?;
                for transaction in transactions.iter().rev() {
                    let is_sent = transaction.sent > transaction.received;
                    let amount = if is_sent {
                        // we need to exclude fee since we already include fee field in csv output
                        transaction.sent - transaction.received - transaction.fees.unwrap_or(0)
                    } else {
                        transaction.received - transaction.sent
                    };
                    let amount_in_btc = (amount as f64) / (BITCOIN as f64);
                    let amount_in_btc_text = format!("{:.8}", amount_in_btc);

                    // Get transaction time
                    let datetime = Utc
                        .timestamp_opt(transaction.get_time() as i64, 0)
                        .single()
                        .ok_or("Wrong datetime format")
                        .map_err(|_| Error::AccountExportDatetimeError)?;
                    let transaction_date = format!(
                        "{}-{}-{} {}",
                        datetime.year(),
                        datetime.month(),
                        datetime.day(),
                        datetime.format("%H:%M:%S")
                    );

                    // Set buy/sell values
                    let transaction_type = if is_sent { "Spend" } else { "Income" };
                    let (buy_amount, buy_currency, sell_amount, sell_currency) = if is_sent {
                        ("", "", amount_in_btc_text.as_str(), "BTC")
                    } else {
                        (amount_in_btc_text.as_str(), "BTC", "", "")
                    };

                    let fee = transaction.fees.unwrap_or(0);
                    let fee_amount = format!("{:.8}", (fee as f64) / BITCOIN as f64);
                    let fee_currency = "BTC";
                    let exchange = "";
                    let comment = acc_name;
                    let group = "";

                    // Format row for CSV
                    let row = format!(
                        "{},{},{},{},{},{},{},{},{},{},{}\n",
                        transaction_type,
                        buy_amount,
                        buy_currency,
                        sell_amount,
                        sell_currency,
                        fee_amount,
                        fee_currency,
                        exchange,
                        group,
                        comment,
                        transaction_date,
                    );
                    csv_text.push_str(&row);
                }
            }
        }

        Ok(csv_text)
    }

    /// Returns typst report in String
    pub async fn generate_typst_text(&self, export_time: u64) -> Result<String, Error> {
        let mut typst_text = String::new();

        // setup font and align
        typst_text += r#"#set text(font: "noto sans tc")"#;
        typst_text += "\n";
        typst_text += r#"#set page(footer: "Proton Wallet")"#;
        typst_text += "\n";
        typst_text += "#set align(center)\n";
        typst_text += "= Account Statement\n";

        let datetime = Utc
            .timestamp_opt(export_time as i64, 0)
            .single()
            .ok_or("Wrong datetime format")
            .map_err(|_| Error::AccountExportDatetimeError)?;
        let export_date = format!("{} {}, {}", datetime.format("%B"), datetime.day(), datetime.year());
        typst_text += format!("Report Export Date: {}\n\n\n", export_date).as_str();

        typst_text += "#line(length: 100%)\n\n\n";
        typst_text += "#set align(left)\n";
        typst_text += "== Total Account Value\n";

        // calculate total balance of all accounts
        let mut total_balance = 0.0;
        for acc in self.accounts.iter() {
            let result = acc.get_last_block_id_before_time(export_time).await;
            let balance = match result {
                Some(block_id) => acc.get_balance_at_block(block_id).await?,
                None => BdkBalance {
                    immature: Amount::ZERO,
                    trusted_pending: Amount::ZERO,
                    untrusted_pending: Amount::ZERO,
                    confirmed: Amount::ZERO,
                },
            };
            total_balance += balance.confirmed.to_btc();
        }

        let balance_in_fiat = self.convert_btc_to_fiat_string(total_balance, &self.exchange_rate);
        let balance_text = format!("{:.8} BTC\n", total_balance);
        typst_text += format!("{} #h(1cm) {}\n", balance_text, balance_in_fiat).as_str();
        typst_text += "#line(length: 100%)\n\n\n";

        // generate account balance and transaction table for each account
        for (acc, acc_name) in self.accounts.iter().zip(self.account_names.iter()) {
            typst_text += format!("== {} Balance\n", acc_name).as_str();
            let result = acc.get_last_block_id_before_time(export_time).await;
            let balance = match result {
                Some(block_id) => acc.get_balance_at_block(block_id).await?,
                None => BdkBalance {
                    immature: Amount::ZERO,
                    trusted_pending: Amount::ZERO,
                    untrusted_pending: Amount::ZERO,
                    confirmed: Amount::ZERO,
                },
            };
            let total_balance = balance.confirmed.to_btc();
            let balance_in_fiat = self.convert_btc_to_fiat_string(total_balance, &self.exchange_rate);
            let balance_text = format!("{:.8} BTC\n", total_balance);
            typst_text += format!("{} #h(1cm) {}\n", balance_text, balance_in_fiat).as_str();
            typst_text += "\n\n";

            if result.is_none() {
                typst_text += "No transaction found\\\n";
                continue;
            }
            // only generate transaction table if we can find a block id before given time
            if let Some(block_id) = result {
                let transactions = acc.get_transactions_at_block(block_id).await?;
                if !transactions.is_empty() {
                    typst_text += format!("{} transactions found\\\n", transactions.len()).as_str();

                    typst_text += "#table(\n";
                    typst_text += "columns: (1fr, auto, auto, auto),\n";
                    typst_text += "[*Date*], [*Transaction Type*], [*Amount*], [*Account Balance*]";
                    let mut cumulative_balance = 0.0;

                    // build cumulative balance since it needs to be asc order from transactions
                    let mut cumulative_balances: Vec<f64> = Vec::new();
                    for transaction in transactions.iter().rev() {
                        let is_sent = transaction.sent > transaction.received;
                        let amount = if is_sent {
                            transaction.sent - transaction.received
                        } else {
                            transaction.received - transaction.sent
                        };
                        let amount_in_btc = (amount as f64) / (BITCOIN as f64);
                        cumulative_balance += if is_sent { -amount_in_btc } else { amount_in_btc };
                        cumulative_balances.push(cumulative_balance);
                    }
                    // reverse the cumulative balances to match the transactions order
                    cumulative_balances = cumulative_balances.into_iter().rev().collect();

                    for (index, transaction) in transactions.iter().enumerate() {
                        typst_text += ",\n";
                        let datetime = Utc
                            .timestamp_opt(transaction.get_time() as i64, 0)
                            .single()
                            .ok_or("Wrong datetime format")
                            .map_err(|_| Error::AccountExportDatetimeError)?;
                        let transaction_date_text =
                            format!("{} {}, {}", datetime.format("%B"), datetime.day(), datetime.year());
                        let is_sent = transaction.sent > transaction.received;
                        let amount = if is_sent {
                            transaction.sent - transaction.received
                        } else {
                            transaction.received - transaction.sent
                        };
                        let amount_in_btc = (amount as f64) / (BITCOIN as f64);
                        let amount_in_fiat = self.convert_btc_to_fiat_string(amount_in_btc, &self.exchange_rate);
                        let transaction_type_text = if is_sent { "Spend" } else { "Income" };

                        let cumulative_balance_in_fiat =
                            self.convert_btc_to_fiat_string(cumulative_balances[index], &self.exchange_rate);

                        if self.exchange_rate.is_some() {
                            typst_text += format!(
                                "[{}], [{}], [{:.8} BTC \\ ({})], [{:.8} BTC \\ ({})]",
                                transaction_date_text,
                                transaction_type_text,
                                amount_in_btc,
                                amount_in_fiat,
                                cumulative_balances[index],
                                cumulative_balance_in_fiat
                            )
                            .as_str();
                        } else {
                            typst_text += format!(
                                "[{}], [{}], [{:.8} BTC], [{:.8} BTC]",
                                transaction_date_text, transaction_type_text, amount_in_btc, cumulative_balances[index],
                            )
                            .as_str();
                        }
                    }
                    typst_text += ")\n\n";
                }
            }
        }

        println!("{}", typst_text);
        Ok(typst_text)
    }

    /// Returns report PDF data in bytes
    pub async fn to_pdf(&self, export_time: u64) -> Result<Vec<u8>, Error> {
        let pdf_generator = PDFGenerator::new();
        let typst_text = self.generate_typst_text(export_time).await?;
        let pdf_data = pdf_generator.generate_pdf_from_typst_text(typst_text)?;

        Ok(pdf_data)
    }

    /// Returns report CSV data in bytes
    pub async fn to_csv(&self, export_time: u64) -> Result<Vec<u8>, Error> {
        let csv_data = self.generate_csv(export_time).await?;

        Ok(csv_data.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use andromeda_api::tests::utils::common_api_client;
    use andromeda_bitcoin::{
        account_syncer::AccountSyncer, blockchain_client::BlockchainClient,
        tests::utils::tests::set_test_wallet_account,
    };
    use andromeda_common::{Network, ScriptType};
    use bitcoin::NetworkKind;

    use super::*;

    fn set_test_account_regtest(script_type: ScriptType, derivation_path: &str) -> Account {
        set_test_wallet_account(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower",
            script_type,
            derivation_path,
            None,
            None,
            Some(Network::Regtest),
            Some(NetworkKind::Test),
        )
    }

    #[test]
    fn test_convert_btc_to_fiat_string() {
        let generator = AccountStatementGenerator::new(vec![], vec![], None);
        let fiat_string = generator.convert_btc_to_fiat_string(1.0, &None);
        assert_eq!(fiat_string, "");

        let exchange_rate = Some(ApiExchangeRate {
            ExchangeRate: 8116517,
            ID: "My Exchange Rate ID".to_string(),
            BitcoinUnit: andromeda_common::BitcoinUnit::BTC,
            FiatCurrency: andromeda_api::settings::FiatCurrencySymbol::EUR,
            Sign: Some("$".to_string()),
            ExchangeRateTime: "2025-03-26 12:00:00".to_string(),
            Cents: 100,
        });
        let fiat_string = generator.convert_btc_to_fiat_string(1.0, &exchange_rate);
        assert_eq!(fiat_string, "81165.17 EUR");
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_typst_text() {
        let generator = AccountStatementGenerator::new(vec![], vec![], None);
        let typst_text = generator.generate_typst_text(0).await.unwrap();
        assert_ne!(typst_text, "");
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_csv() {
        let account1 = Arc::new(set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'"));
        let account2 = Arc::new(set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/1'"));
        let api_client = common_api_client().await;
        let client = Arc::new(BlockchainClient::new(api_client));
        let sync = AccountSyncer::new(client, account1.clone());
        // do full sync
        sync.full_sync(None).await.unwrap();

        let exchange_rate = Some(ApiExchangeRate {
            ExchangeRate: 8116517,
            ID: "My Exchange Rate ID".to_string(),
            BitcoinUnit: andromeda_common::BitcoinUnit::BTC,
            FiatCurrency: andromeda_api::settings::FiatCurrencySymbol::EUR,
            Sign: Some("$".to_string()),
            ExchangeRateTime: "2025-03-26 12:00:00".to_string(),
            Cents: 100,
        });

        let mut account_statement_generator = AccountStatementGenerator::new(vec![], vec![], exchange_rate);
        account_statement_generator.add_account(account1, "My Wallet - Primary Account".to_string());
        account_statement_generator.add_account(account2, "My Wallet - BvE Account".to_string());
        let csv = account_statement_generator.generate_csv(1742964438).await.unwrap();
        println!("{}", csv);
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_pdf() {
        let account1 = Arc::new(set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'"));
        let account2 = Arc::new(set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/1'"));
        let api_client = common_api_client().await;
        let client = Arc::new(BlockchainClient::new(api_client));
        let sync = AccountSyncer::new(client, account1.clone());
        // do full sync
        sync.full_sync(None).await.unwrap();

        let exchange_rate = Some(ApiExchangeRate {
            ExchangeRate: 8116517,
            ID: "My Exchange Rate ID".to_string(),
            BitcoinUnit: andromeda_common::BitcoinUnit::BTC,
            FiatCurrency: andromeda_api::settings::FiatCurrencySymbol::EUR,
            Sign: Some("$".to_string()),
            ExchangeRateTime: "2025-03-26 12:00:00".to_string(),
            Cents: 100,
        });

        let mut account_statement_generator = AccountStatementGenerator::new(vec![], vec![], exchange_rate);
        account_statement_generator.add_account(account1, "My Wallet - Primary Account".to_string());
        account_statement_generator.add_account(account2, "My Wallet - BvE Account".to_string());
        let pdf_data = account_statement_generator.to_pdf(1742964438).await.unwrap();
        let _ = std::fs::write("../../test_account_statementreport.pdf", pdf_data);
    }
}
