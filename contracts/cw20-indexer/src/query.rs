use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw20::TokenInfoResponse;
use cw20_factory_pkg::{
    cw20_factory::msgs::SupplyDetailsResponse, cw20_indexer::msgs::TokenDetails,
};
use rhaki_cw_plus::{storage::StorageOrder, traits::Wrapper};

use crate::state::CW20_MAP;

pub fn qy_cw20_token_info(deps: Deps, cw20_addr: &Addr) -> StdResult<TokenInfoResponse> {
    deps.querier
        .query_wasm_smart(cw20_addr, &cw20::Cw20QueryMsg::TokenInfo {})
}

pub fn qy_cw20_denom(deps: Deps, cw20_addr: &Addr) -> StdResult<String> {
    deps.querier.query_wasm_smart(
        cw20_addr,
        &cw20_factory_pkg::cw20_factory::msgs::QueryMsg::FactoryDenom {},
    )
}

pub fn qy_tokens_info(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order: Option<StorageOrder>,
) -> StdResult<Vec<TokenDetails>> {
    let order: Order = order.unwrap_or(StorageOrder::Descending).into();
    rhaki_cw_plus::storage::map::get_items(deps.storage, &CW20_MAP, order, limit, start_after)?
        .into_iter()
        .map(|(k, v)| qy_token_info(deps, k, v.wrap_some()))
        .collect()
}

pub fn qy_token_info(
    deps: Deps,
    denom: String,
    cw20_addr: Option<Addr>,
) -> StdResult<TokenDetails> {
    let cw20_addr = cw20_addr
        .map(|val| val.wrap_ok())
        .unwrap_or_else(|| CW20_MAP.load(deps.storage, denom.clone()))?;
    let info = qy_cw20_token_info(deps, &cw20_addr)?;

    let supply: SupplyDetailsResponse = deps.querier.query_wasm_smart(
        &cw20_addr,
        &cw20_factory_pkg::cw20_factory::msgs::QueryMsg::SupplyDetails {},
    )?;

    TokenDetails {
        cw20_addr: cw20_addr.to_string(),
        native_denom: denom,
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: supply.total_supply,
        cw20_supply: supply.cw20_supply,
        native_supply: supply.native_supply,
    }
    .wrap_ok()
}
