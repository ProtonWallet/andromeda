use std::str::FromStr;

use crate::{account::Account, account_trait::AccessWallet, bdk_wallet_secp_ext::BdkWalletSecpExt, error::Error};
use andromeda_crypto::{
    message::BitcoinMessage,
    message_signature::{MessageSignature, SigningType},
};
use bitcoin::Address;

#[derive(Clone, Copy)]
pub struct MessageSigner {}

impl MessageSigner {
    /// Sign a message with the private key of the address
    ///
    /// Implements the Bitcoin message signing standard (BIP137) https://github.com/bitcoin/bips/blob/master/bip-0137.mediawiki
    /// # Arguments
    /// * `account` - Account to sign the message
    /// * `message` - Message to sign
    /// * `signing_type` - Type of signing
    /// * `btc_address` - Bitcoin address to sign the message
    /// # Returns
    /// * `Result<String>` - Base64 encoded signature
    ///
    pub async fn sign_message(
        &self,
        account: &Account,
        message: &str,
        signing_type: SigningType,
        btc_address: &str,
    ) -> Result<String, Error> {
        let wallet = account.get_wallet().await;
        let script_type = account.get_script_type()?;
        let secp = wallet.secp_ctx();

        let (derived_keypair, is_compressed) = wallet.get_secp256k1_keypair(btc_address)?;
        let signature = BitcoinMessage::from_str(message)?.sign(secp, &derived_keypair)?;
        let message_signature = MessageSignature::new(signature, is_compressed);
        Ok(message_signature.to_base64(signing_type, script_type)?)
    }

    /// Verify a message with the public key of the BTC address
    /// # Arguments
    /// * `account` - Account to verify the message
    /// * `message` - Message to verify
    /// * `signature` - Signature of the message
    /// * `btc_address` - Bitcoin address to verify the message
    /// # Returns
    /// * `Result<(), Error>`
    pub async fn verify_message(
        &self,
        account: &Account,
        message: &str,
        signature: &str,
        btc_address: &str,
    ) -> Result<(), Error> {
        let wallet = account.get_wallet().await;
        let secp_ctx = wallet.secp_ctx();
        let signature = MessageSignature::from_base64(signature)?;
        let address = Address::from_str(btc_address)?.require_network(wallet.network())?;

        Ok(signature.verify(secp_ctx, message, address)?)
    }
}

#[cfg(test)]
mod tests {

    use andromeda_common::{Network, ScriptType};
    use andromeda_crypto::message_signature::SigningType;
    use bitcoin::NetworkKind;
    use tokio_test::{assert_err, assert_ok};

    use crate::{account::Account, message_signer::MessageSigner, tests::utils::tests::set_test_wallet_account};

    #[tokio::test]
    async fn test_signer_with_legacy_account() {
        let script_type = ScriptType::Legacy;
        let account = set_test_account_for_mainnet(script_type, "m/44'/0'/0'");
        let address_str = "1EkwzGz7ojumWC41GXC3gdbb4DD6RxsfVa".to_string();
        let message = "Hello world".to_string();

        let signer = MessageSigner {};
        let result = signer
            .sign_message(&account, &message, SigningType::Electrum, &address_str)
            .await
            .unwrap();

        assert_eq!(
            result,
            "H9zbwo6w9gnsoELFuTVdjDmVUWEk8oyX8stwTEYChrmuRG6QA18wUBgZmznOH4KeY1DfKCkndYFRCsn6kziWsIs="
        );

        assert_ok!(
            signer
                .verify_message(&account, message.as_str(), result.as_str(), address_str.as_str(),)
                .await
        );

        let result = signer
            .sign_message(&account, &message, SigningType::Bip137, &address_str)
            .await
            .unwrap();

        assert_eq!(
            result,
            "H9zbwo6w9gnsoELFuTVdjDmVUWEk8oyX8stwTEYChrmuRG6QA18wUBgZmznOH4KeY1DfKCkndYFRCsn6kziWsIs="
        );

        assert_ok!(
            signer
                .verify_message(&account, message.as_str(), result.as_str(), address_str.as_str(),)
                .await
        );
    }

