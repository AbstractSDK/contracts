use boot_core::BootEnvironment;

use crate::{version_control::VersionControl, ans_host::AnsHost, ibc_client::IbcClient, module_factory::ModuleFactory, os_factory::OSFactory, manager::Manager, proxy::Proxy};


pub struct Deployment<Chain: BootEnvironment> {
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub ibc_client: IbcClient<Chain>,
    pub os_factory: OSFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
}

impl<Chain: BootEnvironment> Deployment<Chain> {
    pub fn new(chain: &Chain) -> Self {
    let (ans_host, os_factory, version_control, _module_factory) = get_native_contracts(&chain);

     }

    
}

pub struct OS <Chain: BootEnvironment> {
    pub manager: Manager<Chain>,
    pub proxy: Proxy<Chain>
}