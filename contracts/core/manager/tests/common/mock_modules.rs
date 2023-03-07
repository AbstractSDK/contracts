#![allow(dead_code)]

use abstract_api::{mock::MockError as ApiMockError, ApiContract};
use abstract_app::mock::MockError as AppMockError;
use abstract_app::AppContract;
use abstract_boot::{ApiDeployer, AppDeployer};
use boot_core::{BootEnvironment, ContractWrapper};
use boot_core::{Empty, Mock};

pub type MockApiContract = ApiContract<ApiMockError, Empty, Empty, Empty>;
pub type MockAppContract = AppContract<AppMockError, Empty, Empty, Empty>;
pub use self::api_1::v1::BootMockApi1V1;
pub use self::api_1::v2::BootMockApi1V2;
pub use self::api_1::MOCK_API1_ID;
pub use self::api_2::v1::BootMockApi2V1;
pub use self::api_2::v2::BootMockApi2V2;
pub use self::api_2::MOCK_API2_ID;
pub use self::app_1::v1::BootMockApp1V1;
pub use self::app_1::v2::BootMockApp1V2;
pub use self::app_1::MOCK_APP1_ID;

pub const V1: &str = "1.0.0";
pub const V2: &str = "2.0.0";

/// deploys different version apis and app for migration testing
pub fn deploy_modules(mock: &Mock) {
    BootMockApi1V1::new(MOCK_API1_ID, mock.clone())
        .deploy(V1.parse().unwrap(), Empty {})
        .unwrap();

    // do same for version 2
    BootMockApi1V2::new(MOCK_API1_ID, mock.clone())
        .deploy(V2.parse().unwrap(), Empty {})
        .unwrap();

    // and now for api 2
    BootMockApi2V1::new(MOCK_API2_ID, mock.clone())
        .deploy(V1.parse().unwrap(), Empty {})
        .unwrap();

    // do same for version 2
    BootMockApi2V2::new(MOCK_API2_ID, mock.clone())
        .deploy(V2.parse().unwrap(), Empty {})
        .unwrap();

    // and now for app 1
    BootMockApp1V1::new(MOCK_APP1_ID, mock.clone())
        .deploy(V1.parse().unwrap())
        .unwrap();

    // do same for version 2
    BootMockApp1V2::new(MOCK_APP1_ID, mock.clone())
        .deploy(V2.parse().unwrap())
        .unwrap();
}

pub mod api_1 {
    use super::*;
    use abstract_os::api::ExecuteMsg;
    use abstract_os::api::InstantiateMsg;
    pub const MOCK_API1_ID: &str = "tester:mock-api1";

    pub mod v1 {
        use super::*;

        pub const MOCK_API1_V1: MockApiContract = MockApiContract::new(MOCK_API1_ID, V1, None);
        abstract_api::export_endpoints!(MOCK_API1_V1, MockApiContract);

        #[boot_core::boot_contract(InstantiateMsg, ExecuteMsg, Empty, Empty)]
        pub struct BootMockApi1V1;

        impl<Chain: BootEnvironment> ApiDeployer<Chain, Empty> for BootMockApi1V1<Chain> {}

        impl<Chain: boot_core::BootEnvironment> BootMockApi1V1<Chain> {
            pub fn new(name: &str, chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new(name, chain).with_mock(Box::new(ContractWrapper::<
                        ExecuteMsg,
                        _,
                        _,
                        _,
                        _,
                        _,
                    >::new_with_empty(
                        self::execute,
                        self::instantiate,
                        self::query,
                    ))),
                )
            }
        }
    }

    pub mod v2 {
        use super::*;

        pub const MOCK_API1_V2: MockApiContract = MockApiContract::new(MOCK_API1_ID, V2, None);
        abstract_api::export_endpoints!(MOCK_API1_V2, MockApiContract);

        #[boot_core::boot_contract(InstantiateMsg, Empty, Empty, Empty)]
        pub struct BootMockApi1V2;

        impl<Chain: BootEnvironment> ApiDeployer<Chain, Empty> for BootMockApi1V2<Chain> {}

        impl<Chain: boot_core::BootEnvironment> BootMockApi1V2<Chain> {
            pub fn new(name: &str, chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new(name, chain).with_mock(Box::new(ContractWrapper::<
                        ExecuteMsg,
                        _,
                        _,
                        _,
                        _,
                        _,
                    >::new_with_empty(
                        self::execute,
                        self::instantiate,
                        self::query,
                    ))),
                )
            }
        }
    }
}

pub mod api_2 {
    use super::*;

    use abstract_os::api::ExecuteMsg;
    use abstract_os::api::InstantiateMsg;
    pub const MOCK_API2_ID: &str = "tester:mock-api2";
    pub mod v1 {
        use super::*;

