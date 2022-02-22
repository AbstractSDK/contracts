use secp256k1::{Context, Secp256k1, Signing};
use terra_rust_api::{core_types::Coin, messages::MsgSend, GasOptions, Message, PrivateKey, Terra, errors::TerraRustAPIError};

pub struct Sender<C: Signing + Context> {
    pub terra: Terra,
    pub private_key: PrivateKey,
    pub secp: Secp256k1<C>,
}

impl<C: Signing + Context> Sender<C> {
    pub fn pub_addr(&self) -> Result<String, TerraRustAPIError> {
        self.private_key.public_key(&self.secp).account()
    }
}