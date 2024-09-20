use std::sync::Arc;

use muon::rest::core::v4::{keys::salts::KeySalt, users::User};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ChildSession {
    pub session_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: Vec<String>,
}
pub struct UserData {
    pub user: User,
    pub key_salts: Vec<KeySalt>,
}

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_CORE_API_V4,
};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonUserSettingsResponse {
    pub Code: u32,
    pub UserSettings: ProtonUserSettings,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProtonUserSettings {
    pub Email: EmailSettings,
    pub Password: Option<PasswordSettings>,
    pub Phone: Option<PhoneSettings>,
    #[serde(rename = "2FA")]
    pub two_fa: Option<TwoFASettings>,
    pub News: u32,
    pub Locale: String,
    pub LogAuth: u32,
    pub InvoiceText: String,
    pub Density: u32,
    pub WeekStart: u32,
    pub DateFormat: u32,
    pub TimeFormat: u32,
    pub Welcome: u32,
    pub WelcomeFlag: u32,
    pub EarlyAccess: u32,
    pub Flags: Option<FlagsSettings>,
    pub Referral: Option<ReferralSettings>,
    pub DeviceRecovery: Option<u32>,
    pub Telemetry: u32,
    pub CrashReports: u32,
    pub HideSidePanel: u32,
    pub HighSecurity: Option<HighSecuritySettings>,
    pub SessionAccountRecovery: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct EmailSettings {
    pub Value: Option<String>,
    pub Status: u32,
    pub Notify: u32,
    pub Reset: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PasswordSettings {
    // PasswordSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PhoneSettings {
    // PhoneSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TwoFASettings {
    pub Enabled: u32,
    pub Allowed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct FlagsSettings {
    // FlagsSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ReferralSettings {
    // ReferralSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HighSecuritySettings {
    pub Eligible: u32,
    pub Value: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonUserResponse {
    Code: u32,
    User: ProtonUser,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProtonUser {
    pub ID: String,
    pub Name: Option<String>,
    pub UsedSpace: u64,
    pub Currency: String,
    pub Credit: u32,
    pub CreateTime: u64,
    pub MaxSpace: u64,
    pub MaxUpload: u64,
    pub Role: u32,
    pub Private: u32,
    pub Subscribed: u32,
    pub Services: u32,
    pub Delinquent: u32,
    pub OrganizationPrivateKey: Option<String>,
    pub Email: String,
    pub DisplayName: Option<String>,
    pub Keys: Option<Vec<ProtonUserKey>>,
    pub MnemonicStatus: u32,
}
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProtonUserKey {
    pub ID: String,
    pub Version: u32,
    pub PrivateKey: String,
    pub RecoverySecret: Option<String>,
    pub RecoverySecretSignature: Option<String>,
    pub Token: Option<String>,
    pub Fingerprint: String,
    pub Primary: u32,
    pub Active: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiUserInfo {
    pub ID: String,
    pub Name: String,
    pub Email: String,
    pub CanonicalEmail: String,
    pub IsProton: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetAuthModulusResponse {
    pub Code: u32,
    pub Modulus: String,
    pub ModulusID: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct GetAuthInfoRequest {
    pub Intent: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct TwoFA {
    #[allow(non_snake_case)]
    pub Enabled: u8,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetAuthInfoResponseBody {
    pub Code: u32,
    pub Modulus: String,
    pub ServerEphemeral: String,
    pub Version: u8,
    pub Salt: String,
    pub SRPSession: String,
    #[serde(rename = "2FA")]
    pub two_fa: TwoFA,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ProtonSrpClientProofs {
    pub ClientEphemeral: String,
    pub ClientProof: String,
    pub SRPSession: String,
    pub TwoFactorCode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ProtonSrpServerProofs {
    pub Code: u32,
    pub ServerProof: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct EmptyResponseBody {
    pub Code: u32,
}

#[derive(Clone)]
pub struct ProtonUsersClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ProtonUsersClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_CORE_API_V4
    }
}

impl ProtonUsersClient {
    pub async fn get_auth_modulus(&self) -> Result<GetAuthModulusResponse, Error> {
        let request = self.get("auth/modulus");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetAuthModulusResponse>()?;
        Ok(parsed)
    }

    // this is spical endpoint. it is get data but with a post call
    pub async fn get_auth_info(&self, req: GetAuthInfoRequest) -> Result<GetAuthInfoResponseBody, Error> {
        let request: muon::ProtonRequest = self.post("auth/info").body_json(req)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetAuthInfoResponseBody>()?;
        Ok(parsed)
    }

    pub async fn unlock_password_change(&self, proofs: ProtonSrpClientProofs) -> Result<String, Error> {
        let request = self.put("users/password").body_json(proofs)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ProtonSrpServerProofs>()?;
        Ok(parsed.ServerProof)
    }

    pub async fn unlock_sensitive_settings(&self, proofs: ProtonSrpClientProofs) -> Result<String, Error> {
        let request = self.put("users/unlock").body_json(proofs)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ProtonSrpServerProofs>()?;
        Ok(parsed.ServerProof)
    }

    pub async fn lock_sensitive_settings(&self) -> Result<u32, Error> {
        let request = self.put("users/lock");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<EmptyResponseBody>()?;
        Ok(parsed.Code)
    }

    // get proton user info. This includes the user's keys.
    pub async fn get_user_info(&self) -> Result<ProtonUser, Error> {
        let request = self.get("users");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ApiProtonUserResponse>()?;
        Ok(parsed.User)
    }

    // get proton user settings.
    //  used for 2fa settings and password recovery etc..
    pub async fn get_user_settings(&self) -> Result<ProtonUserSettings, Error> {
        let request = self.get("settings");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ApiProtonUserSettingsResponse>()?;
        Ok(parsed.UserSettings)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::ProtonUsersClient;
    use crate::{
        core::ApiClient,
        proton_users::ProtonSrpClientProofs,
        tests::utils::{common_api_client, setup_test_connection_arc},
        BASE_CORE_API_V4,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_user_info() {
        let api_client = common_api_client().await;
        let client = ProtonUsersClient::new(api_client);
        let res = client.get_user_info().await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }
    #[tokio::test]
    #[ignore]
    async fn should_get_user_settings() {
        let api_client = common_api_client().await;
        let client = ProtonUsersClient::new(api_client);
        let res = client.get_user_settings().await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_info_code_1000() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "User": {
                    "ID": "MJLke8kWh1BBvG95JBIrZvzpgsZ94hNNgjNHVyhXMiv4g9cn6SgvqiIFR5cigpml2LD_iUk_3DkV29oojTt3eA==",
                    "Name": "abc",
                    "UsedSpace": 96691332,
                    "Currency": "USD",
                    "Credit": 0,
                    "CreateTime": 1654615960,
                    "MaxSpace": 10737418240i64,
                    "MaxUpload": 26214400i64,
                    "Role": 2,
                    "Private": 1,
                    "Subscribed": 1,
                    "Services": 1,
                    "Delinquent": 0,
                    "OrganizationPrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK-----*",
                    "Email": "abc@protonmail.ch",
                    "DisplayName": "abc",
                    "Keys": [
                        {
                            "ID": "IlnTbqicN-2HfUGIn-ki8bqZfLqNj5ErUB0z24Qx5g-4NvrrIc6GLvEpj2EPfwGDv28aKYVRRrSgEFhR_zhlkA==",
                            "Version": 3,
                            "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK-----*-----END PGP PRIVATE KEY BLOCK-----", //#gitleaks:allow

                            "Token": "-----BEGIN PGP MESSAGE-----.*-----END PGP MESSAGE-----",
                            "Fingerprint": "c93f767df53b0ca8395cfde90483475164ec6353",
                            "Primary": 1,
                            "Active": 1,
                        }
                    ],
                    "MnemonicStatus": 4,
                },
                "Code": 1000
            }
        );
        let req_path: String = format!("{}/users", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_info = users_client.get_user_info().await;
        match user_info {
            Ok(value) => {
                assert!(value.DisplayName.unwrap() == "abc");
                assert!(value.Name.unwrap() == "abc");
                assert!(!value.Keys.unwrap().is_empty());
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Expected Ok variant but got Err")
            }
        }
    }

    #[tokio::test]
    async fn test_get_user_info_deserialize_error() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({});
        let req_path: String = format!("{}/users", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(400).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_info = users_client.get_user_info().await;
        assert!(user_info.is_err());
    }

    #[tokio::test]
    async fn test_get_users_settings_code_1000() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "UserSettings": {
                  "Email": {
                    "Value": "abc@gmail.com",
                    "Status": 0,
                    "Notify": 1,
                    "Reset": 0
                  },
                  "Password": {
                    "Mode": 2,
                    "ExpirationTime": null
                  },
                  "Phone": {
                    "Value": "+18005555555",
                    "Status": 0,
                    "Notify": 0,
                    "Reset": 0
                  },
                  "2FA": {
                    "Enabled": 3,
                    "Allowed": 3,
                    "ExpirationTime": null,
                    "U2FKeys": [
                      {
                        "Label": "A name",
                        "KeyHandle": "aKeyHandle",
                        "Compromised": 0
                      }
                    ],
                    "RegisteredKeys": [
                      {
                        "AttestationFormat": "fido2-u2f",
                        "CredentialID": [
                          null
                        ],
                        "Name": "My security key"
                      }
                    ]
                  },
                  "News": 244,
                  "Locale": "en_US",
                  "LogAuth": 2,
                  "InvoiceText": "रिवार में हुआ। ज檷\\n Cartoon Law Services\\n 1 DisneyWorld Lane\\n Orlando, FL, 12345\\n VAT",
                  "Density": 0,
                  "Theme": {},
                  "ThemeType": 1,
                  "WeekStart": 1,
                  "DateFormat": 1,
                  "TimeFormat": 1,
                  "Welcome": 1,
                  "WelcomeFlag": 1,
                  "EarlyAccess": 1,
                  "Flags": {
                    "Welcomed": 0,
                    "InAppPromosHidden": 0
                  },
                  "Referral": {
                    "Link": "https://pr.tn/ref/ERBYvlX8SC4KOyb",
                    "Eligible": true
                  },
                  "DeviceRecovery": 1,
                  "Telemetry": 1,
                  "CrashReports": 1,
                  "HideSidePanel": 1,
                  "HighSecurity": {
                    "Eligible": 1,
                    "Value": 1
                  },
                  "SessionAccountRecovery": 1,
                  "MnemonicStatus": 4,
                }
              }
        );
        let req_path: String = format!("{}/settings", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_settings = users_client.get_user_settings().await;
        println!("user_settings {:?}", user_settings);
        match user_settings {
            Ok(value) => {
                assert!(value.HideSidePanel == 1, "Expected hide_side_panel to be 1.");
                assert!(
                    value.DeviceRecovery.unwrap_or(0) == 1,
                    "Expected device_recovery to be 1."
                );
                assert!(value.Locale == "en_US", "Expected locale to be en_US.");
                assert!(value.News == 244, "Expected news to be 244.");
                assert!(value.Phone.is_some(), "Expected phone to be Some.");
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Expected Ok variant but got Err")
            }
        }
    }

    #[tokio::test]
    async fn test_get_users_settings_deserialize_error() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({});
        let req_path: String = format!("{}/settings", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(400).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_settings = users_client.get_user_settings().await;
        assert!(user_settings.is_err());
    }

    #[tokio::test]
    async fn test_get_auth_modulus_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "Modulus": "-----BEGIN PGP SIGNED MESSAGE-----.*-----END PGP SIGNATURE-----",
            "ModulusID": "Oq_JB_IkrOx5WlpxzlRPocN3_NhJ80V7DGav77eRtSDkOtLxW2jfI3nUpEqANGpboOyN-GuzEFXadlpxgVp7_g=="
        });
        let req_path: String = format!("{}/auth/modulus", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let modulus = users_client.get_auth_modulus().await.unwrap();

        assert!(modulus.Code == 1000);
        assert!(modulus.Modulus == "-----BEGIN PGP SIGNED MESSAGE-----.*-----END PGP SIGNATURE-----");
        assert!(
            modulus.ModulusID
                == "Oq_JB_IkrOx5WlpxzlRPocN3_NhJ80V7DGav77eRtSDkOtLxW2jfI3nUpEqANGpboOyN-GuzEFXadlpxgVp7_g=="
        );
    }

    #[tokio::test]
    async fn test_get_auth_info_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "Modulus": "-----BEGIN PGP SIGNED MESSAGE-----*-----END SIGNATURE-----",
            "ServerEphemeral": "<base64_encoded_server_ephemeral>",
            "Version": 4,
            "Salt": "<base64_encoded_salt>",
            "SRPSession": "<hex_encoded_session_key>",
            "2FA": {
                "Enabled": 3,
                "FIDO2": {
                    "AuthenticationOptions": { },
                    "RegisteredKeys": []
                }
            }
        });
        let req_path: String = format!("{}/auth/info", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "Intent": "Proton"
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);

        let req = super::GetAuthInfoRequest {
            Intent: "Proton".to_string(),
        };
        let auth_info = users_client.get_auth_info(req).await.unwrap();
        assert!(auth_info.Code == 1000);
        assert!(auth_info.Modulus == "-----BEGIN PGP SIGNED MESSAGE-----*-----END SIGNATURE-----");
        assert!(auth_info.ServerEphemeral == "<base64_encoded_server_ephemeral>");
        assert!(auth_info.Version == 4);
        assert!(auth_info.Salt == "<base64_encoded_salt>");
        assert!(auth_info.SRPSession == "<hex_encoded_session_key>");
    }

    #[tokio::test]
    async fn test_unlock_password_change_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "ServerProof": "<base64_encoded_proof>"
        });
        let req_path: String = format!("{}/users/password", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "ClientEphemeral": "<base64_encoded_ephemeral>",
                "ClientProof": "<base64_encoded_proof>",
                "SRPSession": "<hex_encoded_session_id>",
                "TwoFactorCode": null,
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);

        let proofs = ProtonSrpClientProofs {
            ClientEphemeral: "<base64_encoded_ephemeral>".to_string(),
            ClientProof: "<base64_encoded_proof>".to_string(),
            SRPSession: "<hex_encoded_session_id>".to_string(),
            TwoFactorCode: None,
        };
        let server_proofs = users_client.unlock_password_change(proofs).await.unwrap();
        assert!(server_proofs == "<base64_encoded_proof>");
    }

    #[tokio::test]
    async fn test_unlock_sensitive_settings_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "ServerProof": "<base64_encoded_proof>"
        });
        let req_path: String = format!("{}/users/unlock", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "ClientEphemeral": "<base64_encoded_ephemeral>",
                "ClientProof": "<base64_encoded_proof>",
                "SRPSession": "<hex_encoded_session_id>",
                "TwoFactorCode": null,
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);

        let proofs = ProtonSrpClientProofs {
            ClientEphemeral: "<base64_encoded_ephemeral>".to_string(),
            ClientProof: "<base64_encoded_proof>".to_string(),
            SRPSession: "<hex_encoded_session_id>".to_string(),
            TwoFactorCode: None,
        };
        let server_proofs = users_client.unlock_sensitive_settings(proofs).await.unwrap();
        assert!(server_proofs == "<base64_encoded_proof>");
    }

    #[tokio::test]
    async fn test_lock_sensitive_settings_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
        });
        let req_path: String = format!("{}/users/lock", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(req_path))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let code = users_client.lock_sensitive_settings().await.unwrap();
        assert!(code == 1000);
    }
}
