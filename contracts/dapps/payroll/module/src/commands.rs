use cosmwasm_std::{from_binary, Addr, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;
use terraswap::asset::{Asset, AssetInfo};

use pandora::treasury::dapp_base::state::{BaseState, BASESTATE};

use crate::contract::PayrollResult;
use crate::error::PayrollError;
use crate::msg::DepositHookMsg;
use crate::state::{CONFIG, CUSTOMERS};

/// handler function invoked when the vault dapp contract receives
/// a transaction. In this case it is triggered when either a LP tokens received
/// by the contract or when the deposit asset is a cw20 asset.
pub fn receive_cw20(
    deps: DepsMut,
    _env: Env,
    msg_info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> PayrollResult {
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
) -> PayrollResult {
    // Load all needed states
    let config = CONFIG.load(deps.storage)?;
    let base_state: BaseState = BASESTATE.load(deps.storage)?;
    let _memory = base_state.memory;

    // Get the liquidity provider address
    let _liq_provider = match sender {
        Some(addr) => Addr::unchecked(addr),
        None => {
            // Check if deposit matches claimed deposit.
            if asset.is_native_token() {
                // If native token, assert claimed amount is correct
                asset.assert_sent_native_token_balance(&msg_info)?;
                msg_info.sender
            } else {
                // Can't add liquidity with cw20 if not using the hook
                return Err(PayrollError::NotUsingCW20Hook {});
            }
        }
    };

    // Construct deposit info
    let deposit_info = config.payment_asset;

    // Assert payment asset and claimed asset infos are the same
    if deposit_info != asset.info {
        return Err(PayrollError::WrongToken {});
    }

    let mut customer_balance = CUSTOMERS.data.load(deps.storage, &os_id.to_string())?;
    customer_balance.increase(asset.amount);
    CUSTOMERS
        .data
        .save(deps.storage, &os_id.to_string(), &customer_balance)?;

    // Init vector for logging
    let attrs = vec![
        ("Action:", String::from("Deposit to payment module")),
        ("Received funds:", asset.to_string()),
    ];

    Ok(Response::new().add_attributes(attrs))
}

// /// Attempt to withdraw deposits. Fees are calculated and deducted in liquidity tokens.
// /// This allowes the war-chest to accumulate a stake in the vault.
// /// The refund is taken out of Anchor if possible.
// /// Luna holdings are not eligible for withdrawal.
// pub fn try_withdraw_liquidity(
//     deps: DepsMut,
//     _env: Env,
//     sender: String,
//     amount: Uint128,
// ) -> PayrollResult {
//     let pool: Pool = POOL.load(deps.storage)?;
//     let state: State = STATE.load(deps.storage)?;
//     let base_state: BaseState = BASESTATE.load(deps.storage)?;
//     let memory = base_state.memory;
//     let fee: Fee = FEE.load(deps.storage)?;
//     // Get assets
//     let assets = memory.query_assets(deps.as_ref(), &pool.assets)?;

//     // Logging var
//     let mut attrs = vec![
//         ("Action:", String::from("Withdraw from vault")),
//         ("Received liquidity tokens:", amount.to_string()),
//     ];

//     // Calculate share of pool and requested pool value
//     let total_share: Uint128 = query_supply(&deps.querier, state.liquidity_token_addr.clone())?;

//     // Get treasury fee in LP tokens
//     let treasury_fee = fee.compute(amount);

//     // Share with fee deducted.
//     let share_ratio: Decimal = Decimal::from_ratio(amount - treasury_fee, total_share);

//     // Init response
//     let response = Response::new();

//     // LP token fee
//     let lp_token_treasury_fee = Asset {
//         info: AssetInfo::Token {
//             contract_addr: state.liquidity_token_addr.to_string(),
//         },
//         amount: treasury_fee,
//     };

//     // Construct treasury fee msg
//     let treasury_fee_msg = fee.msg(
//         deps.as_ref(),
//         lp_token_treasury_fee,
//         base_state.treasury_address.clone(),
//     )?;
//     attrs.push(("Treasury fee:", treasury_fee.to_string()));

//     // Get asset holdings of vault and calculate amount to return
//     let mut pay_back_assets: Vec<Asset> = vec![];
//     // Get asset holdings of vault and calculate amount to return
//     for (_, info) in assets.into_iter() {
//         pay_back_assets.push(Asset {
//             info: info.clone(),
//             amount: share_ratio
//                 // query asset held in treasury
//                 * query_asset_balance(
//                     deps.as_ref(),
//                     &info.clone(),
//                     base_state.treasury_address.clone(),
//                 )
//                 .unwrap(),
//         });
//     }

//     // Construct repay msgs
//     let mut refund_msgs: Vec<CosmosMsg> = vec![];
//     for asset in pay_back_assets.into_iter() {
//         if asset.amount != Uint128::zero() {
//             // Unchecked ok as sender is already validated by VM
//             refund_msgs.push(
//                 asset
//                     .clone()
//                     .into_msg(&deps.querier, Addr::unchecked(sender.clone()))?,
//             );
//             attrs.push(("Repaying:", asset.to_string()));
//         }
//     }

//     // Msg that gets called on the vault address
//     let vault_refund_msg = send_to_treasury(refund_msgs, &base_state.treasury_address)?;

//     // LP burn msg
//     let burn_msg = CosmosMsg::Wasm(WasmMsg::Execute {
//         contract_addr: state.liquidity_token_addr.into(),
//         // Burn exludes fee
//         msg: to_binary(&Cw20ExecuteMsg::Burn {
//             amount: (amount - treasury_fee),
//         })?,
//         funds: vec![],
//     });

//     Ok(response
//         .add_attribute("Action:", "Withdraw Liquidity")
//         // Transfer fee
//         .add_message(treasury_fee_msg)
//         // Burn LP tokens
//         .add_message(burn_msg)
//         // Send treasury funds to owner
//         .add_message(vault_refund_msg)
//         .add_attributes(attrs))
// }

// /// Updates the pool information
// pub fn update_pool(
//     deps: DepsMut,
//     msg_info: MessageInfo,
//     deposit_asset: Option<String>,
//     assets_to_add: Vec<String>,
//     assets_to_remove: Vec<String>,
// ) -> PayrollResult {
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
//             return Err(PayrollError::AssetAlreadyPresent { asset });
//         }
//     }

//     // Remove asset from vector if present
//     for asset in assets_to_remove.into_iter() {
//         if pool.assets.contains(&asset) {
//             pool.assets.retain(|x| *x != asset)
//         } else {
//             return Err(PayrollError::AssetNotPresent { asset });
//         }
//     }

//     // Save pool
//     POOL.save(deps.storage, &pool)?;
//     Ok(Response::new().add_attribute("Update:", "Successful"))
// }

// pub fn set_fee(deps: DepsMut, msg_info: MessageInfo, new_fee: Fee) -> PayrollResult {
//     // Only the admin should be able to call this
//     ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

//     if new_fee.share > Decimal::one() {
//         return Err(PayrollError::InvalidFee {});
//     }

//     FEE.save(deps.storage, &new_fee)?;
//     Ok(Response::new().add_attribute("Update:", "Successful"))
// }
