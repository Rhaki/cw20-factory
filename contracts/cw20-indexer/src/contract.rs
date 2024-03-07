use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw20_factory_pkg::cw20_indexer::{
    error::{ContractResponse, Cw20IndexerError},
    msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
};
use rhaki_cw_plus::{
    storage::interfaces::MapExt,
    traits::{IntoBinaryResult, Wrapper},
};

use crate::{
    function::validate_denom,
    query::{qy_cw20_denom, qy_cw20_token_info},
    state::CW20_MAP,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> ContractResponse {
    Response::new().wrap_ok()
}

#[entry_point]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> ContractResponse {
    match msg {
        ExecuteMsg::RegisterDenom(msg) => {
            validate_denom(&msg.denom, &info.sender)?;
            qy_cw20_token_info(deps.as_ref(), &info.sender)?;
            qy_cw20_denom(deps.as_ref(), &info.sender)?;
            if CW20_MAP.has(deps.storage, &msg.denom) {
                return Err(Cw20IndexerError::DenomAlredySaved { denom: msg.denom });
            };

            CW20_MAP.save(deps.storage, &msg.denom, &info.sender)?;
            Response::new().wrap_ok()
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::TokenInfo { denom } => {
            let cw20_addr = CW20_MAP.better_load(deps.storage, &denom)?;
            qy_cw20_token_info(deps, &cw20_addr).into_binary()
        }
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ContractResponse {
    Response::new().wrap_ok()
}
