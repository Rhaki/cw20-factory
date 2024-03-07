pub mod definitions;
mod error;
pub mod interface;
pub mod msgs;

pub mod traits;
pub use error::{ContractResponse, ContractResult, Cw20FactoryError};

#[cfg(test)]
mod test {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{from_json, to_json_binary};

    #[cw_serde]
    pub struct Lesser {
        pub a: String,
        pub b: String,
    }

    #[cw_serde]
    pub struct Greater {
        pub a: String,
        pub b: String,
        pub c: String,
    }

    #[test]
    fn t_1() {
        let greater = Greater {
            a: "a".to_string(),
            b: "b".to_string(),
            c: "c".to_string(),
        };

        from_json::<Lesser>(to_json_binary(&greater).unwrap()).unwrap_err();
    }
}
