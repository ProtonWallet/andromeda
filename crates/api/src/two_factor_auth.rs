use crate::user_settings::UserSettings;
use async_std::sync::RwLock;
use base64::engine::general_purpose;
use base64::Engine as _;
use muon::pgp_verif;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{error::Error, BASE_CORE_API_V4};

#[derive(Clone)]
pub struct TwoFactorAuthClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct GetTwoFactorAuthEnableResponseBody {
    pub Code: u16,
    pub TwoFactorRecoveryCodes: Vec<String>,
    // TODO::add response body here
    pub UserSettings: UserSettings,
}
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct GetTwoFactorAuthDisableResponseBody {
    pub Code: u16,
    // TODO::add response body here
    pub UserSettings: UserSettings,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TwoFactorAuthTOTPDisableRequestBody {
    pub ClientEphemeral: String,
    pub ClientProof: String,
    pub SRPSession: String,
    pub TOTPConfirmation: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TwoFactorAuthTOTPEnableRequestBody {
    pub ClientEphemeral: String,
    pub ClientProof: String,
    pub SRPSession: String,
    pub TOTPConfirmation: String,
    pub TOTPSharedSecret: String,
}

impl TwoFactorAuthClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_2fa_enabled(&self) -> Result<u32, Error> {
        let request = ProtonRequest::new(Method::POST, format!("{}/settings", BASE_CORE_API_V4));

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetTwoFactorAuthDisableResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.UserSettings.TwoFactorAuth.Enabled)
    }

    pub async fn set_2fa_totp(
        &self,
        username: String,
        password: String,
        totp_shared_secret: String,
        totp_confirmation: String,
    ) -> Result<Vec<String>, Error> {
        let auth_info_res = self
            .session
            .read()
            .await
            .get_auth_info(&username)
            .await
            .map_err(|e| e.into())?;

        let modulus_b64 = pgp_verif::verify_and_extract_modulus(&auth_info_res.Modulus).ok_or(Error::HttpError)?;
        let modulus = general_purpose::STANDARD.decode(modulus_b64).unwrap();
        let salt = general_purpose::STANDARD.decode(auth_info_res.Salt).unwrap();
        let server_ephemeral = general_purpose::STANDARD.decode(auth_info_res.ServerEphemeral).unwrap();

        let client = proton_srp::SRPAuth::new(
            modulus.try_into().unwrap(),
            salt.try_into().unwrap(),
            server_ephemeral.try_into().unwrap(),
            &password,
        )
        .map_err(|_| Error::HttpError)?;

        let client_generated_proofs = client.generate_proofs().map_err(|_| Error::HttpError)?;

        let client_challenge = &general_purpose::STANDARD.encode(client_generated_proofs.client_ephemeral);
        let client_proof = &general_purpose::STANDARD.encode(client_generated_proofs.client_proof);
        let srp_session = &auth_info_res.SRPSession;

        let payload = TwoFactorAuthTOTPEnableRequestBody {
            ClientEphemeral: client_challenge.into(),
            ClientProof: client_proof.into(),
            SRPSession: srp_session.into(),
            TOTPConfirmation: totp_confirmation,
            TOTPSharedSecret: totp_shared_secret,
        };

        let request = ProtonRequest::new(Method::POST, format!("{}/settings/2fa/totp", BASE_CORE_API_V4))
            .json_body(payload)
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetTwoFactorAuthEnableResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.TwoFactorRecoveryCodes)
    }

    pub async fn disable_2fa_totp(
        &self,
        username: String,
        password: String,
        totp_confirmation: String,
    ) -> Result<u32, Error> {
        let auth_info_res = self
            .session
            .read()
            .await
            .get_auth_info(&username)
            .await
            .map_err(|e| e.into())?;

        let modulus_b64 = pgp_verif::verify_and_extract_modulus(&auth_info_res.Modulus).ok_or(Error::HttpError)?;
        let modulus = general_purpose::STANDARD.decode(modulus_b64).unwrap();
        let salt = general_purpose::STANDARD.decode(auth_info_res.Salt).unwrap();
        let server_ephemeral = general_purpose::STANDARD.decode(auth_info_res.ServerEphemeral).unwrap();

        let client = proton_srp::SRPAuth::new(
            modulus.try_into().unwrap(),
            salt.try_into().unwrap(),
            server_ephemeral.try_into().unwrap(),
            &password,
        )
        .map_err(|_| Error::HttpError)?;

        let client_generated_proofs = client.generate_proofs().map_err(|_| Error::HttpError)?;

        let client_challenge = &general_purpose::STANDARD.encode(client_generated_proofs.client_ephemeral);
        let client_proof = &general_purpose::STANDARD.encode(client_generated_proofs.client_proof);
        let srp_session = &auth_info_res.SRPSession;

        let payload = TwoFactorAuthTOTPDisableRequestBody {
            ClientEphemeral: client_challenge.into(),
            ClientProof: client_proof.into(),
            SRPSession: srp_session.into(),
            TOTPConfirmation: totp_confirmation,
        };

        let request = ProtonRequest::new(Method::PUT, format!("{}/settings/2fa/totp", BASE_CORE_API_V4))
            .json_body(payload)
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetTwoFactorAuthDisableResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.UserSettings.TwoFactorAuth.Enabled)
    }
}
