use std::collections::HashSet;

use bdk::{bitcoin::ScriptBuf as BdkScriptBuf, wallet::tx_builder::ChangeSpendPolicy, FeeRate};
use miniscript::bitcoin::{psbt::PartiallySignedTransaction, script::PushBytesBuf, OutPoint};

use crate::{account::Account, bitcoin::Script, error::Error};

#[derive(Clone, Debug)]
pub struct TxBuilder {
    pub recipients: Vec<(BdkScriptBuf, u64)>,
    pub utxos_to_spend: HashSet<OutPoint>,
    pub unspendable_utxos: HashSet<OutPoint>,
    pub change_policy: ChangeSpendPolicy,
    pub manually_selected_only: bool,
    pub fee_rate: Option<f32>,
    pub fee_absolute: Option<u64>,
    pub drain_wallet: bool,
    pub drain_to: Option<BdkScriptBuf>,
    pub rbf_enabled: bool,
    pub data: Vec<u8>,
}

pub struct ScriptAmount {
    pub script: Script,
    pub amount: u64,
}

impl TxBuilder {
    pub fn new() -> Self {
        TxBuilder {
            recipients: Vec::new(),
            utxos_to_spend: HashSet::new(),
            unspendable_utxos: HashSet::new(),
            change_policy: ChangeSpendPolicy::ChangeAllowed,
            manually_selected_only: false,
            fee_rate: None,
            fee_absolute: None,
            drain_wallet: false,
            drain_to: None,
            rbf_enabled: true,
            data: Vec::new(),
        }
    }

    /// Add a recipient to the internal list.
    pub fn add_recipient(&self, script: Script, amount: u64) -> Self {
        let mut recipients: Vec<(BdkScriptBuf, u64)> = self.recipients.clone();
        recipients.append(&mut vec![(script.0.clone(), amount)]);

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    /// Remove a recipient from the internal list.
    pub fn remove_recipient(&self, index: usize) -> Self {
        let mut recipients: Vec<(BdkScriptBuf, u64)> = self.recipients.clone();

        if index < recipients.len() {
            recipients.remove(index);
        }

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    /// Remove a recipient from the internal list.
    pub fn update_recipient(&self, index: usize, update: (Option<Script>, Option<u64>)) -> Self {
        let mut recipients: Vec<(BdkScriptBuf, u64)> = self.recipients.clone();
        let (current_script, current_amount) = recipients[index].clone();

        recipients[index] = (
            match update.0 {
                Some(script) => script.0,
                _ => current_script,
            },
            match update.1 {
                Some(amount) => amount,
                _ => current_amount,
            },
        );

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    /**
     * UTXOs
     */

    /// Add a utxo to the internal list of unspendable utxos. It’s important to note that the "must-be-spent"
    /// utxos added with [TxBuilder.addUtxo] have priority over this. See the Rust docs of the two linked methods for more details.
    pub fn add_unspendable_utxo(&self, unspendable_utxo: &OutPoint) -> Self {
        let mut unspendable_utxos = self.unspendable_utxos.clone();
        unspendable_utxos.insert(unspendable_utxo.clone());

        TxBuilder {
            unspendable_utxos,
            ..self.clone()
        }
    }

    pub fn remove_unspendable_utxo(&self, unspendable_utxo: &OutPoint) -> Self {
        let mut unspendable_utxos = self.unspendable_utxos.clone();
        unspendable_utxos.remove(unspendable_utxo);

        TxBuilder {
            unspendable_utxos,
            ..self.clone()
        }
    }

    /// Add the list of outpoints to the internal list of UTXOs that must be spent. If an error occurs while adding
    /// any of the UTXOs then none of them are added and the error is returned. These have priority over the "unspendable"
    /// utxos, meaning that if a utxo is present both in the "utxos" and the "unspendable" list, it will be spent.
    pub fn add_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.insert(utxo_to_spend.clone());

        TxBuilder {
            utxos_to_spend,
            ..self.clone()
        }
    }

    pub fn remove_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.remove(utxo_to_spend);

        TxBuilder {
            utxos_to_spend,
            ..self.clone()
        }
    }

    /**
     * Coin selection enforcement
     */

    /// Only spend utxos added by [add_utxo]. The wallet will not add additional utxos to the transaction even if they are
    /// needed to make the transaction valid.
    pub fn manually_selected_only(&self) -> Self {
        TxBuilder {
            manually_selected_only: true,
            ..self.clone()
        }
    }

    /**
     * Change policy
     */

    /// Do not spend change outputs. This effectively adds all the change outputs to the "unspendable" list. See TxBuilder.unspendable.
    pub fn do_not_spend_change(&self) -> Self {
        TxBuilder {
            change_policy: ChangeSpendPolicy::ChangeForbidden,
            ..self.clone()
        }
    }

    /// Only spend change outputs. This effectively adds all the non-change outputs to the "unspendable" list. See TxBuilder.unspendable.
    pub fn only_spend_change(&self) -> Self {
        TxBuilder {
            change_policy: ChangeSpendPolicy::OnlyChange,
            ..self.clone()
        }
    }

    // Use both change and non-change outputs (default)
    // TODO: check what it is does in the unspendable list
    pub fn allow_spend_both(&self) -> Self {
        TxBuilder {
            change_policy: ChangeSpendPolicy::ChangeAllowed,
            ..self.clone()
        }
    }

    /**
     * Fees
     */

    /// Set a custom fee rate.
    pub fn fee_rate(&self, sat_per_vb: f32) -> Self {
        TxBuilder {
            fee_rate: Some(sat_per_vb),
            fee_absolute: None,
            ..self.clone()
        }
    }

    /// Set an absolute fee.
    pub fn fee_absolute(&self, fee_amount: u64) -> Self {
        TxBuilder {
            fee_absolute: Some(fee_amount),
            fee_rate: None,
            ..self.clone()
        }
    }

    pub fn create_pbst(&self, account: &Account) -> Result<PartiallySignedTransaction, Error> {
        let mut wallet = account.wallet();
        let mut tx_builder = wallet.build_tx();

        for (script, amount) in &self.recipients {
            tx_builder.add_recipient(script.clone(), *amount);
        }
        tx_builder.change_policy(self.change_policy);

        if !self.utxos_to_spend.is_empty() {
            let bdk_utxos: Vec<OutPoint> = self
                .utxos_to_spend
                .iter()
                .map(|utxo| OutPoint::from(utxo.clone()))
                .collect();
            let utxos: &[OutPoint] = &bdk_utxos;
            tx_builder.add_utxos(utxos).map_err(|e| e.into())?;
        }

        if !self.unspendable_utxos.is_empty() {
            let bdk_unspendable: Vec<OutPoint> = self
                .unspendable_utxos
                .iter()
                .map(|utxo| OutPoint::from(utxo.clone()))
                .collect();

            tx_builder.unspendable(bdk_unspendable);
        }

        if self.manually_selected_only {
            tx_builder.manually_selected_only();
        }

        if let Some(sat_per_vb) = self.fee_rate {
            tx_builder.fee_rate(FeeRate::from_sat_per_vb(sat_per_vb));
        } else if let Some(fee_amount) = self.fee_absolute {
            tx_builder.fee_absolute(fee_amount);
        }

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
}
