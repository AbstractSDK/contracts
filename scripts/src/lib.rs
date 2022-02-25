pub mod contract;
pub mod contract_instances;
mod error;
pub mod example;
pub mod multisig;
pub mod sender;

use dotenv::dotenv;
#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    if let Err(ref err) = example::demo().await {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));

        // The backtrace is not always generated. Try to run this example
        // with `$env:RUST_BACKTRACE=1`.
        //    if let Some(backtrace) = e.backtrace() {
        //        log::debug!("backtrace: {:?}", backtrace);
        //    }

        ::std::process::exit(1);
    }
}

mod macro_dev {
    use terra_rust_script_derive::execute;

    #[derive(Clone, Debug, execute)]
    /// Updates the addressbook
    pub enum ExecuteMsg {
        UpdateContractAddresses {
            to_add: Vec<(String, String)>,
            to_remove: Vec<String>,
        },
        UpdateAssetAddresses {
            to_add: Vec<(String, String)>,
            to_remove: Vec<String>,
        },
        /// Sets a new Admin
        SetAdmin { admin: String },
    }

    #[derive(Clone, Debug, execute)]
    /// Updates the addressbook
    pub struct InitMsg {}
}
