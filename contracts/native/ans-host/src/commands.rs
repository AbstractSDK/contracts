use cosmwasm_std::{Addr, DepsMut, Empty, MessageInfo, Response, StdResult};
use cosmwasm_std::{Env, StdError, Storage};
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use abstract_os::ans_host::state::*;
use abstract_os::ans_host::{AssetPair, CompoundPoolId, DexAssetPairing, ExecuteMsg, UniqueId};
use abstract_os::dex::DexName;
use abstract_os::objects::pool_id::{PoolId, UncheckedPoolId};
use abstract_os::objects::pool_info::PoolMetadata;
use abstract_os::objects::{UncheckedChannelEntry, UncheckedContractEntry};

use crate::contract::AnsHostResult;
use crate::error::AnsHostError;
use crate::error::AnsHostError::InvalidAssetCount;

/// Handles the common base execute messages
pub fn handle_message(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    message: ExecuteMsg,
) -> AnsHostResult {
    match message {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateContractAddresses { to_add, to_remove } => {
            update_contract_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateAssetAddresses { to_add, to_remove } => {
            update_asset_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateChannels { to_add, to_remove } => {
            update_channels(deps, info, to_add, to_remove)
        }
        ExecuteMsg::RegisterDex { name } => register_dex(deps, info, name),
        ExecuteMsg::UpdatePools { to_add, to_remove } => {
            update_pools(deps, info, to_add, to_remove)
        }
    }
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Adds, updates or removes provided addresses.
pub fn update_contract_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedContractEntry, String)>,
    to_remove: Vec<UncheckedContractEntry>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_address) in to_add.into_iter() {
        let key = key.check();
        // validate addr
        // let addr = deps.as_ref().api.addr_validate(&new_address)?;
        // Update function for new or existing keys
        let insert = |_| -> StdResult<Addr> { Ok(Addr::unchecked(new_address)) };
        CONTRACT_ADDRESSES.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CONTRACT_ADDRESSES.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_asset_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(String, AssetInfoUnchecked)>,
    to_remove: Vec<String>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (name, new_asset) in to_add.into_iter() {
        // Update function for new or existing keys
        let extension = deps.api;
        let insert = |_| -> StdResult<AssetInfo> {
            // use own check, cw_asset otherwise changes cases to lowercase
            new_asset.check(extension, None)
        };
        ASSET_ADDRESSES.update(deps.storage, name.into(), insert)?;
    }

    for name in to_remove {
        ASSET_ADDRESSES.remove(deps.storage, name.into());
    }

    Ok(Response::new().add_attribute("action", "updated asset addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_channels(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedChannelEntry, String)>,
    to_remove: Vec<UncheckedChannelEntry>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_channel) in to_add.into_iter() {
        let key = key.check();
        // Update function for new or existing keys
        let insert = |_| -> StdResult<String> { Ok(new_channel) };
        CHANNELS.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CHANNELS.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Registers a new dex to the list of known dexes
fn register_dex(deps: DepsMut, info: MessageInfo, name: String) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let register = |dexes: Vec<String>| -> StdResult<Vec<String>> {
        if dexes.contains(&name) {
            return Err(StdError::generic_err(format!(
                "Dex {} is already registered",
                name
            )));
        }

        let mut dexes = dexes;
        dexes.push(name);
        Ok(dexes)
    };

    REGISTERED_DEXES.update(deps.storage, register)?;

    Ok(Response::new().add_attribute("action", "registered dex"))
}

const MAX_POOL_ASSETS: u8 = 5;

fn update_pools(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Vec<(UncheckedPoolId, PoolMetadata)>,
    to_remove: Vec<UniqueId>,
) -> AnsHostResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut unique_pool_id = CONFIG.load(deps.storage)?.next_unique_pool_id;

    for (pool_id, pool_metadata) in to_add.into_iter() {
        let pool_id = pool_id.check(deps.api)?;

        let mut assets = pool_metadata.assets.clone();
        validate_pool_assets(&assets)?;
        // Account for single asset pools
        if assets.len() == 1 {
            assets = vec![assets[0].clone(), assets[0].clone()];
        }

        let dex = pool_metadata.dex.clone();

        // Register each pair of assets as a pairing and link it to the pool id
        register_pool_pairings(deps.storage, unique_pool_id, pool_id, &assets, &dex)?;

        POOL_METADATA.save(deps.storage, unique_pool_id, &pool_metadata)?;

        unique_pool_id += 1;
    }

    for pool_id_to_remove in to_remove {
        // load the pool metadata
        let pool_metadata = POOL_METADATA.load(deps.storage, pool_id_to_remove)?;

        remove_pool_pairings(deps.storage, pool_id_to_remove, &pool_metadata)?;

        // remove the pool metadata
        POOL_METADATA.remove(deps.storage, pool_id_to_remove);
    }

    // Save the next unique pool id
    CONFIG.update(deps.storage, |mut config| -> StdResult<_> {
        config.next_unique_pool_id = unique_pool_id;
        Ok(config)
    })?;

    Ok(Response::new().add_attribute("action", "updated pools"))
}

// Execute an action on every asset pairing in the list of assets
fn exec_on_asset_pairings<T, A, E>(assets: &[String], mut action: A) -> StdResult<()>
where
    A: FnMut(AssetPair) -> Result<T, E>,
    StdError: From<E>,
{
    for (i, asset_x) in assets.iter().enumerate() {
        for (j, asset_y) in assets.iter().enumerate() {
            // Skip self-pairings
            if i == j {
                continue;
            }
            let pair: AssetPair = (asset_x.clone(), asset_y.clone());
            action(pair)?;
        }
    }
    Ok(())
}

fn register_pool_pairings(
    storage: &mut dyn Storage,
    next_pool_id: UniqueId,
    pool_id: PoolId,
    assets: &[String],
    dex: &DexName,
) -> StdResult<()> {
    let register_pairing = |(asset_x, asset_y)| {
        let key: DexAssetPairing = (asset_x, asset_y, dex.clone());
        let compound_pool_id: CompoundPoolId = (next_pool_id, pool_id.clone());

        register_asset_pairing(storage, key, compound_pool_id)
    };

    exec_on_asset_pairings(assets, register_pairing)
}

/// Register an asset pairing to its pool id
/// We ignore any duplicates, which is why we don't check for them
fn register_asset_pairing(
    storage: &mut dyn Storage,
    pair: DexAssetPairing,
    compound_pool_id: CompoundPoolId,
) -> Result<Vec<CompoundPoolId>, StdError> {
    let insert = |ids: Option<Vec<CompoundPoolId>>| -> StdResult<_> {
        let mut ids = ids.unwrap_or_default();

        ids.push(compound_pool_id);
        Ok(ids)
    };

    PAIR_TO_POOL_ID.update(storage, pair, insert)
}

fn remove_pool_pairings(
    storage: &mut dyn Storage,
    unique_pool_id: UniqueId,
    pool: &PoolMetadata,
) -> StdResult<()> {
    let remove_pairing = |(asset_x, asset_y)| -> Result<(), StdError> {
        let key: DexAssetPairing = (asset_x, asset_y, pool.dex.clone());

        let remove = |ids: Option<Vec<CompoundPoolId>>| -> StdResult<_> {
            let mut ids = ids.unwrap_or_default();
            ids.retain(|(id, _)| id != &unique_pool_id);
            Ok(ids)
        };

        let remaining_ids = PAIR_TO_POOL_ID.update(storage, key.clone(), remove)?;

        // If there are no remaining pools, remove the key
        if remaining_ids.is_empty() {
            PAIR_TO_POOL_ID.remove(storage, key);
        }
        Ok(())
    };

    exec_on_asset_pairings(&pool.assets, remove_pairing)
}

fn validate_pool_assets(assets: &[String]) -> Result<(), AnsHostError> {
    if assets.is_empty() || assets.len() > MAX_POOL_ASSETS as usize {
        return Err(InvalidAssetCount {
            max: MAX_POOL_ASSETS,
            provided: assets.len(),
        });
    }
    Ok(())
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> AnsHostResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
