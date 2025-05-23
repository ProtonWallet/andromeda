use andromeda_bitcoin::{
    transaction_builder::{CoinSelection, TmpRecipient, TxBuilder},
    ChangeSpendPolicy, OutPoint,
};
use wasm_bindgen::prelude::*;

use super::{
    account::WasmAccount,
    psbt::WasmPsbt,
    types::{locktime::WasmLockTime, transaction::WasmOutPoint},
};
use crate::common::{error::ErrorExt, types::WasmNetwork};

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmTxBuilder {
    inner: TxBuilder,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmCoinSelection {
    BranchAndBound,
    LargestFirst,
    OldestFirst,
    Manual,
}

impl From<WasmCoinSelection> for CoinSelection {
    fn from(val: WasmCoinSelection) -> Self {
        match val {
            WasmCoinSelection::BranchAndBound => CoinSelection::BranchAndBound,
            WasmCoinSelection::LargestFirst => CoinSelection::LargestFirst,
            WasmCoinSelection::OldestFirst => CoinSelection::OldestFirst,
            WasmCoinSelection::Manual => CoinSelection::Manual,
        }
    }
}

impl From<CoinSelection> for WasmCoinSelection {
    fn from(val: CoinSelection) -> Self {
        match val {
            CoinSelection::BranchAndBound => WasmCoinSelection::BranchAndBound,
            CoinSelection::LargestFirst => WasmCoinSelection::LargestFirst,
            CoinSelection::OldestFirst => WasmCoinSelection::OldestFirst,
            CoinSelection::Manual => WasmCoinSelection::Manual,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmChangeSpendPolicy {
    ChangeAllowed,
    OnlyChange,
    ChangeForbidden,
}

impl From<WasmChangeSpendPolicy> for ChangeSpendPolicy {
    fn from(val: WasmChangeSpendPolicy) -> Self {
        match val {
            WasmChangeSpendPolicy::ChangeAllowed => ChangeSpendPolicy::ChangeAllowed,
            WasmChangeSpendPolicy::OnlyChange => ChangeSpendPolicy::OnlyChange,
            WasmChangeSpendPolicy::ChangeForbidden => ChangeSpendPolicy::ChangeForbidden,
        }
    }
}

impl From<ChangeSpendPolicy> for WasmChangeSpendPolicy {
    fn from(val: ChangeSpendPolicy) -> Self {
        match val {
            ChangeSpendPolicy::ChangeAllowed => WasmChangeSpendPolicy::ChangeAllowed,
            ChangeSpendPolicy::OnlyChange => WasmChangeSpendPolicy::OnlyChange,
            ChangeSpendPolicy::ChangeForbidden => WasmChangeSpendPolicy::ChangeForbidden,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmRecipient(pub String, pub String, pub u64);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmPsbtAndTxBuilder(pub WasmPsbt, pub WasmTxBuilder);

#[wasm_bindgen]
impl WasmTxBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmTxBuilder {
        WasmTxBuilder {
            inner: TxBuilder::new(),
        }
    }

    #[wasm_bindgen(js_name = setAccount)]
    pub fn set_account(&self, account: &WasmAccount) -> WasmTxBuilder {
        let inner = self.inner.set_account(account.get_inner());
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = constrainRecipientAmounts)]
    pub async fn constrain_recipient_amounts(&self) -> Result<WasmTxBuilder, js_sys::Error> {
        let inner = self.inner.constrain_recipient_amounts().await;
        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen(js_name = clearRecipients)]
    pub fn clear_recipients(&self) -> WasmTxBuilder {
        let inner = self.inner.clear_recipients();
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = addRecipient)]
    pub fn add_recipient(&self, address_str: Option<String>, amount: Option<u64>) -> WasmTxBuilder {
        let inner = self.inner.add_recipient(Some((address_str, amount)));
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = removeRecipient)]
    pub fn remove_recipient(&self, index: usize) -> WasmTxBuilder {
        let inner = self.inner.remove_recipient(index);
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = updateRecipient)]
    pub fn update_recipient(
        &self,
        index: usize,
        address_str: Option<String>,
        amount: Option<u64>,
    ) -> Result<WasmTxBuilder, js_sys::Error> {
        let inner = self.inner.update_recipient(index, (address_str, amount));

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen(js_name = updateRecipientAmountToMax)]
    pub async fn update_recipient_amount_to_max(&self, index: usize) -> Result<WasmTxBuilder, js_sys::Error> {
        let inner = self.inner.update_recipient_amount_to_max(index).await;

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen(js_name = getRecipients)]
    pub fn get_recipients(&self) -> Vec<WasmRecipient> {
        self.inner
            .recipients
            .clone()
            .into_iter()
            .map(|recipient| {
                let TmpRecipient(uuid, address, amount) = recipient;
                WasmRecipient(uuid, address, amount.to_sat())
            })
            .collect()
    }

    /**
     * UTXOs
     */

    #[wasm_bindgen(js_name = addUtxoToSpend)]
    pub fn add_utxo_to_spend(&self, outpoint: WasmOutPoint) -> Result<WasmTxBuilder, js_sys::Error> {
        let serialised: OutPoint = outpoint.try_into()?;
        let inner = self.inner.add_utxo_to_spend(&serialised);

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen(js_name = removeUtxoToSpend)]
    pub fn remove_utxo_to_spend(&self, outpoint: WasmOutPoint) -> Result<WasmTxBuilder, js_sys::Error> {
        let serialised: OutPoint = outpoint.try_into()?;
        let inner = self.inner.remove_utxo_to_spend(&serialised);

        Ok(WasmTxBuilder { inner })
    }

    #[wasm_bindgen(js_name = clearUtxosToSpend)]
    pub fn clear_utxos_to_spend(&self) -> WasmTxBuilder {
        let inner = self.inner.clear_utxos_to_spend();
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = getUtxosToSpend)]
    pub fn get_utxos_to_spend(&self) -> Vec<WasmOutPoint> {
        self.inner
            .utxos_to_spend
            .clone()
            .into_iter()
            .map(|outpoint| {
                let utxo: WasmOutPoint = outpoint.into();
                utxo
            })
            .collect()
    }

    /**
     * Coin selection enforcement
     */

    #[wasm_bindgen(js_name = setCoinSelection)]
    pub fn set_coin_selection(&self, coin_selection: WasmCoinSelection) -> Self {
        let inner = self.inner.set_coin_selection(coin_selection.into());
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = getCoinSelection)]
    pub fn get_coin_selection(&self) -> WasmCoinSelection {
        self.inner.coin_selection.clone().into()
    }

    /**
     * RBF
     */

    #[wasm_bindgen(js_name = enableRbf)]
    pub fn enable_rbf(&self) -> WasmTxBuilder {
        let inner = self.inner.enable_rbf();
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = disableRbf)]
    pub fn disable_rbf(&self) -> WasmTxBuilder {
        let inner = self.inner.disable_rbf();
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = getRbfEnabled)]
    pub fn get_rbf_enabled(&self) -> bool {
        self.inner.rbf_enabled
    }

    /**
     * Change policy
     */

    #[wasm_bindgen(js_name = setChangePolicy)]
    pub fn set_change_policy(&self, change_policy: WasmChangeSpendPolicy) -> Self {
        let inner = self.inner.set_change_policy(change_policy.into());
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = getChangePolicy)]
    pub fn get_change_policy(&self) -> WasmChangeSpendPolicy {
        self.inner.change_policy.into()
    }

    /**
     * Fees
     */

    #[wasm_bindgen(js_name = setFeeRate)]
    pub fn set_fee_rate(&self, sat_per_vb: u64) -> WasmTxBuilder {
        let inner = self.inner.set_fee_rate(sat_per_vb);
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = getFeeRate)]
    pub fn get_fee_rate(&self) -> Option<u64> {
        if let Some(fee_rate) = self.inner.fee_rate {
            Some(fee_rate.to_sat_per_vb_ceil())
        } else {
            None
        }
    }

    /**
     * Locktime
     */

    #[wasm_bindgen(js_name = addLocktime)]
    pub fn add_locktime(&self, locktime: WasmLockTime) -> Self {
        let inner = self.inner.add_locktime(locktime.into());
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = removeLocktime)]
    pub fn remove_locktime(&self) -> Self {
        let inner = self.inner.remove_locktime();
        WasmTxBuilder { inner }
    }

    #[wasm_bindgen(js_name = getLocktime)]
    pub fn get_locktime(&self) -> Option<WasmLockTime> {
        self.inner.locktime.map(|l| l.into())
    }

    /**
     * Final
     */

    #[wasm_bindgen(js_name = createPsbt)]
    pub async fn create_pbst(&self, network: WasmNetwork) -> Result<WasmPsbt, JsValue> {
        let psbt = self
            .inner
            .create_psbt(false, false, false)
            .await
            .map_err(|e| e.to_js_error())?;

        WasmPsbt::from_psbt(&psbt, network.into())
    }

    #[wasm_bindgen(js_name = createDraftPsbt)]
    pub async fn create_draft_psbt(&self, network: WasmNetwork, allow_dust: Option<bool>) -> Result<WasmPsbt, JsValue> {
        let psbt = self
            .inner
            .create_draft_psbt(allow_dust.unwrap_or(false))
            .await
            .map_err(|e| e.to_js_error())?;

        WasmPsbt::from_psbt(&psbt, network.into())
    }
}
