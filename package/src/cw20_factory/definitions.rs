use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum TransmuteIntoMsg {
    Cw20 {},
    Native { amount: Uint128 },
}
