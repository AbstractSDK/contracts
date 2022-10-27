use crate::{
    dex_trait::{Fee, FeeOnInput, Return, Spread},
    error::DexError,
    DEX,
};

use cosmwasm_std::{
    from_binary, to_binary, Addr, Coin, CosmosMsg, Decimal, Deps, QueryRequest, StdResult, Uint128,
};
use cw_asset::{Asset, AssetInfo};

use osmosis_std::types::{
    cosmos::base::v1beta1::Coin as OsmoCoin,
    osmosis::gamm::v1beta1::{
        Pool, QueryPoolParamsRequest, QueryPoolParamsResponse, QueryPoolRequest, QueryPoolResponse,
    },
};

use osmosis_std::types::osmosis::gamm::v1beta1::{
    MsgExitPool, MsgJoinPool, MsgSwapExactAmountIn, MsgSwapExactAmountOut,
    QuerySwapExactAmountInRequest, QuerySwapExactAmountInResponse, SwapAmountInRoute,
};

pub const OSMOSIS: &str = "osmosis";
// Source https://github.com/wasmswap/wasmswap-contracts
pub struct Osmosis {
    pub sender_addr: String,
}

/// Osmosis app-chain dex implementation
impl DEX for Osmosis {
    fn over_ibc(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        OSMOSIS
    }

    fn swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let token_out_denom = match ask_asset {
            AssetInfo::Native(denom) => denom,
            _ => return Err(DexError::Cw1155Unsupported),
        };

        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_out_denom,
        }];

        let token_in = Coin::try_from(offer_asset)?;

        let swap_msg: CosmosMsg = MsgSwapExactAmountIn {
            sender: self.sender_addr.clone(),
            routes,
            token_in: Some(token_in.into()),
            token_out_min_amount: Uint128::zero().to_string(),
        }
        .into();

        return Ok(vec![swap_msg]);
    }

    fn custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<Asset>,
        _ask_assets: Vec<Asset>,
        _max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        // The offer_assets have already been sent to the host contract
        // The ask_assets are the assets we want to receive
        // Generate the swap message(s) between the offer and ask assets
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let pool_id = pair_address.to_string().parse::<u64>().unwrap();
        let token_in_maxs: Vec<OsmoCoin> = offer_assets
            .iter()
            .map(|asset| Coin::try_from(asset).unwrap().into())
            .collect();

        // FIXME: THIS FUNCTION IS NOT DONE
        let share_amount_out = compute_osmo_share_out_amount(token_in_maxs, pool_id, deps)?;

        let osmo_msg: CosmosMsg = MsgJoinPool {
            sender: self.sender_addr.clone(),
            pool_id,
            share_out_amount: todo!(), // TODO: Ask osmosis discord for options?
            token_in_maxs,
        }
        .into();

        return Ok(vec![osmo_msg]);
    }

    fn provide_liquidity_symmetric(
        &self,
        _deps: Deps,
        _pair_address: Addr,
        _offer_asset: Asset,
        _paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        Err(DexError::NotImplemented(self.name().to_string()))
    }

    fn withdraw_liquidity(
        &self,
        deps: Deps,
        pair_address: Addr,
        lp_token: Asset,
    ) -> Result<Vec<cosmwasm_std::CosmosMsg>, DexError> {
        let osmo_msg: CosmosMsg = MsgExitPool {
            sender: self.sender_addr.clone(),
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            share_in_amount: lp_token.amount.to_string(),
            token_out_mins: vec![], // TODO: Set zero for all tokens?  Check osmo docs
        }
        .into();

        Ok(vec![osmo_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pair_address: Addr,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let routes: Vec<SwapAmountInRoute> = vec![SwapAmountInRoute {
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_out_denom: match ask_asset {
                AssetInfo::Native(denom) => denom,
                _ => return Err(DexError::Cw1155Unsupported),
            },
        }];

        let token_in = Coin::try_from(offer_asset)?.to_string();

        let sim_msg = QuerySwapExactAmountInRequest {
            sender: self.sender_addr.clone(),
            pool_id: pair_address.to_string().parse::<u64>().unwrap(),
            token_in,
            routes,
        };
        // .into();

        let query_request = QueryRequest::Stargate {
            path: QuerySwapExactAmountInRequest::TYPE_URL.to_string(),
            data: to_binary(&sim_msg)?,
        };
        let res = deps.querier.query(&query_request)?; // Querier is on osmosis!
        let swap_exact_amount_in_response: QuerySwapExactAmountInResponse = from_binary(&res)?;

        return Ok((
            swap_exact_amount_in_response
                .token_out_amount
                .parse::<Uint128>()
                .unwrap(),
            Uint128::zero(),
            Uint128::zero(),
            false,
        ));
    }
}

fn compute_osmo_share_out_amount(
    offer_assets: Vec<OsmoCoin>,
    pool_id: u64,
    deps: Deps,
) -> StdResult<Uint128> {
    let res: QueryPoolResponse = deps
        .querier
        .query(&QueryRequest::Stargate {
            path: QueryPoolRequest::TYPE_URL.to_string(),
            data: to_binary(&QueryPoolRequest { pool_id }).unwrap(),
        })
        .unwrap();

    let pool: Pool = from_binary(&res.pool).unwrap(); // FIXME: find out how to parse this weird type

    let mut share_out_amount = Uint128::zero();
    let Coin {
        denom: token_in1_denom,
        amount: token_in1_amount,
    } = Coin::try_from(offer_assets[0])?;
    let Coin {
        denom: token_in2_denom,
        amount: token_in2_amount,
    } = Coin::try_from(offer_assets[1])?;

    let (idx0, idx1) = (0, 1);
    if (token_in1_denom == offer_assets[0].denom) && (token_in2_denom == offer_assets[1].denom) {
        (idx0, idx1) = (0, 1);
    } else if (token_in1_denom == offer_assets[1].denom)
        && (token_in2_denom == offer_assets[2].denom)
    {
        (idx0, idx1) = (1, 0);
    }

    let price2 = Decimal::from_ratio(token_in1_amount, token_in2_amount);

    // TODO: compute which token is overrepresented in the offer_assets

    // TODO: compute how much of the overrepressentedd token we need to add to the pool

    // TODO: compute how much GAMM tokens well get for that

    return Ok(Uint128::zero());
}