    #[tokio::test]
    async fn test_signer_with_nested_segwit_account() {
        let script_type = ScriptType::NestedSegwit;
        let account = set_test_account_for_mainnet(script_type, "m/49'/0'/0'");
        let address_str = "3LovK8cXLbRYFZXLdPmnCmHsRk3YFeC5ny".to_string();
        let message = "Hello world".to_string();

        let signer = MessageSigner {};
        let result = signer
            .sign_message(&account, &message, SigningType::Electrum, &address_str)
            .await
            .unwrap();
        println!("result: {}", result);
        assert_eq!(
            result,
            "H6ir3uueMk2nza/ZZitIcrI40n79rC/cmu8oQTsqr+DrG/s4q9X7f1ptvZiGnPbT1Vnw3YogpzRkr6hvyAdl4JU="
        );

        assert_ok!(
            signer
                .verify_message(&account, message.as_str(), result.as_str(), address_str.as_str(),)
                .await
        );

        let result = signer
            .sign_message(&account, &message, SigningType::Bip137, &address_str)
            .await
            .unwrap();

        assert_eq!(
            result,
            "I6ir3uueMk2nza/ZZitIcrI40n79rC/cmu8oQTsqr+DrG/s4q9X7f1ptvZiGnPbT1Vnw3YogpzRkr6hvyAdl4JU="
        );

        assert_ok!(
            signer
                .verify_message(&account, message.as_str(), result.as_str(), address_str.as_str(),)
                .await
        );
    }

    #[tokio::test]
    async fn test_signer_with_native_segwit_account() {
        let script_type = ScriptType::NativeSegwit;
        let account = set_test_account_for_mainnet(script_type, "m/84'/0'/0'");
        let address_str = "bc1q63wfn3mxm4jegwle9v4ll4hh7ypsyg786s7gl6".to_string();
        let message = "Hello world".to_string();

        let signer = MessageSigner {};
        let result = signer
            .sign_message(&account, &message, SigningType::Electrum, &address_str)
            .await
            .unwrap();

        assert_eq!(
            result,
            "IAJllD4KbxxpAx2oXj67d+fNHEcyk+45Pp09HoejR5N2DJMe1Mf7CU4bE91vgLIKBbt0+QQr4F/gNqL4UyYquJg="
        );

        assert_ok!(
            signer
                .verify_message(&account, message.as_str(), result.as_str(), address_str.as_str(),)
                .await
        );

        let result = signer
            .sign_message(&account, &message, SigningType::Bip137, &address_str)
            .await
            .unwrap();

        assert_eq!(
            result,
            "KAJllD4KbxxpAx2oXj67d+fNHEcyk+45Pp09HoejR5N2DJMe1Mf7CU4bE91vgLIKBbt0+QQr4F/gNqL4UyYquJg="
        );

        assert_ok!(
            signer
                .verify_message(&account, message.as_str(), result.as_str(), address_str.as_str(),)
                .await
        );
    }

    #[tokio::test]
    async fn test_signer_with_wrong_btc_address() {
        let script_type = ScriptType::NativeSegwit;
        let account = set_test_account_for_mainnet(script_type, "m/84'/0'/0'");
        let address_str = "3LovK8cXLbRYFZXLdPmnCmHsRk3YFeC5ny".to_string();
        let message = "Hello world".to_string();

        let signer = MessageSigner {};
        let result = signer
            .sign_message(&account, &message, SigningType::Electrum, &address_str)
            .await;

        assert_err!(&result);
        if let Err(e) = result {
            assert_eq!(
                e.to_string(),
                "Address is invalid: 3LovK8cXLbRYFZXLdPmnCmHsRk3YFeC5ny".to_string()
            );
        }
    }

    #[tokio::test]
    async fn test_signer_verify() {
        let script_type = ScriptType::NativeSegwit;
        let account = set_test_account_for_mainnet(script_type, "m/84'/0'/0'");
        let address_str = "bc1q63wfn3mxm4jegwle9v4ll4hh7ypsyg786s7gl6".to_string();
        let message = "Hello world".to_string();

        let signer = MessageSigner {};

        assert_ok!(
            signer
                .verify_message(
                    &account,
                    &message,
                    "IAJllD4KbxxpAx2oXj67d+fNHEcyk+45Pp09HoejR5N2DJMe1Mf7CU4bE91vgLIKBbt0+QQr4F/gNqL4UyYquJg=",
                    &address_str,
                )
                .await
        );
    }

    #[tokio::test]
    async fn test_signer_verify_with_invalid_signature() {
        let script_type = ScriptType::NativeSegwit;
        let account = set_test_account_for_mainnet(script_type, "m/84'/0'/0'");
        let address_str = "bc1q63wfn3mxm4jegwle9v4ll4hh7ypsyg786s7gl6".to_string();
        let message = "Hello world".to_string();

        let signer = MessageSigner {};

        let result = signer
            .verify_message(
                &account,
                &message,
                "JllD4KbxxpAx2oXj67d+fNHEcyk+45Pp09HoejR5N2DJMe1Mf7CU4bE91vgLIKBbt0+QQr4F/gNqL4UyYquJg=",
                &address_str,
            )
            .await;

        assert_err!(result);
    }

    fn set_test_account_for_mainnet(script_type: ScriptType, derivation_path: &str) -> Account {
        set_test_wallet_account(
            "they velvet shoot decide timber stadium inch volcano iron ten eye priority",
            script_type,
            derivation_path,
            None,
            None,
            Some(Network::Bitcoin),
            Some(NetworkKind::Main),
        )
    }
}
