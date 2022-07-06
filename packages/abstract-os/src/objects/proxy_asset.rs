use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, Env, QuerierWrapper, QueryRequest, StdError, StdResult,
    Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_asset::{Asset, AssetInfo};

use crate::{
    manager::state::OS_MODULES,
    memory::state::PAIR_POSTFIX,
    proxy::state::ADMIN,
    proxy::{state::VAULT_ASSETS, ExternalValueResponse, ValueQueryMsg},
};

use super::{
    memory::Memory,
    memory_entry::{AssetEntry, ContractEntry},
};

/// Every ProxyAsset provides a way to determine its value recursivly relative to
/// a base asset.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProxyAsset {
    pub asset: AssetEntry,
    // The value reference provides the tooling to get the value of the holding
    // relative to the base asset.
    pub value_reference: Option<ValueRef>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueRef {
    /// A pool name of an asset/asset pair
    /// Both assets must be defined in the Vault_assets state
    Pool { pair: ContractEntry },
    /// Liquidity pool token
    LiquidityToken {},
    /// Or a Proxy, the proxy also takes a Decimal (the multiplier)
    /// Asset will be valued as if they are Proxy tokens
    Proxy {
        proxy_asset: AssetEntry,
        multiplier: Decimal,
    },
    /// Query an external contract to get the value
    External { api_name: String },
}

impl ProxyAsset {
    /// Calculates the value of the asset through the optionally provided ValueReference
    /// TODO: improve efficiency
    pub fn value(
        &mut self,
        deps: Deps,
        env: &Env,
        mem: &Memory,
        set_holding: Option<Uint128>,
    ) -> StdResult<Uint128> {
        // Query how many of these tokens are held in the contract if not set.
        let asset_info = self.asset.resolve(deps, mem)?;
        let holding: Uint128 = match set_holding {
            Some(setter) => setter,
            None => asset_info.query_balance(&deps.querier, env.contract.address.clone())?,
        };

        let valued_asset = Asset::new(asset_info, holding);

        // Is there a reference to calculate the value?
        if let Some(value_reference) = self.value_reference.as_ref() {
            match value_reference {
                // A Pool refers to a swap pair that recursively leads to an asset/base_asset pool.
                ValueRef::Pool { pair } => {
                    return self.asset_value(deps, env, mem, valued_asset, pair)
                }
                // Liquidity is an LP token, value() fn is called recursively on both assets in the pool
                ValueRef::LiquidityToken {} => return self.lp_value(deps, env, mem, valued_asset),
                // A proxy asset is used instead
                ValueRef::Proxy {
                    proxy_asset,
                    multiplier,
                } => return proxy_value(deps, env, mem, proxy_asset, multiplier, holding),
                ValueRef::External { api_name } => {
                    let manager = ADMIN.get(deps)?.unwrap();
                    let maybe_api_addr = OS_MODULES.query(&deps.querier, manager, api_name)?;
                    if let Some(api_addr) = maybe_api_addr {
                        let response: ExternalValueResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: api_addr.to_string(),
                                msg: to_binary(&ValueQueryMsg {
                                    asset_name: self.asset.to_string(),
                                    amount: valued_asset.amount,
                                })?,
                            }))?;
                        return Ok(response.value);
                    } else {
                        return Err(StdError::generic_err(format!(
                            "external contract api {} must be enabled on OS",
                            api_name
                        )));
                    }
                }
            }
        }
        // If there is no valueref, it means this token is the base token.
        Ok(holding)
    }

    /// Calculates the value of an asset compared to some base asset through the provided trading pair.
    pub fn asset_value(
        &self,
        deps: Deps,
        env: &Env,
        mem: &Memory,
        valued_asset: Asset,
        pair: &ContractEntry,
    ) -> StdResult<Uint128> {
        let other_pool_asset: AssetEntry = other_asset_name(self.asset.as_str(), pair.as_str())?
            .to_string()
            .into();

        let pair_address = pair.resolve(deps, mem)?;
        let other_asset_info = other_pool_asset.resolve(deps, mem)?;

        // query assets held in pool, gives price
        let pool_info = (
            other_asset_info.query_balance(&deps.querier, &pair_address)?,
            valued_asset
                .info
                .query_balance(&deps.querier, pair_address)?,
        );

        // other / this
        let ratio = Decimal::from_ratio(pool_info.0.u128(), pool_info.1.u128());

        // Get the value of the current asset in the denom of the other asset
        let identifier = get_asset_identifier(&other_asset_info);
        let mut recursive_vault_asset = VAULT_ASSETS.load(deps.storage, &identifier)?;

        // #other = #this * (pool_other/pool_this)
        let amount_in_other_denom = valued_asset.amount * ratio;
        // Call value on this other asset.
        recursive_vault_asset.value(deps, env, mem, Some(amount_in_other_denom))
    }

    /// Calculate the value of an LP token
    /// Uses the lp token name to query pair pool for both assets
    pub fn lp_value(
        &self,
        deps: Deps,
        env: &Env,
        mem: &Memory,
        lp_asset: Asset,
    ) -> StdResult<Uint128> {
        let supply: Uint128;
        if let AssetInfo::Cw20(addr) = &lp_asset.info {
            supply = query_cw20_supply(&deps.querier, addr)?;
        } else {
            return Err(StdError::generic_err("Can't have a native LP token"));
        }

        // Get total supply of LP tokens and calculate share
        let share: Decimal = Decimal::from_ratio(lp_asset.amount, supply.u128());

        let other_pool_asset_names = pair_asset_names(self.asset.as_str());

        if other_pool_asset_names.len() != 2 {
            return Err(StdError::generic_err(format!(
                "lp token name {} must be composed of two assets.",
                self.asset.as_str()
            )));
        }

        let pair_address =
            mem.query_contract(deps, &format!("{}_{}", self.asset.as_str(), PAIR_POSTFIX))?;

        let asset_1 = mem.query_asset(deps, other_pool_asset_names[0])?;
        let asset_2 = mem.query_asset(deps, other_pool_asset_names[1])?;
        // query assets held in pool, gives price
        let (amount1, amount2) = (
            asset_1.query_balance(&deps.querier, &pair_address)?,
            asset_2.query_balance(&deps.querier, pair_address)?,
        );

        // load the assets
        let mut vault_asset_1: ProxyAsset =
            VAULT_ASSETS.load(deps.storage, get_asset_identifier(&asset_1).as_str())?;
        let mut vault_asset_2: ProxyAsset =
            VAULT_ASSETS.load(deps.storage, get_asset_identifier(&asset_2).as_str())?;

        // set the amounts to the LP holdings
        let vault_asset_1_amount = share * Uint128::new(amount1.u128());
        let vault_asset_2_amount = share * Uint128::new(amount2.u128());
        // Call value on these assets.
        Ok(
            vault_asset_1.value(deps, env, mem, Some(vault_asset_1_amount))?
                + vault_asset_2.value(deps, env, mem, Some(vault_asset_2_amount))?,
        )
    }
}

