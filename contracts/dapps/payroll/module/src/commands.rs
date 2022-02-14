use cosmwasm_std::{from_binary, Addr, DepsMut, Env, MessageInfo, Response, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::{Asset, AssetInfo};

use pandora::treasury::dapp_base::state::{BaseState, BASESTATE, ADMIN};

use crate::contract::PaymentResult;
use crate::error::PaymentError;
use crate::msg::DepositHookMsg;
use crate::state::{CONFIG, CUSTOMERS, Compensation, STATE, CONTRIBUTORS, MONTH, State, CLIENTS};

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> PaymentResult {
    match from_binary(&cw20_msg.msg)? {
        DepositHookMsg::Pay { os_id } => {
            // Construct deposit asset
            let asset = Asset {
                info: AssetInfo::Token {
                    contract_addr: msg_info.sender.to_string(),
                },
                amount: cw20_msg.amount,
            };
            try_pay(deps, msg_info, asset, Some(cw20_msg.sender), os_id)
        }
    }
}

/// Called when either paying with a native token or when paying
/// with a CW20.
pub fn try_pay(
    deps: DepsMut,
    msg_info: MessageInfo,
    asset: Asset,
    sender: Option<String>,
    os_id: u32,
) -> PaymentResult {
    // Load all needed states
    let config = CONFIG.load(deps.storage)?;

    // Get the liquidity provider address
    match sender {
        Some(addr) => Addr::unchecked(addr),
        None => {
            // Check if deposit matches claimed deposit.
            if asset.is_native_token() {
                // If native token, assert claimed amount is correct
                asset.assert_sent_native_token_balance(&msg_info)?;
                msg_info.sender
            } else {
                // Can't add liquidity with cw20 if not using the hook
                return Err(PaymentError::NotUsingCW20Hook {});
            }
        }
    };

    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(PaymentError::WrongToken {});
    }

    let mut customer_balance = CUSTOMERS.data.load(deps.storage, &os_id.to_be_bytes())?;
    customer_balance.increase(asset.amount);
    CUSTOMERS
        .data
        .save(deps.storage, &os_id.to_be_bytes(), &customer_balance)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Deposit to payment module")),
        ("Received funds:", asset.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}

/// Function that adds/updates the contributor config of a given address
pub fn update_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    contributor_addr: String,
    mut compensation: Compensation,
) -> PaymentResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Load all needed states
    let mut state = STATE.load(deps.storage)?;

    let maybe_compensation = CONTRIBUTORS.data.may_load(deps.storage, &contributor_addr.as_bytes())?;

    match maybe_compensation {
        Some(current_compensation) => {
            let weight_diff: i32 = current_compensation.weight as i32 - compensation.weight as i32;
            let base_diff: i32 = current_compensation.base as i32 - compensation.base as i32;
            state.total_weight = Uint128::from((state.total_weight.u128() as i128 + weight_diff as i128) as u128);
            state.expense = Uint128::from((state.expense.u128() as i128 + base_diff as i128) as u128);
        }
        None => {
            state.total_weight += Uint128::from(compensation.weight);
            state.expense += Uint128::from(compensation.base);
            // Can only get paid on pay day after next pay day
            compensation.first_pay_day = state.next_pay_day + Uint64::from(MONTH);
        }
    };
    
    CONTRIBUTORS.data.save(deps.storage, &contributor_addr.as_bytes(), &compensation)?;
    STATE.save(deps.storage, &state)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Update Compensation")),
        ("For:", contributor_addr.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}


/// Called when either paying with a native token or when paying
/// with a CW20.
pub fn remove_contributor(
    deps: DepsMut,
    msg_info: MessageInfo,
    contributor_addr: String,
) -> PaymentResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Load all needed states
    let mut state = STATE.load(deps.storage)?;

    let maybe_compensation = CONTRIBUTORS.data.may_load(deps.storage, &contributor_addr.as_bytes())?;

    match maybe_compensation {
        Some(current_compensation) => {
            state.total_weight -= Uint128::from(current_compensation.weight);
            state.expense -= Uint128::from(current_compensation.base);
            // Can only get paid on pay day after next pay day
            CONTRIBUTORS.data.remove(deps.storage, &contributor_addr.as_bytes());
            STATE.save(deps.storage, &state)?;
        }
        None => {
            return Err(PaymentError::ContributorNotRegistered{})
        }
    };
    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Remove Contributor")),
        ("Address:", contributor_addr.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}


