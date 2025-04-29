use crate::api::exchange_rate::WasmApiExchangeRate;
use crate::bitcoin::account::WasmAccount;
use crate::common::error::ErrorExt;
use andromeda_features::account_statement_generator::AccountStatementGenerator;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmAccountStatementGenerator {
    inner: AccountStatementGenerator,
}

#[wasm_bindgen]
impl WasmAccountStatementGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(wasm_exchange_rate: Option<WasmApiExchangeRate>) -> Self {
        let exchange_rate = wasm_exchange_rate.map(|r| r.into());

        Self {
            inner: AccountStatementGenerator::new(Vec::new(), Vec::new(), exchange_rate),
        }
    }

    #[wasm_bindgen(js_name = addAccount)]
    pub async fn add_account(&mut self, account: &WasmAccount, name: String) -> Result<(), js_sys::Error> {
        self.inner.add_account(account.get_inner(), name);
        Ok(())
    }

    #[wasm_bindgen(js_name = toPdf)]
    pub async fn to_pdf(&self, export_time: u64) -> Result<Vec<u8>, js_sys::Error> {
        let data = self.inner.to_pdf(export_time).await.map_err(|e| e.to_js_error())?;

        Ok(data)
    }

    #[wasm_bindgen(js_name = toCsv)]
    pub async fn to_csv(&self, export_time: u64) -> Result<Vec<u8>, js_sys::Error> {
        let data = self.inner.to_csv(export_time).await.map_err(|e| e.to_js_error())?;

        Ok(data)
    }
}
