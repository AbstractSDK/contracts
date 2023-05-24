
use abstract_core::ans_host::InstantiateMsg;
use cw_orch::daemon::sync::core::Daemon;
use cw_orch::deploy::Deploy;

use crate::Abstract;
use cw_orch::prelude::*;

#[test]
#[cfg(feature = "node-tests")]  
#[serial_test::serial]
fn test_deploy_abstract(){

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut daemon = Daemon::builder()
        .chain(networks::LOCAL_JUNO)
        .handle(runtime.handle())
        .build()
        .unwrap();


	let abstr = Abstract::load_from(&mut daemon).unwrap();

	// Now we upload abstract using the file loaded configuration

	abstr.ans_host.instantiate(&InstantiateMsg{

	}, None, None).unwrap();

}