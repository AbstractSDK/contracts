// pub fn get_module_version() -> Version {}

// /// Assert the dependencies that this app relies on are installed.
// pub fn assert_dependencies() -> StdResult<()> {
//     let app_dependencies = Dependencies::dependencies(self.base);
//     let manager_addr = self.base.manager_address(self.deps)?;
//     for dep in app_dependencies {
//         let maybe_app_version = self.app_version(dep.id);
//         if maybe_app_version.is_err() {
//             return Err(StdError::generic_err(format!(
//                 "Module {} not enabled on OS.",
//                 dep.id
//             )));
//         };
//         let app_version = Version::parse(&maybe_app_version?.version).unwrap();
//         for req in dep.version_req {
//             if !req.matches(&app_version) {
//                 return Err(StdError::generic_err(format!(
//                     "Module {} with version {} does not fit requirement {}.",
//                     dep.id, app_version, req
//                 )));
//             }
//         }
//     }
//     Ok(())
// }
