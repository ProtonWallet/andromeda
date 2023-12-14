use crate::error::Error;

use bdk_esplora::esplora_client::AsyncClient as AsyncEsploraClient;

use bdk_esplora::esplora_client;

pub struct Client(AsyncEsploraClient);

impl Client {
    pub fn new(url: Option<String>) -> Result<Self, Error> {
        let url = match url {
            Some(url) => url.clone(),
            _ => "https://mempool.space/testnet/api".to_string(),
        };

        let client = esplora_client::Builder::new(&url)
            .build_async()
            .map_err(|_| Error::Generic {
                msg: "Could not create client".to_string(),
            })?;

        Ok(Client(client))
    }

    pub fn inner(&self) -> AsyncEsploraClient {
        self.0.clone()
    }
}
