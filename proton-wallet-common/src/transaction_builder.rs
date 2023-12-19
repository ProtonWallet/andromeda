use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};

use bdk::{
    bitcoin::ScriptBuf,
    chain::PersistBackend,
    wallet::{
        coin_selection::{
            BranchAndBoundCoinSelection, CoinSelectionAlgorithm, LargestFirstCoinSelection, OldestFirstCoinSelection,
        },
        tx_builder::{ChangeSpendPolicy, CreateTx, TxBuilder as BdkTxBuilder},
        Balance, ChangeSet,
    },
    FeeRate,
};
use miniscript::bitcoin::{
    absolute::LockTime, psbt::PartiallySignedTransaction, script::PushBytesBuf, Address, OutPoint,
};
use uuid::Uuid;

use crate::{account::Account, error::Error};

#[derive(Clone, Debug)]
pub enum CoinSelection {
    BranchAndBound,
    LargestFirst,
    OldestFirst,
    Manual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TmpRecipient(pub String, pub String, pub u64);

#[derive(Clone, Debug)]
pub struct TxBuilder<Storage> {
    account: Option<Arc<Mutex<Account<Storage>>>>,
    pub recipients: Vec<TmpRecipient>,
    pub utxos_to_spend: HashSet<OutPoint>,
    pub change_policy: ChangeSpendPolicy,
    pub fee_rate: Option<FeeRate>,
    pub drain_wallet: bool,
    pub drain_to: Option<ScriptBuf>,
    pub rbf_enabled: bool,
    pub data: Vec<u8>,
    pub coin_selection: CoinSelection,
    pub locktime: Option<LockTime>,
}

pub struct ScriptAmount {
    pub script: ScriptBuf,
    pub amount: u64,
}

struct AllocateBalanceAcc {
    remaining: u64,
    recipients: Vec<TmpRecipient>,
}

/**
 * This functions allocates a given balance accross the provided recipients.
 * If recipients total amount is greater than provided balance, the last recipients will be allocated less than initially.
 * "First come, first served"
 */
fn allocate_recipients_balance(recipients: Vec<TmpRecipient>, balance: &Balance) -> Vec<TmpRecipient> {
    let acc_result: AllocateBalanceAcc = recipients.into_iter().fold(
        AllocateBalanceAcc {
            remaining: balance.confirmed,
            recipients: Vec::new(),
        },
        |acc, current| {
            // If remainingAmount = 123 and recipient's amount is 100, we'll allocate 100
            // If remainingAmount = 73 and recipient's amount is 100, we'll allocate only 73
            let amount_to_allocate = std::cmp::min(acc.remaining, current.2);
            let next_remaining = acc.remaining - amount_to_allocate;

            let mut next_recipients = acc.recipients.clone();
            next_recipients.extend(vec![TmpRecipient(current.0, current.1, amount_to_allocate)]);

            AllocateBalanceAcc {
                remaining: next_remaining,
                recipients: next_recipients,
            }
        },
    );

    acc_result.recipients
}

/**
 * This function remove allocated amount from the last recipient to the first one and returns an array of updated recipients
 */
fn correct_recipients_amounts(recipients: Vec<TmpRecipient>, amount_to_remove: u64) -> Vec<TmpRecipient> {
    let mut cloned = recipients.clone();
    cloned.reverse(); // R3 R2 R1

    let acc_result: AllocateBalanceAcc = cloned.into_iter().fold(
        AllocateBalanceAcc {
            remaining: amount_to_remove,
            recipients: Vec::new(),
        },
        |acc, current| {
            let new_amount: i64 = current.2 as i64 - acc.remaining as i64;
            // If new_amount is null or negative, we have unallocated all necessary balance
            let new_remaining_to_remove = if new_amount > 0 { 0 } else { -new_amount as u64 };

            let mut next_recipients = vec![TmpRecipient(current.0, current.1, std::cmp::max(new_amount, 0) as u64)];
            next_recipients.extend(acc.recipients.clone());

            AllocateBalanceAcc {
                remaining: new_remaining_to_remove,
                recipients: next_recipients,
            }
        },
    );

    acc_result.recipients
}

impl<Storage> TxBuilder<Storage>
where
    Storage: PersistBackend<ChangeSet> + Clone,
{
    pub fn new() -> Self {
        TxBuilder::<Storage> {
            account: None,
            recipients: vec![TmpRecipient(Uuid::new_v4().to_string(), String::new(), 0)],
            utxos_to_spend: HashSet::new(),
            change_policy: ChangeSpendPolicy::ChangeAllowed,
            fee_rate: None,
            drain_wallet: false,
            drain_to: None,
            rbf_enabled: true,
            locktime: None,
            coin_selection: CoinSelection::BranchAndBound,
            data: Vec::new(),
        }
    }

    pub fn set_account(&self, account: Arc<Mutex<Account<Storage>>>) -> Self {
        let balance = &account.lock().unwrap().get_balance();

        let tx_builder = TxBuilder::<Storage> {
            account: Some(account),
            recipients: allocate_recipients_balance(self.recipients.clone(), balance),
            ..self.clone()
        };

        tx_builder.constrain_recipient_amounts()
    }

    /// Add a recipient to the internal list.
    pub fn add_recipient(&self) -> Self {
        let mut recipients = self.recipients.clone();
        recipients.append(&mut vec![TmpRecipient(Uuid::new_v4().to_string(), String::new(), 0)]);

        TxBuilder::<Storage> {
            recipients,
            ..self.clone()
        }
    }

    /// Remove a recipient from the internal list.
    pub fn remove_recipient(&self, index: usize) -> Self {
        let mut recipients = self.recipients.clone();

        if index < recipients.len() {
            recipients.remove(index);
        }

        TxBuilder::<Storage> {
            recipients,
            ..self.clone()
        }
    }

    fn constrain_recipient_amounts(&self) -> Self {
        if self.account.is_none() {
            return self.clone();
        } else {
            let result = self.create_pbst_with_coin_selection(true);

            match result {
                Err(Error::InsufficientFunds { needed, available }) => {
                    let amount_to_remove = needed - available;

                    TxBuilder::<Storage> {
                        recipients: correct_recipients_amounts(self.recipients.clone(), amount_to_remove),
                        ..self.clone()
                    }
                }
                _ => self.clone(),
            }
        }
    }

    /// Remove a recipient from the internal list.
    pub fn update_recipient(&self, index: usize, update: (Option<String>, Option<u64>)) -> Self {
        let mut recipients = self.recipients.clone();
        let TmpRecipient(uuid, current_script, current_amount) = recipients[index].clone();

        recipients[index] = TmpRecipient(
            uuid,
            match update.0 {
                Some(address_str) => address_str,
                _ => current_script,
            },
            match update.1 {
                Some(amount) => amount,
                _ => current_amount,
            },
        );

        let tx_builder = TxBuilder::<Storage> {
            recipients,
            ..self.clone()
        };

        tx_builder.constrain_recipient_amounts()
    }

    /**
     * UTXOs
     */

    /// Add the list of outpoints to the internal list of UTXOs that must be spent. If an error occurs while adding
    /// any of the UTXOs then none of them are added and the error is returned. These have priority over the "unspendable"
    /// utxos, meaning that if a utxo is present both in the "utxos" and the "unspendable" list, it will be spent.
    pub fn add_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.insert(utxo_to_spend.clone());

        TxBuilder::<Storage> {
            utxos_to_spend,
            ..self.clone()
        }
    }

