use cw_multi_test::{App, Executor};
use cosmwasm_std::{Addr, to_binary};
use abstract_os::objects::module::ModuleInfo;
use abstract_os::version_control::Core;
use anyhow::Result as AnyResult;
use serde::Serialize;

pub fn install_module<TInitMsg>(
    app: &mut App,
    sender: &Addr,
    module: ModuleInfo,
    core: &Core,
    init_msg: &TInitMsg,
) -> AnyResult<()> where TInitMsg: Serialize + ?Sized {

    let installation_msg = abstract_os::manager::ExecuteMsg::InstallModule {
        module,
        init_msg: Some(to_binary(init_msg)?),
    };

    let resp = app
        .execute_contract(sender.clone(), core.manager.clone(), &installation_msg, &[])
        .unwrap();

    Ok(())
}
