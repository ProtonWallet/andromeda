use andromeda_api::invite::InviteClient;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmInviteClient(InviteClient);

impl From<InviteClient> for WasmInviteClient {
    fn from(value: InviteClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmInviteClient {
    #[wasm_bindgen(js_name = "sendNewcomerInvite")]
    pub async fn send_newcomer_invite(&self, invitee_email: String) -> Result<(), js_sys::Error> {
        self.0
            .send_newcomer_invite(invitee_email)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "checkInviteStatus")]
    pub async fn check_invite_status(&self, invitee_email: String) -> Result<(), js_sys::Error> {
        self.0
            .check_invite_status(invitee_email)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "sendEmailIntegrationInvite")]
    pub async fn send_email_integration_invite(&self, invitee_email: String) -> Result<(), js_sys::Error> {
        self.0
            .send_email_integration_invite(invitee_email)
            .await
            .map_err(|e| e.to_js_error())
    }
}
