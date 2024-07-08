use crate::state::Agreement;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAgreement {
        id: u64,
    },
    GetTotalAgreementCount {},
    GetInitiatedAgreementCount {},
    GetAcceptedAgreementCount {},
    GetExecutedAgreementCount {},
    GetCanceledAgreementCount {},
    GetAgreementsByInitiator {
        initiator: Addr,
        page: u64,
        page_size: u64,
    },
    GetAgreementsByCounterparty {
        counterparty: Addr,
        page: u64,
        page_size: u64,
    },
    GetAgreementsByStatus {
        status: String,
        page: u64,
        page_size: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenInfo {
    pub address: Addr,
    pub amount: u128,
}

impl TokenInfo {
    #[inline]
    pub fn into_string(self) -> String {
        [self.amount.to_string(), self.address.to_string()].join("")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AgreementResponse {
    pub agreement: Agreement,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TotalAgreementCountResponse {
    pub total_agreement_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InitiatedAgreementCountResponse {
    pub initiated_agreement_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AcceptedAgreementCountResponse {
    pub accepted_agreement_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ExecutedAgreementCountResponse {
    pub executed_agreement_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CanceledAgreementCountResponse {
    pub canceled_agreement_count: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AgreementsResponse {
    pub agreements: Vec<Agreement>,
}
