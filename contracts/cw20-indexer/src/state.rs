use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const CW20_MAP: Map<&str, Addr> = Map::new("cw20_map");
