use cosmwasm_std::Empty;

use terra_rust_script::{
    contract::{ContractInstance, Interface},
    sender::GroupConfig,
};

pub struct Template(pub ContractInstance<InstantiateMsg, ExecuteMsg, QueryMsg, Empty>);

impl Template  
{
    pub fn new(
        group_config: GroupConfig,
    ) -> Template {
        let instance = ContractInstance {
            interface: Interface::default(),
            group_config,
            name: "template".to_string(),
        };
        instance.check_scaffold().unwrap();
        Template(instance)
    }
}
