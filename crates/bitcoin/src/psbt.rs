use std::{collections::HashMap, fmt::Debug};

use bdk_esplora::esplora_client::TxIn;
use bdk_wallet::{
    bitcoin::psbt::Psbt as BdkPsbt,
    wallet::coin_selection::{self, decide_change, CoinSelectionAlgorithm, CoinSelectionResult},
    LocalOutput, Utxo, WeightedUtxo,
};
use bitcoin::{Amount, FeeRate, Script, Transaction, Weight};

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Psbt(BdkPsbt);

impl From<BdkPsbt> for Psbt {
    fn from(value: BdkPsbt) -> Self {
        Psbt(value)
    }
}

impl Psbt {
    pub fn new(psbt: BdkPsbt) -> Self {
        Psbt(psbt)
    }

    pub fn inner(&self) -> BdkPsbt {
        self.0.clone()
    }

    pub fn extract_tx(&self) -> Result<Transaction, Error> {
        Ok(self.0.clone().extract_tx()?)
    }

    pub fn fee(&self) -> Result<Amount, Error> {
        Ok(self.0.clone().fee()?)
    }

    pub fn compute_tx_size(&self) -> Result<usize, Error> {
        Ok(self.extract_tx()?.total_size())
    }
}

// Taken from bdk, the function is not public there
fn select_sorted_utxos(
    utxos: impl Iterator<Item = (bool, WeightedUtxo)>,
    fee_rate: FeeRate,
    target_amount: u64,
    drain_script: &Script,
) -> Result<CoinSelectionResult, coin_selection::Error> {
    let mut selected_amount = 0;
    let mut fee_amount = 0;
    let selected = utxos
        .scan(
            (&mut selected_amount, &mut fee_amount),
            |(selected_amount, fee_amount), (must_use, weighted_utxo)| {
                if must_use || **selected_amount < target_amount + **fee_amount {
                    **fee_amount += (fee_rate
                        * Weight::from_wu(
                            TxIn::default().segwit_weight().to_wu() + weighted_utxo.satisfaction_weight as u64,
                        ))
                    .to_sat();
                    **selected_amount += weighted_utxo.utxo.txout().value.to_sat();
                    Some(weighted_utxo.utxo)
                } else {
                    None
                }
            },
        )
        .collect::<Vec<_>>();

    let amount_needed_with_fees = target_amount + fee_amount;
    if selected_amount < amount_needed_with_fees {
        return Err(coin_selection::Error::InsufficientFunds {
            needed: amount_needed_with_fees,
            available: selected_amount,
        });
    }

    let remaining_amount = selected_amount - amount_needed_with_fees;

    let excess = decide_change(remaining_amount, fee_rate, drain_script);

    Ok(CoinSelectionResult {
        selected,
        fee_amount,
        excess,
    })
}

#[derive(Debug)]
pub struct TemplatePsbtCoinSelection(pub Psbt, pub HashMap<String, (LocalOutput, usize)>);

impl CoinSelectionAlgorithm for TemplatePsbtCoinSelection {
    fn coin_select(
        &self,
        _required_utxos: Vec<bdk_wallet::WeightedUtxo>,
        _optional_utxos: Vec<bdk_wallet::WeightedUtxo>,
        fee_rate: bitcoin::FeeRate,
        target_amount: u64,
        drain_script: &bitcoin::Script,
    ) -> Result<bdk_wallet::wallet::coin_selection::CoinSelectionResult, bdk_wallet::wallet::coin_selection::Error>
    {
        let available_utxos = self.1.clone();

        let utxos = self
            .0
            .extract_tx()
            .unwrap()
            .input
            .into_iter()
            .map(|input| available_utxos.get(&input.previous_output.to_string()))
            .filter_map(|u| {
                u.map(|(output, wu)| {
                    (
                        false,
                        WeightedUtxo {
                            utxo: Utxo::Local(output.clone()),
                            satisfaction_weight: *wu,
                        },
                    )
                })
            });

        select_sorted_utxos(utxos, fee_rate, target_amount, drain_script)
    }
}
