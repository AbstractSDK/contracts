use secp256k1::{Context, Secp256k1, Signing};
use std::env;
use terra_rust_api::{errors::TerraRustAPIError, GasOptions, PrivateKey, Terra};

pub struct Sender<C: Signing + Context> {
    pub terra: Terra,
    pub private_key: PrivateKey,
    pub secp: Secp256k1<C>,
}

impl<C: Signing + Context> Sender<C> {
    pub fn pub_addr(&self) -> Result<String, TerraRustAPIError> {
        self.private_key.public_key(&self.secp).account()
    }
    pub fn new(config: &GroupConfig, key: PrivateKey, secp: Secp256k1<C>) -> Sender<C> {
        Sender {
            terra: Terra::lcd_client(
                config.network.lcd_url.clone(),
                config.network.chain_id.clone(),
                &config.network.gas_opts,
                None,
            ),
            private_key: key,
            secp,
        }
    }
}

pub enum Network {
    LocalTerra,
    Mainnet,
    Testnet,
}

impl Network {
    async fn config(&self, client: reqwest::Client, denom: &str) -> anyhow::Result<NetworkConfig> {
        let conf = match self {
            Network::LocalTerra => (
                env::var("LTERRA_LCD")?,
                env::var("LTERRA_FCD")?,
                env::var("LTERRA_ID")?,
            ),
            Network::Mainnet => (
                env::var("MAINNET_LCD")?,
                env::var("MAINNET_FCD")?,
                env::var("MAINNET_ID")?,
            ),
            Network::Testnet => (
                env::var("TESTNET_LCD")?,
                env::var("TESTNET_FCD")?,
                env::var("TESTNET_ID")?,
            ),
        };
        let gas_opts = GasOptions::create_with_fcd(&client, &conf.1, denom, 1.3f64).await?;

        Ok(NetworkConfig {
            lcd_url: conf.0,
            fcd_url: conf.1,
            chain_id: conf.2,
            gas_opts,
        })
    }
}
#[derive(Clone, Debug)]
pub struct GroupConfig {
    pub network: NetworkConfig,
    pub name: String,
    pub file_path: String,
}

impl GroupConfig {
    pub async fn new(
        network: Network,
        name: String,
        client: reqwest::Client,
        denom: &str,
        file_path: String,
    ) -> anyhow::Result<GroupConfig> {
        Ok(GroupConfig {
            network: network.config(client, denom).await?,
            name,
            file_path,
        })
    }
}
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub lcd_url: String,
    pub fcd_url: String,
    pub chain_id: String,
    pub gas_opts: GasOptions,
}
