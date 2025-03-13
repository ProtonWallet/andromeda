use std::fmt::Debug;

use bdk_wallet::bitcoin::psbt::Psbt as BdkPsbt;
use bitcoin::{psbt::Error as PsbtError, Amount, Transaction};

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

    pub fn compute_tx_vbytes(&self) -> Result<u64, Error> {
        Ok(self.extract_tx()?.weight().to_vbytes_ceil())
    }

    pub fn outputs_amount(&self) -> Result<Amount, Error> {
        let mut outputs: u64 = 0;
        for out in self.0.clone().unsigned_tx.output {
            outputs = outputs.checked_add(out.value.to_sat()).ok_or(PsbtError::FeeOverflow)?;
        }
        Ok(Amount::from_sat(outputs))
    }
}
