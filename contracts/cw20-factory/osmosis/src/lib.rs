use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, StdResult};
use cw20_factory_base::contract::Cw20FactoryBase;
use cw20_factory_pkg::cw20_factory::{
    msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    ContractResponse,
};
use interface::OsmosisTokenFactoryInterface;

pub mod interface;

pub type Cw20FactoryOsmosis = Cw20FactoryBase<Empty, OsmosisTokenFactoryInterface, Empty>;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ContractResponse<Empty> {
    Cw20FactoryOsmosis::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResponse<Empty> {
    Cw20FactoryOsmosis::execute(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Cw20FactoryOsmosis::query(deps, env, msg)
}

#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> ContractResponse<Empty> {
    Cw20FactoryOsmosis::migrate(deps, env, msg)
}
