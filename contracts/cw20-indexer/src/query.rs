use cosmwasm_std::{Addr, Deps, StdResult};
use cw20::TokenInfoResponse;

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
