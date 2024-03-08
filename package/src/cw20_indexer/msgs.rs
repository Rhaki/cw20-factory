use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use rhaki_cw_plus::storage::StorageOrder;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    RegisterDenom(RegisterDenomMsg),
}

#[cw_serde]
pub enum QueryMsg {
    TokenInfo {
        denom: String,
    },
    TokensInfo {
        start_after: Option<String>,
        limit: Option<u32>,
        order: Option<StorageOrder>,
    },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct RegisterDenomMsg {
    pub denom: String,
}

#[cw_serde]
pub struct TokenDetails {
    pub cw20_addr: String,
    pub native_denom: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: Uint128,
    pub cw20_supply: Uint128,
    pub native_supply: Uint128,
}
