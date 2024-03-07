use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub enum TransmuteInto {
    Cw20 { amount: Uint128 },
    Native {},
}
