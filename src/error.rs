use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized: expected `{expected}`, but found `{found}`")]
    Unauthorized { expected: String, found: String },

    #[error("Insufficient funds sent")]
    InsufficientFunds,

    #[error("Incorrect funds amount sent: expected `{expected}`, but found `{found}`")]
    IncorrectFundsAmount { expected: String, found: String },

    #[error("Invalid agreement status: expected `{expected}`, but found `{found}`")]
    InvalidAgreementStatus { expected: String, found: String },

    #[error("Unexpected funds found: expected `{expected}`, but found `{found}`")]
    UnexpectedFunds { expected: String, found: String },

    #[error("Insufficient contract funds: expected `{expected}`, but found `{found}`")]
    InsufficientContractFunds { expected: String, found: String },

    #[error("Invalid counterparty: counterparty `{counterparty}` cannot be the same as initiator `{initiator}`")]
    InvalidCounterparty {
        initiator: String,
        counterparty: String,
    },
}
