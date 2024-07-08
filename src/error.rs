use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized: expecting `{expected}`, found `{found}`")]
    Unauthorized { expected: String, found: String },

    #[error("Insufficient funds sent")]
    InsufficientFunds { },

    #[error("Incorrect funds amount sent: expecting `{expected}`, found `{found}`")]
    IncorrectFundsAmount { expected: String, found: String },

    #[error("Invalid Agreement Status: expecting `{expected}`, found `{found}`")]
    InvalidAgreementStatus { expected: String, found: String },

    #[error("Unexpected funds found: expecting `{expected}`, found `{found}`")]
    UnexpectedFunds { expected: String, found: String },

    #[error("Insufficient contract funds: expecting `{expected}`, found `{found}`")]
    InsufficientContractFunds { expected: String, found: String },

    #[error("Invalid counterparty: counterparty `{counterparty}` cannot be the same as initiator `{initiator}`")]
    InvalidCounterparty { initiator: String, counterparty: String },
}
