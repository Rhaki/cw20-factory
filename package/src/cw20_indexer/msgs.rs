use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterDenom(RegisterDenomMsg),
}

#[cw_serde]
pub enum QueryMsg {
    TokenInfo { denom: String },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct RegisterDenomMsg {
    pub denom: String,
}
