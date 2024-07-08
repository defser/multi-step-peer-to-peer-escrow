use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::Agreement;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum ExecuteMsg {
    InitiateAgreement {
        initiator_token: TokenInfo,
        counterparty_token: TokenInfo,
        counterparty: Addr,
    },
    AcceptAgreement {
        id: u64,
    },
    ExecuteAgreement {
        id: u64,
    },
    CancelAgreement {
        id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum QueryMsg {
    GetAgreement { id: u64 },
    GetAgreementsByInitiator { initiator: Addr },
    GetAgreementsByCounterparty { counterparty: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenInfo {
    pub address: Addr,
    pub amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AgreementResponse {
    pub agreement: Agreement,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AgreementsResponse {
    pub agreements: Vec<Agreement>,
}
