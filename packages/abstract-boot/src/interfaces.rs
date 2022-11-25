mod ans_host;
mod dex_extension;
mod etf;
mod ibc_client;
mod idea_token;
mod manager;
mod module_factory;
mod os_factory;
mod osmosis_host;
mod proxy;
mod subscription;
mod tendermint_staking_extension;
mod version_control;
mod vesting;

pub use ans_host::AnsHost;
pub use dex_extension::DexExtension;
pub use etf::ETF;
pub use ibc_client::IbcClient;
pub use idea_token::Idea;
pub use manager::Manager;
pub use module_factory::ModuleFactory;
pub use os_factory::OSFactory;
pub use osmosis_host::OsmosisHost;
pub use proxy::Proxy;
pub use subscription::Subscription;
pub use tendermint_staking_extension::TMintStakingExtension;
pub use version_control::VersionControl;
pub use vesting::Vesting;

pub mod helpers {
    use boot_core::prelude::*;
    use std::rc::Rc;

    use super::{
        ans_host::AnsHost,
        manager::Manager,
        module_factory::ModuleFactory,
        proxy::Proxy, // subscription::Subscription, terraswap_dapp::Terraswap, liquidity_interface::ETF,
    };
    use super::{
        os_factory::OSFactory, version_control::VersionControl, DexExtension, IbcClient,
        OsmosisHost, Subscription, TMintStakingExtension,
    };
    use abstract_os::{
        ans_host, extension, module_factory, objects::gov_type::GovernanceDetails, os_factory,
        version_control, ANS_HOST, ETF, EXCHANGE, MANAGER, MODULE_FACTORY, OS_FACTORY, PROXY,
        SUBSCRIPTION, TENDERMINT_STAKING, VERSION_CONTROL,
    };
    use boot_core::{state::StateInterface, BootEnvironment, IndexResponse, TxHandler, TxResponse};
    use cosmwasm_std::Empty;
    use secp256k1::All;
    use semver::Version;

    pub fn get_native_contracts<Chain: BootEnvironment>(
        chain: &Chain,
    ) -> (
        AnsHost<Chain>,
        OSFactory<Chain>,
        VersionControl<Chain>,
        ModuleFactory<Chain>,
    )
    where
        <Chain as TxHandler>::Response: IndexResponse,
    {
        let ans_host = AnsHost::new(ANS_HOST, chain);
        let os_factory = OSFactory::new(OS_FACTORY, chain);
        let version_control = VersionControl::new(VERSION_CONTROL, chain);
        let module_factory = ModuleFactory::new(MODULE_FACTORY, chain);
        (ans_host, os_factory, version_control, module_factory)
    }

    pub fn get_os_core_contracts<Chain: BootEnvironment>(
        chain: &Chain,
        os_id: Option<u32>,
    ) -> (Manager<Chain>, Proxy<Chain>)
    where
        <Chain as TxHandler>::Response: IndexResponse,
    {
        if let Some(os_id) = os_id {
            let version_control = VersionControl::new(VERSION_CONTROL, chain);
            let core = version_control.get_os_core(os_id).unwrap();
            chain.state().set_address(MANAGER, &core.manager);
            chain.state().set_address(PROXY, &core.proxy);
            let manager = Manager::new(MANAGER, chain);
            let proxy = Proxy::new(PROXY, chain);
            (manager, proxy)
        } else {
            let manager = Manager::new(MANAGER, chain);
            let proxy = Proxy::new(PROXY, chain);
            (manager, proxy)
        }
    }

    pub fn get_apps<Chain: BootEnvironment>(
        chain: &Chain,
    ) -> (super::ETF<Chain>, Subscription<Chain>)
    where
        <Chain as TxHandler>::Response: IndexResponse,
    {
        let liquidity_interface = super::ETF::new(ETF, chain);
        let subscription = Subscription::new(SUBSCRIPTION, chain);
        (liquidity_interface, subscription)
    }

    pub fn get_extensions<Chain: BootEnvironment>(
        chain: &Chain,
    ) -> (DexExtension<Chain>, TMintStakingExtension<Chain>)
    where
        <Chain as TxHandler>::Response: IndexResponse,
    {
        let dex_extension = DexExtension::new(EXCHANGE, chain);
        let staking_extension = TMintStakingExtension::new(TENDERMINT_STAKING, chain);
        (dex_extension, staking_extension)
    }

    pub fn deploy_abstract<Chain: BootEnvironment>(
        chain: &Chain,
        abstract_os_version: Version,
    ) -> anyhow::Result<()>
    where
        TxResponse<Chain>: IndexResponse,
    {
        let (mut ans_host, mut os_factory, mut version_control, mut module_factory) =
            get_native_contracts(chain);
        let (mut manager, mut proxy) = get_os_core_contracts(chain, None);
        let (mut etf, mut subscription) = get_apps(chain);
        let (mut dex_ext, mut staking_ext) = get_extensions(chain);

        let sender = &chain.sender();

        // ########### upload ans_host ##############

        ans_host.upload()?;

        // ########### upload native ##############

        version_control.upload()?;
        os_factory.upload()?;
        module_factory.upload()?;

        // ########### upload core ##############

        manager.upload()?;
        proxy.upload()?;

        // ########### Instantiate ##############

        ans_host.instantiate(&ans_host::InstantiateMsg {}, Some(sender), None)?;

        version_control.instantiate(&version_control::InstantiateMsg {}, Some(sender), None)?;

        module_factory.instantiate(
            &module_factory::InstantiateMsg {
                version_control_address: version_control.address()?.into_string(),
                ans_host_address: ans_host.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        os_factory.instantiate(
            &os_factory::InstantiateMsg {
                version_control_address: version_control.address()?.into_string(),
                ans_host_address: ans_host.address()?.into_string(),
                module_factory_address: module_factory.address()?.into_string(),
            },
            Some(sender),
            None,
        )?;

        // Set Factory
        version_control.execute(
            &version_control::ExecuteMsg::SetFactory {
                new_factory: os_factory.address()?.into_string(),
            },
            None,
        )?;
        // ########### upload modules and token ##############

        version_control.upload_and_register_module(etf.as_instance_mut(), &abstract_os_version)?;
        version_control
            .upload_and_register_module(subscription.as_instance_mut(), &abstract_os_version)?;

        // ########### upload api and instantiate ##############
        let extension_init_msg = extension::InstantiateMsg {
            base: extension::BaseInstantiateMsg {
                ans_host_address: ans_host.address()?.into_string(),
                version_control_address: version_control.address()?.into_string(),
            },
            app: Empty {},
        };
        // Instantiate the DEX API
        version_control.upload_and_register_extension(
            dex_ext.as_instance_mut(),
            &extension_init_msg,
            &abstract_os_version,
        )?;

        version_control.upload_and_register_extension(
            staking_ext.as_instance_mut(),
            &extension_init_msg,
            &abstract_os_version,
        )?;

        // Updates code-ids and extensions on version control
        version_control.add_code_ids(abstract_os_version)?;

        Ok(())
    }
}
