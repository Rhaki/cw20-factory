use cosmwasm_std::{Response, StdError};
use thiserror::Error;

pub type ContractResponse = Result<Response, Cw20IndexerError>;
pub type ContractResult<T> = Result<T, Cw20IndexerError>;

#[derive(Error, Debug, PartialEq)]
pub enum Cw20IndexerError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid denom lenght: denom {denom} is not format factory/contract/subdenom")]
    InvalidDenomFormatLenght { denom: String },

    #[error("Invalid denom data: denom {denom} is not format factory/contract/subdenom")]
    InvalidDenomFormatData { denom: String },

    #[error("Denom already saved: {denom}")]
    DenomAlredySaved { denom: String },
}
