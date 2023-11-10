use std::sync::Arc;

use bdk::{
    bitcoin::{
        bip32::{ChildNumber, ExtendedPrivKey},
        secp256k1::Secp256k1,
    },
    keys::DescriptorSecretKey,
    miniscript::{
        descriptor::{DescriptorXKey, Wildcard},
        DescriptorPublicKey as BdkDescriptorPublicKey,
    },
    wallet::Balance,
    KeychainKind,
};

use bdk::bitcoin::Network;
use bdk::Wallet;

use crate::error::Error;
use crate::keys::Deriveable;
use crate::{descriptor::SupportedBIPs, wallet::sync};

pub struct Account {
    descriptor_secret_key: DescriptorSecretKey,
    config: AccountConfig,
}

#[derive(Clone)]
pub struct AccountConfig {
    pub bip: SupportedBIPs,
    pub network: Network,
    pub account_index: u32,
}

impl Account {
    pub fn new(master_secret_key: ExtendedPrivKey, config: AccountConfig) -> Result<Self, Error> {
        let cloned_config = config.clone();

        let derivation_path =
            Self::gen_account_derivation_path(cloned_config.bip, cloned_config.network, cloned_config.account_index)?;

        let descriptor_secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
            origin: None,
            xkey: master_secret_key,
            derivation_path: derivation_path.clone().into(),
            wildcard: Wildcard::Unhardened,
        });

        Ok(Self {
            descriptor_secret_key,
            config,
        })
    }

    fn wallet(self) -> Wallet {
        let deriveable_descriptor = DescriptorSecretKey::from(self.descriptor_secret_key);

        let external_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::External as u32).unwrap()];
        let external_descriptor =
            Arc::into_inner(deriveable_descriptor.derive(external_derivation.into()).unwrap()).unwrap();

        let internal_descriptor = Arc::into_inner(
            deriveable_descriptor
                .derive(vec![ChildNumber::from_normal_idx(KeychainKind::Internal as u32).unwrap()].into())
                .unwrap(),
        )
        .unwrap();

        Wallet::new_no_persist(
            &external_descriptor.to_string(),
            Some(&internal_descriptor.to_string()),
            self.config.network,
        )
        .unwrap()
    }

    fn gen_account_derivation_path(
        bip: SupportedBIPs,
        network: Network,
        account_index: u32,
    ) -> Result<Vec<ChildNumber>, Error> {
        let mut derivation_path = Vec::with_capacity(4);

        // purpose' derivation
        derivation_path.push(
            ChildNumber::from_normal_idx(match bip {
                SupportedBIPs::Bip49 => 49,
                SupportedBIPs::Bip84 => 84,
                SupportedBIPs::Bip86 => 86,
                _ => 44,
            })
            .unwrap(),
        );

        //  coin_type' derivation
        derivation_path.push(
            ChildNumber::from_normal_idx(match network {
                Network::Bitcoin => 0,
                _ => 1,
            })
            .unwrap(),
        );

        // account' derivation
        derivation_path.push(ChildNumber::from_hardened_idx(account_index).map_err(|_| Error::InvalidAccountIndex)?);

        Ok(derivation_path)
    }

    pub fn public_descriptor(self) -> BdkDescriptorPublicKey {
        let secp = Secp256k1::new();
        self.descriptor_secret_key.to_public(&secp).unwrap()
    }

    pub async fn get_balance(self) -> Result<Balance, Error> {
        let wallet = self.wallet();

        let updated_wallet = sync(wallet).await.map_err(|_| Error::SyncError)?;

        Ok(updated_wallet.get_balance())
    }
}
