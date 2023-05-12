    use abstract_core::manager::InfoResponse;
    use abstract_boot::{OsmosisHost, Manager};

    use cw_orch::{ContractInstance};
    use abstract_core::{PROXY, objects::{AccountId, account::AccountTrace}, manager::ConfigResponse};
    use abstract_boot::{ManagerExecFns, ManagerQueryFns, AccountFactoryExecFns};
    use cosmwasm_std::to_binary;
    use cw_orch::Daemon;
    use std::thread;
    use crate::follow_ibc_trail::follow_trail;
    use abstract_core::objects::chain_name::ChainName;
    
    use abstract_boot::{AccountDetails, IbcClient};
    use abstract_core::IBC_CLIENT;
    use cw_orch::{Deploy, InterchainInfrastructure, networks::JUNO_1, networks::OSMO_2};
    use abstract_boot::Abstract;
    use anyhow::Result;

    const JUNO_MNEMONIC: &str = "dilemma imitate split detect useful creek cart sort grow essence fish husband seven hollow envelope wedding host dry permit game april present panic move";
    const OSMOSIS_MNEMONIC: &str = "settle gas lobster judge silk stem act shoulder pluck waste pistol word comfort require early mouse provide marine butter crowd clock tube move wool";
    const JUNO: &str = "juno-1";
    const OSMOSIS: &str = "osmosis-2";

    fn set_env(){
        std::env::set_var("STATE_FILE", "daemon_state.json"); // Set in code for tests
        std::env::set_var("ARTIFACTS_DIR", "../artifacts"); // Set in code for tests
        std::env::set_var("RUST_LOG", "DEBUG"); // Set in code for tests
        std::env::set_var("MAIN_MNEMONIC", JUNO_MNEMONIC); // Set in code for tests (used only for our weird follow trail function (not optimal))
    }

    fn set_interchain_env(rt: &tokio::runtime::Runtime) -> Result<(Daemon, Daemon)>{


        let interchain = InterchainInfrastructure::new(
            rt.handle(),
            vec![(JUNO_1, JUNO_MNEMONIC), (OSMO_2, OSMOSIS_MNEMONIC)],
        )?;

        let juno = interchain.daemon(JUNO)?;
        let osmosis = interchain.daemon(OSMOSIS)?;

        Ok((juno, osmosis))
    }

    #[test]
    fn test_create_ibc_account() {
        set_env();
            
        // We start by creating an abstract account
        let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
        let (juno, osmosis) = set_interchain_env(&rt).unwrap();

        let juno_abstr = Abstract::load_from(juno.clone()).unwrap();
        let osmo_abstr = Abstract::load_from(osmosis.clone()).unwrap();

        // Create a local account
        let account_name = "osmo-test".to_string();
        let description = Some("Description of the account".to_string());
        let link = Some("https://google.com".to_string());
        osmo_abstr.account_factory.create_new_account(AccountDetails{
            name:account_name.clone(),
            description: description.clone(),
            link: link.clone()
        }, abstract_core::objects::gov_type::GovernanceDetails::Monarchy { monarch: osmosis.sender.address().unwrap().to_string() }).unwrap();
    
        // We need to register the ibc client as a module of the manager
        let osmo_client = IbcClient::new(IBC_CLIENT, osmosis.clone());
        osmo_abstr.account.manager.update_module_addresses(Some(vec![(IBC_CLIENT.to_string(), osmo_client.address().unwrap().to_string())]), None).unwrap();

        // We need to register the ibc host in the distant chain account factory
        let juno_host = OsmosisHost::new("host", juno.clone());
        juno_abstr.account_factory.update_config(None, Some(juno_host.address().unwrap().to_string()), None, None).unwrap();


        // Now we send a message to the client saying that we want to create an account on osmosis
        let register_tx = osmo_abstr.account.manager.exec_on_module(to_binary(&abstract_core::proxy::ExecuteMsg::IbcAction{
            msgs: vec![abstract_core::ibc_client::ExecuteMsg::Register{
                host_chain: ChainName::from("juno")
            }]
        }).unwrap(), PROXY.to_string()).unwrap();

        let grpc_channel = osmosis.channel();
        let chain_id = osmosis.state.chain_id.clone();
        // Follow the IBC trail of this transaction
        thread::spawn(|| follow_trail(
            grpc_channel,
            chain_id,
            register_tx.txhash
        ).unwrap()).join().unwrap();

        // After this is all ended, we query the accounts to make sure everything is executed and setup alright on the distant chain
        // First we query the account id from the manager
        let account_config = osmo_abstr.account.manager.config().unwrap();

        let distant_account = AccountId::new(account_config.account_id.seq(),AccountTrace::Remote(vec![ChainName::from("osmosis")])).unwrap();
        let distant_account_config = juno_abstr.version_control.get_account(distant_account.clone()).unwrap();
        // This shouldn't fail as we have just created an account using those characteristics
        log::info!("Distant account config {:?} ",distant_account_config);

        let distant_manager = Manager::new("distant_account_manager", juno);
        distant_manager.set_address(&distant_account_config.manager);
        
        // Now we need to test some things about this account on the juno chain
        let manager_config = distant_manager.config().unwrap();
        assert_eq!(manager_config, ConfigResponse{
            account_id: distant_account,
            is_suspended: false,
            module_factory_address: juno_abstr.module_factory.address().unwrap(),
            version_control_address: juno_abstr.version_control.address().unwrap(),
        });

        let manager_info = distant_manager.info().unwrap();
        assert_eq!(manager_info, InfoResponse{
            info: abstract_core::manager::state::AccountInfo { 
                name: account_name,
                governance_details: abstract_core::objects::gov_type::GovernanceDetails::External { governance_address: juno_host.address().unwrap(), governance_type: "abstract-ibc".to_string() }, 
                chain_id: "juno-1".to_string(),
                description,
                link
            }
        });
    }
