use std::str::FromStr;

use proton_wallet_common::{bitcoin::Script, transaction_builder::TxBuilder, OutPoint, PartiallySignedTransaction};
use wasm_bindgen::prelude::*;

use crate::{
    account::WasmAccount,
    error::{DetailledWasmError, WasmError},
};

/**
 * Maybe to put in other file
 */

#[wasm_bindgen(getter_with_clone)]
pub struct WasmScript(pub Vec<u8>);

impl Into<Script> for WasmScript {
    fn into(self) -> Script {
        Script::new(self.0)
    }
}

#[wasm_bindgen(getter_with_clone)]
/// Serialised Outpoint under the form <txid>:<index>
pub struct WasmOutPoint(String);

impl TryInto<OutPoint> for WasmOutPoint {
    type Error = WasmError;

    fn try_into(self) -> Result<OutPoint, WasmError> {
        OutPoint::from_str(&self.0).map_err(|_| WasmError::OutpointParsingError)
    }
}

// ----

#[wasm_bindgen]
pub struct WasmTxBuilder {
    inner: TxBuilder,
}

#[wasm_bindgen]
pub struct WasmPartiallySignedTransaction {
    inner: PartiallySignedTransaction,
}

#[wasm_bindgen]
impl WasmTxBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTxBuilder {
        WasmTxBuilder {
            inner: TxBuilder::new(),
        }
    }

    #[wasm_bindgen]
    pub fn add_recipient(&self, script: WasmScript, amount: u64) -> WasmTxBuilder {
        let inner = self.inner.add_recipient(script.into(), amount);
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen]
    pub fn remove_recipient(&mut self, index: usize) -> () {
        self.inner.remove_recipient(index);
    }

    #[wasm_bindgen]
    pub fn update_recipient(&mut self, index: usize, script: Option<WasmScript>, amount: Option<u64>) -> WasmTxBuilder {
        let script = match script {
            Some(script) => {
                let script: Script = script.into();
                Some(script)
            }
            _ => None,
        };

        let inner = self.inner.update_recipient(index, (script, amount));
        WasmTxBuilder { inner }
    }

    /**
     * UTXOs
     */

    #[wasm_bindgen]
    pub fn add_unspendable_utxo(&self, outpoint: WasmOutPoint) -> Result<WasmTxBuilder, WasmError> {
        let serialised: OutPoint = outpoint.try_into()?;
        let inner = self.inner.add_unspendable_utxo(&serialised);

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen]
    pub fn remove_unspendable_utxo(&self, outpoint: WasmOutPoint) -> Result<WasmTxBuilder, WasmError> {
        let serialised: OutPoint = outpoint.try_into()?;
        let inner = self.inner.remove_unspendable_utxo(&serialised);

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen]
    pub fn add_utxo_to_spend(&self, outpoint: WasmOutPoint) -> Result<WasmTxBuilder, WasmError> {
        let serialised: OutPoint = outpoint.try_into()?;
        let inner = self.inner.add_utxo_to_spend(&serialised);

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen]
    pub fn remove_utxo_to_spend(&self, outpoint: WasmOutPoint) -> Result<WasmTxBuilder, WasmError> {
        let serialised: OutPoint = outpoint.try_into()?;
        let inner = self.inner.remove_utxo_to_spend(&serialised);

        Ok(WasmTxBuilder { inner })
    }

    /**
     * Coin selection enforcement
     */

    #[wasm_bindgen]
    pub fn manually_selected_only(&self) -> WasmTxBuilder {
        let inner = self.inner.manually_selected_only();
        WasmTxBuilder { inner }
    }

    /**
     * Change policy
     */

    #[wasm_bindgen]
    pub fn do_not_spend_change(&self) -> WasmTxBuilder {
        let inner = self.inner.do_not_spend_change();
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen]
    pub fn only_spend_change(&self) -> WasmTxBuilder {
        let inner = self.inner.only_spend_change();
        WasmTxBuilder { inner }
    }

    pub fn allow_spend_both(&self) -> WasmTxBuilder {
        let inner = self.inner.allow_spend_both();
        WasmTxBuilder { inner }
    }

    /**
     * Fees
     */

    #[wasm_bindgen]
    pub fn fee_rate(&self, sat_per_vb: f32) -> WasmTxBuilder {
        let inner = self.inner.fee_rate(sat_per_vb);
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen]
    pub fn fee_absolute(&self, fee_amount: u64) -> WasmTxBuilder {
        let inner = self.inner.fee_absolute(fee_amount);
        WasmTxBuilder { inner }
    }

    /**
     * Final
     */

    #[wasm_bindgen]
    pub fn create_pbst(&self, account: WasmAccount) -> Result<WasmPartiallySignedTransaction, DetailledWasmError> {
        let inner = self.inner.create_pbst(&account.into()).map_err(|e| e.into())?;
        Ok(WasmPartiallySignedTransaction { inner })
    }
}
