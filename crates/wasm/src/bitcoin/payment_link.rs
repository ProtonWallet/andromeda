use andromeda_bitcoin::payment_link::PaymentLink;
use wasm_bindgen::prelude::*;

use crate::common::{error::DetailledWasmError, types::WasmNetwork};

#[wasm_bindgen]
pub enum WasmPaymentLinkKind {
    BitcoinAddress,
    BitcoinURI,
    LightningURI,
    UnifiedURI,
}

#[wasm_bindgen]
pub struct WasmPaymentLink {
    inner: PaymentLink,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default)]
pub struct WasmOnchainPaymentLink {
    pub address: Option<String>,
    pub amount: Option<u64>,
    pub message: Option<String>,
    pub label: Option<String>,
}

impl Into<WasmPaymentLink> for PaymentLink {
    fn into(self) -> WasmPaymentLink {
        WasmPaymentLink { inner: self }
    }
}

impl Into<PaymentLink> for WasmPaymentLink {
    fn into(self) -> PaymentLink {
        self.inner
    }
}

#[wasm_bindgen]
impl WasmPaymentLink {
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name = toUri)]
    pub fn to_uri(&self) -> String {
        self.inner.to_uri()
    }

    #[wasm_bindgen(js_name = tryParse)]
    pub fn try_parse(str: String, network: WasmNetwork) -> Result<WasmPaymentLink, DetailledWasmError> {
        let inner = PaymentLink::try_parse(str, network.into()).map_err(|e| e.into())?;

        Ok(WasmPaymentLink { inner })
    }

    #[wasm_bindgen(js_name = getKind)]
    pub fn get_kind(&self) -> WasmPaymentLinkKind {
        match self.inner {
            PaymentLink::BitcoinAddress(_) => WasmPaymentLinkKind::BitcoinAddress,
            PaymentLink::BitcoinURI { .. } => WasmPaymentLinkKind::BitcoinURI,
            PaymentLink::LightningURI { .. } => WasmPaymentLinkKind::LightningURI,
            PaymentLink::UnifiedURI { .. } => WasmPaymentLinkKind::UnifiedURI,
        }
    }

    #[wasm_bindgen(js_name = assumeOnchain)]
    pub fn assume_onchain(&self) -> WasmOnchainPaymentLink {
        match self.inner.clone() {
            PaymentLink::BitcoinAddress(address) => WasmOnchainPaymentLink {
                address: Some(address.to_string()),
                ..WasmOnchainPaymentLink::default()
            },
            PaymentLink::BitcoinURI {
                address,
                amount,
                label,
                message,
            } => WasmOnchainPaymentLink {
                address: Some(address.to_string()),
                amount,
                label,
                message,
            },
            PaymentLink::LightningURI { .. } => WasmOnchainPaymentLink::default(),
            PaymentLink::UnifiedURI { .. } => WasmOnchainPaymentLink::default(),
        }
    }
}
