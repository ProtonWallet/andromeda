use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    proton_users::{ApiProtonUserSettingsResponse, EmptyResponseBody, ProtonSrpClientProofs, ProtonUserSettings},
    ProtonWalletApiClient, BASE_CORE_API_V4,
};

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ApiMnemonicUserKey {
    pub ID: String,
    pub PrivateKey: String,
    pub Salt: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetMnemonicSettingsResponseBody {
    pub Code: u32,
    pub MnemonicUserKeys: Vec<ApiMnemonicUserKey>,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct UpdateMnemonicSettingsResponseBody {
    pub Code: u32,
    pub ServerProof: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct MnemonicUserKey {
    pub ID: String,
    pub PrivateKey: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct MnemonicAuth {
    pub Version: u32,
    pub ModulusID: String,
    pub Salt: String,
    pub Verifier: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(non_snake_case)]
pub struct UpdateMnemonicSettingsRequestBody {
    pub MnemonicUserKeys: Vec<MnemonicUserKey>,
    pub MnemonicSalt: String,
    pub MnemonicAuth: MnemonicAuth,
}

#[derive(Serialize, Debug)]
#[allow(non_snake_case)]
pub struct SetTwoFaTOTPRequestBody {
    pub TOTPConfirmation: String,
    pub TOTPSharedSecret: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct SetTwoFaTOTPResponseBody {
    pub Code: u32,
    pub TwoFactorRecoveryCodes: Vec<String>,
    pub UserSettings: ProtonUserSettings,
}

#[derive(Clone)]
pub struct ProtonSettingsClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ProtonSettingsClient {
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

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait ProtonSettingsClientExt {
    async fn get_mnemonic_settings(&self) -> Result<Vec<ApiMnemonicUserKey>, Error>;

    async fn set_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error>;

    async fn reactive_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error>;

    async fn disable_mnemonic_settings(&self, req: ProtonSrpClientProofs) -> Result<String, Error>;

    async fn enable_2fa_totp(&self, req: SetTwoFaTOTPRequestBody) -> Result<SetTwoFaTOTPResponseBody, Error>;

    async fn disable_2fa_totp(&self, req: ProtonSrpClientProofs) -> Result<ProtonUserSettings, Error>;
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl ProtonSettingsClientExt for ProtonSettingsClient {
    async fn get_mnemonic_settings(&self) -> Result<Vec<ApiMnemonicUserKey>, Error> {
        let request = self.get("settings/mnemonic");
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetMnemonicSettingsResponseBody>()?;
        Ok(parsed.MnemonicUserKeys)
    }

    async fn set_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error> {
        let request = self.put("settings/mnemonic").body_json(req)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<EmptyResponseBody>()?;
        Ok(parsed.Code)
    }

    async fn reactive_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error> {
        let request = self.put("settings/mnemonic/reactivate").body_json(req)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<EmptyResponseBody>()?;
        Ok(parsed.Code)
    }

    async fn disable_mnemonic_settings(&self, req: ProtonSrpClientProofs) -> Result<String, Error> {
        let request = self.post("settings/mnemonic/disable").body_json(req)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateMnemonicSettingsResponseBody>()?;
        Ok(parsed.ServerProof)
    }

    async fn enable_2fa_totp(&self, req: SetTwoFaTOTPRequestBody) -> Result<SetTwoFaTOTPResponseBody, Error> {
        let request = self.post("settings/2fa/totp").body_json(req)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<SetTwoFaTOTPResponseBody>()?;
        Ok(parsed)
    }

    async fn disable_2fa_totp(&self, req: ProtonSrpClientProofs) -> Result<ProtonUserSettings, Error> {
        let request = self.put("settings/2fa/totp").body_json(req)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ApiProtonUserSettingsResponse>()?;
        Ok(parsed.UserSettings)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::{MnemonicAuth, UpdateMnemonicSettingsRequestBody};
    use crate::{
        core::ApiClient,
        proton_settings::{
            GetMnemonicSettingsResponseBody, MnemonicUserKey, ProtonSettingsClient, ProtonSettingsClientExt,
        },
        proton_users::ProtonSrpClientProofs,
        read_mock_file,
        tests::utils::{common_api_client, setup_test_connection},
        BASE_CORE_API_V4,
    };

    #[tokio::test]
    async fn test_get_mnemonic_settings_response_body_parse() {
        let json_data = r#"
        {
            "Code": 1000,
            "MnemonicUserKeys": [
                {
                    "ID": "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==",
                    "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK ...",
                    "Salt": "1H8EGg3J1Qwk243hf=="
                }
            ]
        }
        "#;

        let response: GetMnemonicSettingsResponseBody = serde_json::from_str(json_data).unwrap();
        assert!(response.Code == 1000);
        assert!(!response.MnemonicUserKeys.is_empty());
        let mnemonic_user_key = response.MnemonicUserKeys.first().unwrap();

        assert_eq!(mnemonic_user_key.ID, "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==");
        assert_eq!(mnemonic_user_key.PrivateKey, "-----BEGIN PGP PRIVATE KEY BLOCK ...");
        assert_eq!(mnemonic_user_key.Salt, "1H8EGg3J1Qwk243hf==");
    }

    #[tokio::test]
    async fn test_update_mnemonic_settings_request_serialize() {
        let json_data = r#"
        {
            "MnemonicUserKeys": [
                {
                    "ID": "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==",
                    "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK ..."
                }
            ],
            "MnemonicSalt": "1H8EGg3J1Qwk243hf==",
            "MnemonicAuth": {
                "Version": 4,
                "ModulusID": "<encrypted_id>",
                "Salt": "<base64_encoded_salt>",
                "Verifier": "<base64_encoded_verifier>"
            }
        }
        "#;
        let value: Value = serde_json::from_str(json_data).unwrap();
        let compact_json_data = serde_json::to_string(&value).unwrap();
        let req = UpdateMnemonicSettingsRequestBody {
            MnemonicUserKeys: vec![MnemonicUserKey {
                ID: "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==".to_string(),
                PrivateKey: "-----BEGIN PGP PRIVATE KEY BLOCK ...".to_string(),
            }],
            MnemonicSalt: "1H8EGg3J1Qwk243hf==".to_string(),
            MnemonicAuth: MnemonicAuth {
                Version: 4,
                ModulusID: "<encrypted_id>".to_string(),
                Salt: "<base64_encoded_salt>".to_string(),
                Verifier: "<base64_encoded_verifier>".to_string(),
            },
        };
        let check_string = serde_json::to_string(&req).unwrap();
        assert_eq!(check_string.len(), compact_json_data.len());
        assert!(check_string.contains("\"<encrypted_id>\","));
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_mnemonic_settings() {
        let api_client = common_api_client().await;
        let client = ProtonSettingsClient::new(api_client);
        let res = client.get_mnemonic_settings().await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_mnemonic_settings_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "MnemonicUserKeys": [
              {
                "ID": "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==",
                "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK ...",
                "Salt": "1H8EGg3J1Qwk243hf=="
              }
            ]
        });
        let req_path: String = format!("{}/settings/mnemonic", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let settings_client = ProtonSettingsClient::new(api_client);
        let userkeys = settings_client.get_mnemonic_settings().await.unwrap();

        assert!(!userkeys.is_empty());
        let mnemonic_user_key = userkeys.first().unwrap();
        assert_eq!(mnemonic_user_key.ID, "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==");
        assert_eq!(mnemonic_user_key.PrivateKey, "-----BEGIN PGP PRIVATE KEY BLOCK ...");
        assert_eq!(mnemonic_user_key.Salt, "1H8EGg3J1Qwk243hf==");
    }

    #[tokio::test]
    async fn test_set_mnemonic_settings_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
        });
        let req_path: String = format!("{}/settings/mnemonic", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "MnemonicUserKeys": [
                    {
                    "ID": "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==",
                    "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK ..."
                    }
                ],
                "MnemonicSalt": "1H8EGg3J1Qwk243hf==",
                "MnemonicAuth": {
                    "Version": 4,
                    "ModulusID": "<encrypted_id>",
                    "Salt": "<base64_encoded_salt>",
                    "Verifier": "<base64_encoded_verifier>"
                },
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let settings_client = ProtonSettingsClient::new(api_client);

        let mnemonic_user_keys = vec![MnemonicUserKey {
            ID: "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==".to_string(),
            PrivateKey: "-----BEGIN PGP PRIVATE KEY BLOCK ...".to_string(),
        }];
        let auth = MnemonicAuth {
            Version: 4,
            ModulusID: "<encrypted_id>".to_string(),
            Salt: "<base64_encoded_salt>".to_string(),
            Verifier: "<base64_encoded_verifier>".to_string(),
        };

        let req = UpdateMnemonicSettingsRequestBody {
            MnemonicUserKeys: mnemonic_user_keys,
            MnemonicSalt: "1H8EGg3J1Qwk243hf==".to_string(),
            MnemonicAuth: auth,
        };
        let code = settings_client.set_mnemonic_settings(req).await.unwrap();
        assert!(code == 1000);
    }

    #[tokio::test]
    async fn test_disable_mnemonic_settings_1000() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "ServerProof": "<base64_encoded_proof>",
        });
        let req_path: String = format!("{}/settings/mnemonic/disable", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
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
        let api_client = setup_test_connection(mock_server.uri());
        let settings_client = ProtonSettingsClient::new(api_client);
        let req = ProtonSrpClientProofs {
            ClientEphemeral: "<base64_encoded_ephemeral>".to_string(),
            ClientProof: "<base64_encoded_proof>".to_string(),
            SRPSession: "<hex_encoded_session_id>".to_string(),
            TwoFactorCode: None,
        };
        let server_proofs = settings_client.disable_mnemonic_settings(req).await.unwrap();
        assert!(server_proofs == *"<base64_encoded_proof>");
    }

    #[tokio::test]
    async fn test_reactive_mnemonic_settings_success() {
        let json_body = serde_json::json!(
        {
            "Code": 1000,
        });
        let req_path: String = format!("{}/settings/mnemonic/reactivate", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        let mock_server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "MnemonicUserKeys": [
                    {
                    "ID": "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==",
                    "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK ..."
                    }
                ],
                "MnemonicSalt": "1H8EGg3J1Qwk243hf==",
                "MnemonicAuth": {
                    "Version": 4,
                    "ModulusID": "<encrypted_id>",
                    "Salt": "<base64_encoded_salt>",
                    "Verifier": "<base64_encoded_verifier>"
                },
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let settings_client = ProtonSettingsClient::new(api_client);

        let mnemonic_user_keys = vec![MnemonicUserKey {
            ID: "1H8EGg3J1QpSDL...k0uhrHx6nnGQ==".to_string(),
            PrivateKey: "-----BEGIN PGP PRIVATE KEY BLOCK ...".to_string(),
        }];
        let auth = MnemonicAuth {
            Version: 4,
            ModulusID: "<encrypted_id>".to_string(),
            Salt: "<base64_encoded_salt>".to_string(),
            Verifier: "<base64_encoded_verifier>".to_string(),
        };

        let req = UpdateMnemonicSettingsRequestBody {
            MnemonicUserKeys: mnemonic_user_keys,
            MnemonicSalt: "1H8EGg3J1Qwk243hf==".to_string(),
            MnemonicAuth: auth,
        };
        let code = settings_client.reactive_mnemonic_settings(req).await.unwrap();
        assert!(code == 1000);
    }

    #[tokio::test]
    async fn test_enable_2fa_totp_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("two_factor_auth_enable_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/settings/2fa/totp", BASE_CORE_API_V4);
        let secret = "JBSWY3DPEHPK3PXP";
        let code = "123456";
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "TOTPConfirmation": code,
                "TOTPSharedSecret": secret,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ProtonSettingsClient::new(api_client);
        let data = crate::proton_settings::SetTwoFaTOTPRequestBody {
            TOTPConfirmation: code.to_string(),
            TOTPSharedSecret: secret.to_string(),
        };
        let result = client.enable_2fa_totp(data).await;
        match result {
            Ok(response) => {
                assert_eq!(response.Code, 1000);
                assert_eq!(response.TwoFactorRecoveryCodes.len(), 8);
                assert_eq!(response.TwoFactorRecoveryCodes[0], "aaaaaaaa");
                assert_eq!(response.TwoFactorRecoveryCodes[1], "bbbbbbbb");
                assert_eq!(response.TwoFactorRecoveryCodes[2], "cccccccc");
                assert_eq!(response.TwoFactorRecoveryCodes[3], "dddddddd");
                assert_eq!(response.TwoFactorRecoveryCodes[4], "eeeeeeee");
                assert_eq!(response.TwoFactorRecoveryCodes[5], "ffffffff");
                assert_eq!(response.TwoFactorRecoveryCodes[6], "gggggggg");
                assert_eq!(response.TwoFactorRecoveryCodes[7], "hhhhhhhh");
                let two_fa_settings = response.UserSettings.two_fa.unwrap();
                assert_eq!(two_fa_settings.Enabled, 1);
                assert_eq!(two_fa_settings.Allowed, 3);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_disable_2fa_totp_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("two_factor_auth_disable_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/settings/2fa/totp", BASE_CORE_API_V4);
        let code = "123456";
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "ClientEphemeral": "<base64_encoded_ephemeral>",
                "ClientProof": "<base64_encoded_proof>",
                "SRPSession": "<hex_encoded_session_id>",
                "TwoFactorCode": code,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ProtonSettingsClient::new(api_client);
        let req = ProtonSrpClientProofs {
            ClientEphemeral: "<base64_encoded_ephemeral>".to_string(),
            ClientProof: "<base64_encoded_proof>".to_string(),
            SRPSession: "<hex_encoded_session_id>".to_string(),
            TwoFactorCode: Some(code.to_string()),
        };
        let result = client.disable_2fa_totp(req).await;
        match result {
            Ok(response) => {
                let two_fa_settings = response.two_fa.unwrap();
                assert_eq!(two_fa_settings.Enabled, 0);
                assert_eq!(two_fa_settings.Allowed, 3);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
