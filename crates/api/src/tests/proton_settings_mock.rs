#[cfg(feature = "mocking")]
pub mod mock_utils {
    use mockall::mock;

    use crate::{
        error::Error,
        proton_settings::{
            ApiMnemonicUserKey, ProtonSettingsClientExt, SetTwoFaTOTPRequestBody, SetTwoFaTOTPResponseBody,
            UpdateMnemonicSettingsRequestBody,
        },
        proton_users::{ProtonSrpClientProofs, ProtonUserSettings},
    };

    mock! {
        pub ProtonSettingsClient {}

        #[cfg(target_arch = "wasm32")]
        #[async_trait::async_trait(?Send)]
        impl ProtonSettingsClientExt for ProtonSettingsClient {
            async fn get_mnemonic_settings(&self) -> Result<Vec<ApiMnemonicUserKey>, Error>;

            async fn set_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error>;

            async fn reactive_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error>;

            async fn disable_mnemonic_settings(&self, req: ProtonSrpClientProofs) -> Result<String, Error>;

            async fn enable_2fa_totp(&self, req: SetTwoFaTOTPRequestBody) -> Result<SetTwoFaTOTPResponseBody, Error>;

            async fn disable_2fa_totp(&self, req: ProtonSrpClientProofs) -> Result<ProtonUserSettings, Error>;
        }

        #[cfg(not(target_arch = "wasm32"))]
        #[async_trait::async_trait]
        impl ProtonSettingsClientExt for ProtonSettingsClient {
            async fn get_mnemonic_settings(&self) -> Result<Vec<ApiMnemonicUserKey>, Error>;

            async fn set_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error>;

            async fn reactive_mnemonic_settings(&self, req: UpdateMnemonicSettingsRequestBody) -> Result<u32, Error>;

            async fn disable_mnemonic_settings(&self, req: ProtonSrpClientProofs) -> Result<String, Error>;

            async fn enable_2fa_totp(&self, req: SetTwoFaTOTPRequestBody) -> Result<SetTwoFaTOTPResponseBody, Error>;

            async fn disable_2fa_totp(&self, req: ProtonSrpClientProofs) -> Result<ProtonUserSettings, Error>;
        }
    }
}
