use cosmwasm_std::{Response, StdError, Uint128};
use cw20_base::ContractError as Cw20BaseError;
use thiserror::Error;

pub type ContractResponse<T> = Result<Response<T>, Cw20FactoryError>;
pub type ContractResult<T> = Result<T, Cw20FactoryError>;

#[derive(Error, Debug, PartialEq)]
pub enum Cw20FactoryError {
    #[error("{0}")]
    Base(#[from] Cw20BaseError),

    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InvalidDenom: expected {expected}, received {received}")]
    InvalidDenom { expected: String, received: String },

    #[error("Insufficient cw20 balance: current: {current}, requested: {requested}")]
    InsufficientCw20Balance {
        current: Uint128,
        requested: Uint128,
    },
}
