use cw_orch::WasmPath;
use cw_orch::Contract;
use cw_orch::CwEnv;

use cosmwasm_std::Empty;
use cw_orch::contract;


#[contract(abstract_etf::msg::InstantiateMsg, abstract_etf::msg::ExecuteMsg, abstract_etf::msg::QueryMsg, Empty)]
pub struct AbstractETF;

impl<Chain: CwEnv> ::cw_orch::Uploadable for AbstractETF<Chain> {
    fn wasm(&self) -> cw_orch::WasmPath {
    	WasmPath::new("artifacts/abstract_etf.wasm").unwrap()
    }
}

impl<Chain: CwEnv> AbstractETF<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}


#[contract(cw20_base::msg::InstantiateMsg,cw20_base::msg::ExecuteMsg, cw20_base::msg::QueryMsg, Empty)]
pub struct Cw20Base;

impl<Chain: CwEnv> ::cw_orch::Uploadable for Cw20Base<Chain> {
    fn wasm(&self) -> cw_orch::WasmPath {
    	WasmPath::new("artifacts/cw20_base.wasm").unwrap()
    }
}

impl<Chain: CwEnv> Cw20Base<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(Contract::new(name, chain))
    }
}