pub fn try_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> PaymentResult {
    let state: State = STATE.load(deps.storage)?;

    // Are we beyond the next pay time?
    if state.next_pay_day.u64() < env.block.time.seconds() {
        // First tally income, then set next block time 
        tally_income(deps, env)?;
    }
    
    let assets = memory.query_assets(deps.as_ref(), &pool.assets)?;

    // Logging var
    let mut attrs = vec![
        ("Action:", String::from("Withdraw from vault")),
        ("Received liquidity tokens:", amount.to_string()),
    ];

    // Calculate share of pool and requested pool value
    let total_share: Uint128 = query_supply(&deps.querier, state.liquidity_token_addr.clone())?;

    // Get treasury fee in LP tokens
    let treasury_fee = fee.compute(amount);

    // Share with fee deducted.
    let share_ratio: Decimal = Decimal::from_ratio(amount - treasury_fee, total_share);

    // Init response
    let response = Response::new();

    // LP token fee
    let lp_token_treasury_fee = Asset {
        info: AssetInfo::Token {
            contract_addr: state.liquidity_token_addr.to_string(),
        },
        amount: treasury_fee,
    };

    // Construct treasury fee msg
    let treasury_fee_msg = fee.msg(
        deps.as_ref(),
        lp_token_treasury_fee,
        base_state.treasury_address.clone(),
    )?;
    attrs.push(("Treasury fee:", treasury_fee.to_string()));

    // Get asset holdings of vault and calculate amount to return
    let mut pay_back_assets: Vec<Asset> = vec![];
    // Get asset holdings of vault and calculate amount to return
    for (_, info) in assets.into_iter() {
        pay_back_assets.push(Asset {
            info: info.clone(),
            amount: share_ratio
                // query asset held in treasury
                * query_asset_balance(
                    deps.as_ref(),
                    &info.clone(),
                    base_state.treasury_address.clone(),
                )
                .unwrap(),
        });
    }

    // Construct repay msgs
    let mut refund_msgs: Vec<CosmosMsg> = vec![];
    for asset in pay_back_assets.into_iter() {
        if asset.amount != Uint128::zero() {
            // Unchecked ok as sender is already validated by VM
            refund_msgs.push(
                asset
                    .clone()
                    .into_msg(&deps.querier, Addr::unchecked(sender.clone()))?,
            );
            attrs.push(("Repaying:", asset.to_string()));
        }
    }

    // Msg that gets called on the vault address
    let vault_refund_msg = send_to_treasury(refund_msgs, &base_state.treasury_address)?;

    // LP burn msg
    let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.liquidity_token_addr.into(),
        // Burn exludes fee
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: (amount - treasury_fee),
        })?,
        funds: vec![],
    });

    Ok(response
        .add_attribute("Action:", "Withdraw Liquidity")
        // Transfer fee
        .add_message(treasury_fee_msg)
        // Burn LP tokens
        .add_message(burn_msg)
        // Send treasury funds to owner
        .add_message(vault_refund_msg)
        .add_attributes(attrs))
}

fn tally_income(deps: DepsMut, env: Env) -> StdResult<()> {
    if !CLIENTS.status. {

    }
}

// /// Updates the pool information
// pub fn update_pool(
//     deps: DepsMut,
//     msg_info: MessageInfo,
//     deposit_asset: Option<String>,
//     assets_to_add: Vec<String>,
//     assets_to_remove: Vec<String>,
// ) -> PaymentResult {
//     // Only the admin should be able to call this
//     ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

//     let mut pool = POOL.load(deps.storage)?;

//     // If provided, update pool
//     if let Some(deposit_asset) = deposit_asset {
//         pool.deposit_asset = deposit_asset;
//     }

//     // Add the asset to the vector if not already present
//     for asset in assets_to_add.into_iter() {
//         if !pool.assets.contains(&asset) {
//             pool.assets.push(asset)
//         } else {
//             return Err(PaymentError::AssetAlreadyPresent { asset });
//         }
//     }

//     // Remove asset from vector if present
//     for asset in assets_to_remove.into_iter() {
//         if pool.assets.contains(&asset) {
//             pool.assets.retain(|x| *x != asset)
//         } else {
//             return Err(PaymentError::AssetNotPresent { asset });
//         }
//     }

//     // Save pool
//     POOL.save(deps.storage, &pool)?;
//     Ok(Response::new().add_attribute("Update:", "Successful"))
// }

// pub fn set_fee(deps: DepsMut, msg_info: MessageInfo, new_fee: Fee) -> PaymentResult {
//     // Only the admin should be able to call this
//     ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

//     if new_fee.share > Decimal::one() {
//         return Err(PaymentError::InvalidFee {});
//     }

//     FEE.save(deps.storage, &new_fee)?;
//     Ok(Response::new().add_attribute("Update:", "Successful"))
// }
