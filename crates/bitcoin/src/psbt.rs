use std::fmt::Debug;

use bdk_wallet::bitcoin::psbt::Psbt as BdkPsbt;
use bitcoin::{Amount, Transaction};

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
}
