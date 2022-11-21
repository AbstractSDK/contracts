use abstract_sdk::os::extension;
use abstract_sdk::os::tendermint_staking::{QueryMsg, RequestMsg};
use boot_core::prelude::boot_contract;
use boot_core::BootEnvironment;
use cosmwasm_std::Empty;

use boot_core::{Contract, IndexResponse, TxResponse};

type TMintExec = extension::ExecuteMsg<RequestMsg>;
type TMintQuery = extension::QueryMsg<QueryMsg>;
#[boot_contract(TMintExec, extension::InstantiateMsg, TMintQuery, Empty)]
pub struct TMintStakingExtension;

impl<Chain: BootEnvironment> TMintStakingExtension<Chain>
where
    TxResponse<Chain>: IndexResponse,
{
    pub fn new(name: &str, chain: &Chain) -> Self {
        Self(
            Contract::new(name, chain).with_wasm_path("tendermint_staking"),
            // .with_mock(Box::new(
            //     ContractWrapper::new_with_empty(
            //         ::contract::execute,
            //         ::contract::instantiate,
            //         ::contract::query,
            //     ),
            // ))
        )
    }
}
