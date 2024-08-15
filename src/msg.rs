use crate::state::Agreement;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AgreementResponse)]
    GetAgreement { id: u64 },
    #[returns(TotalAgreementCountResponse)]
    GetTotalAgreementCount {},
    #[returns(InitiatedAgreementCountResponse)]
    GetInitiatedAgreementCount {},
    #[returns(AcceptedAgreementCountResponse)]
    GetAcceptedAgreementCount {},
    #[returns(ExecutedAgreementCountResponse)]
    GetExecutedAgreementCount {},
    #[returns(CanceledAgreementCountResponse)]
    GetCanceledAgreementCount {},
    #[returns(AgreementsResponse)]
    GetAgreementsByInitiator {
        initiator: Addr,
        page: u64,
        page_size: u64,
    },
    #[returns(AgreementsResponse)]
    GetAgreementsByCounterparty {
        counterparty: Addr,
        page: u64,
        page_size: u64,
    },
    #[returns(AgreementsResponse)]
    GetAgreementsByStatus {
        status: String,
        page: u64,
        page_size: u64,
    },
}

#[cw_serde]
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

#[cw_serde]
pub struct AgreementResponse {
    pub agreement: Agreement,
}

#[cw_serde]
pub struct TotalAgreementCountResponse {
    pub total_agreement_count: u64,
}

#[cw_serde]
pub struct InitiatedAgreementCountResponse {
    pub initiated_agreement_count: u64,
}

#[cw_serde]
pub struct AcceptedAgreementCountResponse {
    pub accepted_agreement_count: u64,
}

#[cw_serde]
pub struct ExecutedAgreementCountResponse {
    pub executed_agreement_count: u64,
}

#[cw_serde]
pub struct CanceledAgreementCountResponse {
    pub canceled_agreement_count: u64,
}

#[cw_serde]
pub struct AgreementsResponse {
    pub agreements: Vec<Agreement>,
}
