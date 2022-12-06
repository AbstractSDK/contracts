/// Macro for generating the mock contract implementation.
/// Example:
/// ```
/// export_test_contract!(etf::contract, etf_contract);
///```
/// translates to:
/// ```
/// /// Mock Contract Return
/// pub fn etf_contract() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
///     Box::new(
///         cw_multi_test::ContractWrapper::new_with_empty(
///             etf::contract::execute,
///             etf::contract::instantiate,
///             etf::contract::query,
///         )
///         .with_migrate_empty(etf::contract::migrate)
///         .with_reply(etf::contract::reply),
///     )
/// }
/// ```
#[macro_export]
macro_rules! export_test_contract {
    // See https://github.com/rust-lang/rust/issues/48067 for the mod parameter
    ($first:ident$(::$rest:ident)*, $fn_name:ident) => {

        /// Mock Contract Return
        pub fn $fn_name() -> Box<dyn cw_multi_test::Contract<cosmwasm_std::Empty>> {
            Box::new(
                cw_multi_test::ContractWrapper::new_with_empty(
                    $first$(::$rest)*::execute,
                    $first$(::$rest)*::instantiate,
                    $first$(::$rest)*::query,
                )
                .with_migrate_empty($first$(::$rest)*::migrate)
                .with_reply($first$(::$rest)*::reply),
            )
        }
    };
}