    pub fn remove_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.remove(utxo_to_spend);

        TxBuilder::<Storage> {
            utxos_to_spend,
            ..self.clone()
        }
    }

    pub fn clear_utxos_to_spend(&self) -> Self {
        TxBuilder::<Storage> {
            utxos_to_spend: HashSet::new(),
            ..self.clone()
        }
    }

    /**
     * Coin selection enforcement
     */

    pub fn set_coin_selection(&self, coin_selection: CoinSelection) -> Self {
        TxBuilder::<Storage> {
            coin_selection,
            ..self.clone()
        }
    }

    /**
     * Enable RBF
     */

    pub fn enable_rbf(&self) -> Self {
        TxBuilder::<Storage> {
            rbf_enabled: true,
            ..self.clone()
        }
    }

    pub fn disable_rbf(&self) -> Self {
        TxBuilder::<Storage> {
            rbf_enabled: false,
            ..self.clone()
        }
    }

    /**
     * Locktime
     */

    pub fn add_locktime(&self, locktime: LockTime) -> Self {
        TxBuilder::<Storage> {
            locktime: Some(locktime),
            ..self.clone()
        }
    }

    pub fn remove_locktime(&self) -> Self {
        TxBuilder::<Storage> {
            locktime: None,
            ..self.clone()
        }
    }

    /**
     * Change policy
     */

    /// Do not spend change outputs. This effectively adds all the change outputs to the "unspendable" list. See TxBuilder.unspendable.
    pub fn set_change_policy(&self, change_policy: ChangeSpendPolicy) -> Self {
        TxBuilder::<Storage> {
            change_policy,
            ..self.clone()
        }
    }

    /**
     * Fees
     */

    /// Set a custom fee rate.
    pub fn set_fee_rate(&self, sat_per_vb: f32) -> Self {
        let tx_builder = TxBuilder::<Storage> {
            fee_rate: Some(FeeRate::from_sat_per_vb(sat_per_vb)),
            ..self.clone()
        };

        tx_builder.constrain_recipient_amounts()
    }

    fn commit_utxos<'a, D: PersistBackend<ChangeSet>, Cs: CoinSelectionAlgorithm>(
        &self,
        mut tx_builder: BdkTxBuilder<'a, D, Cs, CreateTx>,
    ) -> Result<BdkTxBuilder<'a, D, Cs, CreateTx>, Error> {
        if !self.utxos_to_spend.is_empty() {
            let bdk_utxos: Vec<OutPoint> = self
                .utxos_to_spend
                .iter()
                .map(|utxo| OutPoint::from(utxo.clone()))
                .collect();
            let utxos: &[OutPoint] = &bdk_utxos;
            tx_builder.add_utxos(utxos).map_err(|e| e.into())?;
        }

        Ok(tx_builder)
    }

    fn create_psbt<'a, D: PersistBackend<ChangeSet>, Cs: CoinSelectionAlgorithm>(
        &self,
        mut tx_builder: BdkTxBuilder<'a, D, Cs, CreateTx>,
        allow_dust: bool,
    ) -> Result<PartiallySignedTransaction, Error> {
        for TmpRecipient(_uuid, address, amount) in &self.recipients {
            tx_builder.add_recipient(
                Address::from_str(&address)
                    .map_err(|_| Error::InvalidAddress)?
                    .assume_checked()
                    .script_pubkey(),
                *amount,
            );
        }

        tx_builder.change_policy(self.change_policy);

        if let Some(fee_rate) = self.fee_rate {
            tx_builder.fee_rate(fee_rate);
        }

        tx_builder.allow_dust(allow_dust);

        if self.drain_wallet {
            tx_builder.drain_wallet();
        }

        if self.rbf_enabled {
            tx_builder.enable_rbf();
        }

        if !&self.data.is_empty() {
            let mut buf = PushBytesBuf::new();
            buf.extend_from_slice(self.data.as_slice())
                .map_err(|_| Error::InvalidData)?;

            tx_builder.add_data(&buf.as_push_bytes());
        }

        let psbt = tx_builder.finish().map_err(|e| e.into())?;

        Ok(psbt.into())
    }

    pub fn create_pbst_with_coin_selection(&self, allow_dust: bool) -> Result<PartiallySignedTransaction, Error> {
        let account = self.account.clone().ok_or(Error::NoRecipients)?;
        let mut account = account.lock().unwrap();
        let wallet = account.get_mutable_wallet();

        match self.coin_selection {
            CoinSelection::BranchAndBound => {
                let tx_builder = wallet.build_tx().coin_selection(BranchAndBoundCoinSelection::default());
                self.create_psbt(tx_builder, allow_dust)
            }
            CoinSelection::LargestFirst => {
                let tx_builder = wallet.build_tx().coin_selection(LargestFirstCoinSelection::default());
                self.create_psbt(tx_builder, allow_dust)
            }
            CoinSelection::OldestFirst => {
                let tx_builder = wallet.build_tx().coin_selection(OldestFirstCoinSelection::default());
                self.create_psbt(tx_builder, allow_dust)
            }
            CoinSelection::Manual => {
                let mut tx_builder = wallet.build_tx().coin_selection(BranchAndBoundCoinSelection::default());

                tx_builder = self.commit_utxos(tx_builder)?;
                self.create_psbt(tx_builder, allow_dust)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{correct_recipients_amounts, TmpRecipient};

    /**
     * correct_recipients_amounts
     */

    #[test]
    fn should_remove_correct_amount() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = correct_recipients_amounts(recipients, 3400);

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
                TmpRecipient("2".to_string(), "addr2".to_string(), 1700),
                TmpRecipient("3".to_string(), "addr3".to_string(), 0)
            ]
        );
    }

    #[test]
    fn should_not_create_negative_amounts() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = correct_recipients_amounts(recipients, 9000);

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), 0),
                TmpRecipient("2".to_string(), "addr2".to_string(), 0),
                TmpRecipient("3".to_string(), "addr3".to_string(), 0)
            ]
        );
    }

    #[test]
    fn should_not_remove_anything() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = correct_recipients_amounts(recipients.clone(), 0);

        assert_eq!(updated, recipients);
    }
}