pub fn proxy_value(
    deps: Deps,
    env: &Env,
    mem: &Memory,
    proxy_asset: &AssetEntry,
    multiplier: &Decimal,
    holding: Uint128,
) -> StdResult<Uint128> {
    // Get the proxy asset
    let mut proxy_vault_asset: ProxyAsset = VAULT_ASSETS.load(
        deps.storage,
        get_asset_identifier(&proxy_asset.resolve(deps, mem)?).as_str(),
    )?;
    // call value on proxy asset with adjusted multiplier.
    proxy_vault_asset.value(deps, env, mem, Some(holding * *multiplier))
}
/// Get the other asset's name from a composite name
/// ex: asset= "btc" composite = "btc_eth_pair"
/// returns "eth"
fn other_asset_name<'a>(asset: &'a str, composite: &'a str) -> StdResult<&'a str> {
    composite
        .split('_')
        .filter(|component| *component != asset)
        .next()
        .ok_or(StdError::generic_err(format!(
            "composite {} is not structured correctly",
            composite
        )))
}

fn pair_asset_names(composite: &str) -> Vec<&str> {
    composite.split('_').collect()
}

/// The proxy struct acts as an Asset overwrite.
/// By setting this proxy you define the asset to be some
/// other asset with a multiplier.
/// For example: AssetInfo = bluna, BaseAsset = uusd, Proxy: luna, multiplier = 1
/// Each bluna would be valued as one luna.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proxy {
    // Proxy asset
    proxy_asset: String,
    // Can be set to some constant or set to price,
    multiplier: Decimal,
}

impl Proxy {
    pub fn new(multiplier: Decimal, proxy_asset: String) -> StdResult<Self> {
        Ok(Self {
            proxy_asset,
            multiplier,
        })
    }
}

/// Gets the identifier of the asset (either its denom or contract address)
pub fn get_asset_identifier(asset_info: &AssetInfo) -> String {
    match asset_info {
        AssetInfo::Native(denom) => denom.to_owned(),
        AssetInfo::Cw20(contract_addr) => contract_addr.into(),
    }
}

fn query_cw20_supply(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    let response: cw20::TokenInfoResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.into(),
            msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {})?,
        }))?;
    Ok(response.total_supply)
}
