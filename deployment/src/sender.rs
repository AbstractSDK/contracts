use secp256k1::{Secp256k1, Signing, Context};
use terra_rust_api::{core_types::Coin, messages::MsgSend, GasOptions, Message, PrivateKey, Terra};

pub struct Sender <C: Signing + Context>{
    pub terra: Terra,
    pub private_key: PrivateKey,
    pub secp: Secp256k1<C>,
}