        pub const MOCK_API2_V1: MockApiContract = MockApiContract::new(MOCK_API2_ID, V1, None);
        abstract_api::export_endpoints!(MOCK_API2_V1, MockApiContract);

        #[boot_core::boot_contract(InstantiateMsg, ExecuteMsg, Empty, Empty)]
        pub struct BootMockApi2V1;

        impl<Chain: BootEnvironment> ApiDeployer<Chain, Empty> for BootMockApi2V1<Chain> {}

        impl<Chain: boot_core::BootEnvironment> BootMockApi2V1<Chain> {
            pub fn new(name: &str, chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new(name, chain).with_mock(Box::new(ContractWrapper::<
                        ExecuteMsg,
                        _,
                        _,
                        _,
                        _,
                        _,
                    >::new_with_empty(
                        self::execute,
                        self::instantiate,
                        self::query,
                    ))),
                )
            }
        }
    }

    pub mod v2 {

        use super::*;

        pub const MOCK_API2_V2: MockApiContract = MockApiContract::new(MOCK_API2_ID, V2, None);
        abstract_api::export_endpoints!(MOCK_API2_V2, MockApiContract);

        #[boot_core::boot_contract(InstantiateMsg, ExecuteMsg, Empty, Empty)]
        pub struct BootMockApi2V2;

        impl<Chain: BootEnvironment> ApiDeployer<Chain, Empty> for BootMockApi2V2<Chain> {}

        impl<Chain: boot_core::BootEnvironment> BootMockApi2V2<Chain> {
            pub fn new(name: &str, chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new(name, chain).with_mock(Box::new(ContractWrapper::<
                        ExecuteMsg,
                        _,
                        _,
                        _,
                        _,
                        _,
                    >::new_with_empty(
                        self::execute,
                        self::instantiate,
                        self::query,
                    ))),
                )
            }
        }
    }
}

// app 1 depends on api 1 and api 2
pub mod app_1 {
    use super::*;
    use abstract_os::app::*;

    pub const MOCK_APP1_ID: &str = "tester:mock-app1";
    pub mod v1 {
        use super::*;
        use crate::common::mock_modules::{api_1::MOCK_API1_ID, api_2::MOCK_API2_ID};
        use abstract_boot::AppDeployer;
        use abstract_os::objects::dependency::StaticDependency;

        /// App that depends on API1 v1 and API2 v1
        pub const MOCK_APP1_V1: MockAppContract = MockAppContract::new(MOCK_APP1_ID, V1, None)
            .with_dependencies(&[
                StaticDependency::new(MOCK_API1_ID, &[V1]),
                StaticDependency::new(MOCK_API2_ID, &[V1]),
            ]);

        abstract_app::export_endpoints!(MOCK_APP1_V1, MockAppContract);

        #[boot_core::boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
        pub struct BootMockApp1V1;

        impl<Chain: BootEnvironment> AppDeployer<Chain> for BootMockApp1V1<Chain> {}

        impl<Chain: boot_core::BootEnvironment> BootMockApp1V1<Chain> {
            pub fn new(name: &str, chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new(name, chain).with_mock(Box::new(
                        ContractWrapper::new_with_empty(
                            self::execute,
                            self::instantiate,
                            self::query,
                        )
                        .with_migrate(self::migrate),
                    )),
                )
            }
        }
    }

    pub mod v2 {
        use super::*;
        use crate::common::mock_modules::{api_1::MOCK_API1_ID, api_2::MOCK_API2_ID};
        use abstract_boot::AppDeployer;
        use abstract_os::objects::dependency::StaticDependency;

        /// App that depends on API1 v2 and API2 v2
        pub const MOCK_APP1_V2: MockAppContract = MockAppContract::new(MOCK_APP1_ID, V2, None)
            .with_dependencies(&[
                StaticDependency::new(MOCK_API1_ID, &[V2]),
                StaticDependency::new(MOCK_API2_ID, &[V2]),
            ]);
        abstract_app::export_endpoints!(MOCK_APP1_V2, MockAppContract);

        #[boot_core::boot_contract(InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg)]
        pub struct BootMockApp1V2;

        impl<Chain: BootEnvironment> AppDeployer<Chain> for BootMockApp1V2<Chain> {}

        impl<Chain: boot_core::BootEnvironment> BootMockApp1V2<Chain> {
            pub fn new(name: &str, chain: Chain) -> Self {
                Self(
                    boot_core::Contract::new(name, chain).with_mock(Box::new(
                        ContractWrapper::new_with_empty(
                            self::execute,
                            self::instantiate,
                            self::query,
                        )
                        .with_migrate(self::migrate),
                    )),
                )
            }
        }
    }
}
