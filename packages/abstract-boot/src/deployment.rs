use boot_core::{prelude::*, BootEnvironment, BootError, TxHandler};
use semver::Version;

use crate::{
    get_native_contracts, get_os_core_contracts, AnsHost, IbcClient, Manager, ModuleFactory,
    OSFactory, Proxy, VersionControl,
};

pub struct Deployment<'a, Chain: BootEnvironment> {
    pub chain: &'a Chain,
    pub version: Version,
    pub ans_host: AnsHost<Chain>,
    pub version_control: VersionControl<Chain>,
    pub os_factory: OSFactory<Chain>,
    pub module_factory: ModuleFactory<Chain>,
    pub ibc_client: IbcClient<Chain>,
}

impl<'a, Chain: BootEnvironment> Deployment<'a, Chain> {
    pub fn new(chain: &'a Chain, version: Version) -> Self {
        let (ans_host, os_factory, version_control, module_factory, ibc_client) =
            get_native_contracts(chain);
        Self {
            chain,
            ans_host,
            version_control,
            ibc_client,
            os_factory,
            module_factory,
            version,
        }
    }

    pub fn deploy(&mut self) -> Result<(), BootError> {
        let sender = &self.chain.sender();

        // ########### Upload ##############

        self.ans_host.upload()?;
        self.version_control.upload()?;
        self.os_factory.upload()?;
        self.module_factory.upload()?;

        let mut os_core = OS::new(self.chain, None);
        os_core.upload()?;

        // ########### Instantiate ##############

        self.ans_host.instantiate(
            &abstract_os::ans_host::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.version_control.instantiate(
            &abstract_os::version_control::InstantiateMsg {},
            Some(sender),
            None,
        )?;

        self.module_factory.instantiate(
            &abstract_os::module_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        self.os_factory.instantiate(
            &abstract_os::os_factory::InstantiateMsg {
                version_control_address: self.version_control.address()?.into_string(),
                ans_host_address: self.ans_host.address()?.into_string(),
                module_factory_address: self.module_factory.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        // Set Factory
        self.version_control.execute(
            &abstract_os::version_control::ExecuteMsg::SetFactory {
                new_factory: self.os_factory.address()?.into_string(),
            },
            None,
        )?;

        // ########### upload modules and token ##############

        self.version_control
            .register_core(&os_core, &self.version.to_string())?;

        self.version_control.register_native(self)?;

        Ok(())
    }

    pub fn contracts(&self) -> Vec<&Contract<Chain>> {
        vec![
            self.ans_host.as_instance(),
            self.version_control.as_instance(),
            self.os_factory.as_instance(),
            self.module_factory.as_instance(),
            self.ibc_client.as_instance(),
        ]
    }
}

impl<'a> Deployment<'a, Daemon> {
    pub fn deploy_with_ibc_client(&mut self) -> Result<(), BootError> {
        let sender = &self.chain.sender();
        self.ibc_client.upload()?;
        self.ibc_client.instantiate(
            &abstract_os::ibc_client::InstantiateMsg {
                ans_host_address: self.ans_host.address()?.into_string(),
                version_control_address: self.version_control.address()?.into_string(),
                chain: self.chain.state.chain.chain_id.into(),
            },
            Some(sender),
            None,
        )?;
        self.deploy()
    }
}

pub struct OS<Chain: BootEnvironment> {
    pub manager: Manager<Chain>,
    pub proxy: Proxy<Chain>,
}

impl<Chain: BootEnvironment> OS<Chain> {
    pub fn new(chain: &Chain, os_id: Option<u32>) -> Self {
        let (manager, proxy) = get_os_core_contracts(chain, os_id);
        Self { manager, proxy }
    }

    pub fn upload(&mut self) -> Result<(), BootError> {
        self.manager.upload()?;
        self.proxy.upload()?;
        Ok(())
    }
}