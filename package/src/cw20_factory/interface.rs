use cosmwasm_std::{
    Addr, Attribute, Coin, CosmosMsg, CustomQuery, DepsMut, Empty, Env, MessageInfo,
};

use crate::cw20_factory::ContractResult;

pub struct InterfaceInstantiateResponse<CM> {
    pub attributes: Vec<Attribute>,
    pub messages: Vec<CosmosMsg<CM>>,
    pub factory_denom: String,
}

pub trait TokenFactoryInterface<CQ: CustomQuery = Empty, CM = Empty> {
    fn instantiate(
        deps: DepsMut<CQ>,
        env: &Env,
        info: MessageInfo,
        symbol: String,
    ) -> ContractResult<InterfaceInstantiateResponse<CM>>;

    fn burn(
        deps: DepsMut<CQ>,
        env: &Env,
        info: &MessageInfo,
        amount: &Coin,
    ) -> ContractResult<Vec<CosmosMsg<CM>>>;

    fn mint(
        deps: DepsMut<CQ>,
        env: &Env,
        info: &MessageInfo,
        to: &Addr,
        amount: &Coin,
    ) -> ContractResult<Vec<CosmosMsg<CM>>>;
}
