use cosmwasm_std::Addr;
use cw20_factory_pkg::cw20_indexer::error::{ContractResult, Cw20IndexerError};

pub fn validate_denom(denom: &str, sender: &Addr) -> ContractResult<()> {
    let split: Vec<&str> = denom.split("/").collect();

    if split.len() != 3 {
        return Err(Cw20IndexerError::InvalidDenomFormatLenght { denom: denom.to_string() });
    }

    let first = split[0];
    let second = split[1];

    if second != sender.as_str() || first != "factory" {
        return Err(Cw20IndexerError::InvalidDenomFormatData { denom: denom.to_string() });
    }

    Ok(())
}
