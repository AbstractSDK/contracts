    use std::thread;
    
    use abstract_boot_integration_tests::follow_trail;
    use abstract_core::abstract_ica::IBC_APP_VERSION;
    use abstract_boot::IbcClient;
    use abstract_boot::OsmosisHost;
    use abstract_core::ibc_host::InstantiateMsg;
    use abstract_core::objects::chain_name::ChainName;
    use abstract_core::{ibc_client, IBC_CLIENT};
    use cw_orch::queriers::DaemonQuerier;
    use cw_orch::ContractInstance;
    use cw_orch::CwOrcExecute;

    use cw_orch::CwOrcQuery;
    use cw_orch::Daemon;
    use cw_orch::Deploy;

    use abstract_boot::Abstract;
    use anyhow::Result;
    use cw_orch::networks::JUNO_1;
    use cw_orch::CwOrcInstantiate;
    use cw_orch::CwOrcUpload;
    use cw_orch::InterchainInfrastructure;

    use cw_orch::networks::osmosis::OSMO_2;
    use cw_orch::queriers::Node;

    const JUNO_MNEMONIC: &str = "dilemma imitate split detect useful creek cart sort grow essence fish husband seven hollow envelope wedding host dry permit game april present panic move";
    const OSMOSIS_MNEMONIC: &str = "settle gas lobster judge silk stem act shoulder pluck waste pistol word comfort require early mouse provide marine butter crowd clock tube move wool";
    const JUNO: &str = "juno-1";
    const OSMOSIS: &str = "osmosis-2";
    const CONNECTION: &str = "connection-0";

    use clap::Parser;

    #[derive(Parser, Debug)]
    struct Cli {
        skip_abstract_upload: Option<bool>,
    }

    fn deploy_on_one_chain(chain: &Daemon) -> anyhow::Result<()> {
        let args = Cli::parse();

        let chain_abstr = if args.skip_abstract_upload.unwrap_or(false){
            Abstract::load_from(chain.clone())?
        }else{
            Abstract::deploy_on(chain.clone(), "1.0.0".parse().unwrap())?
        };

        // now deploy IBC stuff
        let client = IbcClient::new(IBC_CLIENT, chain.clone());
        let host = OsmosisHost::new("host", chain.clone());
        client.upload()?;
        host.upload()?;

        client.instantiate(
            &ibc_client::InstantiateMsg {
                ans_host_address: chain_abstr.ans_host.addr_str()?,
                chain: chain.state.chain_id.to_string(),
                version_control_address: chain_abstr.version_control.addr_str()?,
            },
            None,
            None,
        )?;

        host.instantiate(
            &InstantiateMsg {
                ans_host_address: chain_abstr.ans_host.addr_str()?,
                account_factory_address: chain_abstr.account_factory.addr_str()?,
                version_control_address: chain_abstr.version_control.addr_str()?,
            },
            None,
            None,
        )?;

        Ok(())
    }

    fn deploy_contracts(juno: &Daemon, osmosis: &Daemon) -> anyhow::Result<()> {
        deploy_on_one_chain(juno)?;
        deploy_on_one_chain(osmosis)?;
        Ok(())
    }

    fn create_channel(
        contract1: &dyn ContractInstance<Daemon>,
        contract2: &dyn ContractInstance<Daemon>,
        rt: &tokio::runtime::Runtime,
        interchain: &InterchainInfrastructure,
    ) -> Result<()> {


        log::info!("Start creating IBC connection between {} and {}", contract1.address()?, contract2.address()?);
        interchain
            .hermes
            .create_channel(rt, CONNECTION, IBC_APP_VERSION, contract1, contract2);

        log::info!("Channel creation complete between {} and {}, Sleeping 30 seconds", contract1.address()?, contract2.address()?);
        // wait for channel creation to complete
        std::thread::sleep(std::time::Duration::from_secs(30));


        // Then we query the LAST transactions that register the channel creation between those two ports and see if something matches
        // On chain 1
        let channel_creation_tx1 = &rt
            .block_on(
                Node::new(contract1.get_chain().channel()).find_tx_by_events(
                    vec![
                        format!(
                            "channel_open_ack.port_id='wasm.{}'",
                            contract1.address().unwrap()
                        ), // client is on chain1
                        format!(
                            "channel_open_ack.counterparty_port_id='wasm.{}'",
                            contract2.address().unwrap()
                        ), // host is on chain2
                        format!("channel_open_ack.connection_id='{}'", CONNECTION),
                    ],
                    None,
                    Some(cosmos_sdk_proto::cosmos::tx::v1beta1::OrderBy::Desc),
                ),
            )
            .unwrap()[0];

        let channel_creation_tx2 = &rt
            .block_on(
                Node::new(contract2.get_chain().channel()).find_tx_by_events(
                    vec![
                        format!(
                            "channel_open_confirm.port_id='wasm.{}'",
                            contract2.address().unwrap()
                        ),
                        format!(
                            "channel_open_confirm.counterparty_port_id='wasm.{}'",
                            contract1.address().unwrap()
                        ),
                        format!("channel_open_confirm.connection_id='{}'", CONNECTION),
                    ],
                    None,
                    Some(cosmos_sdk_proto::cosmos::tx::v1beta1::OrderBy::Desc),
                ),
            )
            .unwrap()[0];

        log::info!("Successfully created a channel between {} and {} on connection '{}' and channels {}:'{}'(txhash : {}) and {}:'{}(txhash : {})'", 
	    	contract1.address().unwrap(),
	    	contract2.address().unwrap(),
	    	CONNECTION,
	    	contract1.get_chain().state.chain_id,
	    	channel_creation_tx1.get_events("channel_open_ack")[0].get_first_attribute_value("channel_id").unwrap(),
	    	channel_creation_tx1.txhash,
	    	contract2.get_chain().state.chain_id,
	    	channel_creation_tx2.get_events("channel_open_confirm")[0].get_first_attribute_value("channel_id").unwrap(),
	    	channel_creation_tx2.txhash,
	    );

        // We follow the trail of channel creation to make sure we are doing the right thing and everything is setup alright

        let grpc_channel1 = contract1.get_chain().channel();
        let chain_id1 = contract1.get_chain().state.chain_id.clone();
        let tx_hash1 =  channel_creation_tx1.txhash.clone();

        let grpc_channel2 = contract2.get_chain().channel();
        let chain_id2 = contract2.get_chain().state.chain_id.clone();
        let tx_hash2 =  channel_creation_tx2.txhash.clone();

        let chain1_follow_thread = thread::spawn(|| follow_trail(
            grpc_channel1,
            chain_id1,
            tx_hash1
        ).unwrap());

        let chain2_follow_thread = thread::spawn(|| follow_trail(
            grpc_channel2,
            chain_id2,
            tx_hash2
        ).unwrap());

        chain1_follow_thread.join().unwrap();
        chain2_follow_thread.join().unwrap();

        Ok(())
    }

    fn join_host_and_clients(
        chain1: &Daemon,
        chain2: &Daemon,
        rt: &tokio::runtime::Runtime,
        interchain: &InterchainInfrastructure,
    ) -> anyhow::Result<()> {
        let client = IbcClient::new(IBC_CLIENT, chain1.clone());
        let host = OsmosisHost::new("host", chain2.clone());

        // First we register client and host respectively
        let chain1_name = chain1.state.chain_id.rsplitn(2, '-').collect::<Vec<&str>>()[1];
        let chain2_name = chain2.state.chain_id.rsplitn(2, '-').collect::<Vec<&str>>()[1];

        client.execute(
            &abstract_core::ibc_client::ExecuteMsg::RegisterChainHost {
                chain: chain2_name.to_string(),
                host: host.address()?.to_string(),
            },
            None,
        )?;
        host.execute(
            &abstract_core::ibc_host::ExecuteMsg::RegisterChainClient {
                chain_id: chain1_name.to_string(),
                client: client.address()?.to_string(),
            },
            None,
        )?;

        create_channel(&client, &host, rt, interchain)
    }

    fn ibc_abstract_setup() -> Result<()> {
        std::env::set_var("STATE_FILE", "daemon_state.json"); // Set in code for tests
        std::env::set_var("ARTIFACTS_DIR", "../artifacts"); // Set in code for tests
        std::env::set_var("RUST_LOG", "DEBUG"); // Set in code for tests
        std::env::set_var("MAIN_MNEMONIC", "toss visual amateur gospel receive panel employ flower wave barely marine have food blanket welcome chuckle anxiety find blast illegal rebuild inside silent squeeze");

        // Chains setup
        let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

        let interchain = InterchainInfrastructure::new(
            rt.handle(),
            vec![(JUNO_1, JUNO_MNEMONIC), (OSMO_2, OSMOSIS_MNEMONIC)],
        )?;

        let juno = interchain.daemon(JUNO)?;
        let osmosis = interchain.daemon(OSMOSIS)?;

        // Deploying abstract and the IBC abstract logic
        deploy_contracts(&juno, &osmosis)?;

        // Create the connection between client and host
        join_host_and_clients(&osmosis, &juno, &rt, &interchain)?;

        // Some tests to make sure the connection has been established between the 2 contracts
        // We query the channels for each host to see if the client has been connected
        let osmosis_client = IbcClient::new(IBC_CLIENT, osmosis);

        let osmosis_channels: ibc_client::ListChannelsResponse =
            osmosis_client.query(&ibc_client::QueryMsg::ListChannels {})?;

        assert_eq!(osmosis_channels.channels[0].0, ChainName::from("juno"));

        Ok(())
    }

    fn main(){
         ibc_abstract_setup().unwrap();
    }