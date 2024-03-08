use cosmwasm_schema::cw_serde;

use rhaki_cw_plus::storage::interfaces::ItemInterface;

#[cw_serde]
pub struct FactoryDenom(String);

impl FactoryDenom {
    pub fn new(denom: String) -> Self {
        FactoryDenom(denom)
    }
    pub fn inner(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for FactoryDenom {
    fn from(val: String) -> Self {
        FactoryDenom(val)
    }
}

impl ItemInterface for FactoryDenom {
    const NAMESPACE: &'static str = "factory_denom";
    const CONTRACT_NAME: &'static str = "cw20_factory";
}
