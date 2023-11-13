use bdk::{
    bitcoin::{
        bip32::{ChildNumber, ExtendedPrivKey},
        secp256k1::Secp256k1,
    },
    descriptor,
    miniscript::DescriptorPublicKey as BdkDescriptorPublicKey,
    wallet::Balance,
    KeychainKind,
};

use bdk::bitcoin::Network;
use bdk::Wallet;
use miniscript::{bitcoin::bip32::DerivationPath, Descriptor};

use crate::error::Error;
use crate::{descriptor::SupportedBIPs, wallet::sync};

#[derive(Clone, Debug)]
pub struct Account {
    account_xprv: ExtendedPrivKey,
    derivation_path: DerivationPath,
    config: AccountConfig,
}

#[derive(Clone, Debug)]
pub struct AccountConfig {
    pub bip: SupportedBIPs,
    pub network: Network,
    pub account_index: u32,
}

impl Account {
    pub fn new(master_secret_key: ExtendedPrivKey, config: AccountConfig) -> Result<Self, Error> {
        let secp = Secp256k1::new();
        let cloned_config = config.clone();

        let derivation_path =
            Self::gen_account_derivation_path(cloned_config.bip, cloned_config.network, cloned_config.account_index)?;

        Ok(Self {
            account_xprv: master_secret_key.derive_priv(&secp, &derivation_path).unwrap(),
            derivation_path: derivation_path.into(),
            config,
        })
    }

    fn wallet(self) -> Wallet {
        let external_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::External as u32).unwrap()];
        let internal_derivation = vec![ChildNumber::from_normal_idx(KeychainKind::Internal as u32).unwrap()];

        let external_descriptor = descriptor!(wpkh((self.account_xprv, external_derivation.into()))).unwrap();
        let internal_descriptor = descriptor!(wpkh((self.account_xprv, internal_derivation.into()))).unwrap();

        Wallet::new_no_persist(external_descriptor, Some(internal_descriptor), self.config.network)
            .map_err(|e| println!("error {}", e))
            .unwrap()
    }

    pub fn derivation_path(&self) -> DerivationPath {
        self.derivation_path.clone()
    }

    fn gen_account_derivation_path(
        bip: SupportedBIPs,
        network: Network,
        account_index: u32,
    ) -> Result<Vec<ChildNumber>, Error> {
        let mut derivation_path = Vec::with_capacity(4);

        // purpose' derivation
        derivation_path.push(
            ChildNumber::from_hardened_idx(match bip {
                SupportedBIPs::Bip49 => 49,
                SupportedBIPs::Bip84 => 84,
                SupportedBIPs::Bip86 => 86,
                _ => 44,
            })
            .unwrap(),
        );

        //  coin_type' derivation
        derivation_path.push(
            ChildNumber::from_hardened_idx(match network {
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
        let (descriptor, _, _) = descriptor!(wpkh((self.account_xprv, Vec::new().into()))).unwrap();
        match descriptor {
            Descriptor::Wpkh(pk) => pk.into_inner(),
            _ => unreachable!(),
        }
    }

    pub async fn get_balance(self) -> Result<Balance, Error> {
        let wallet = self.wallet();

        let updated_wallet = sync(wallet).await.map_err(|_| Error::SyncError)?;

        Ok(updated_wallet.get_balance())
    }
}
