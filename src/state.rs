use crate::msg::{TotalAgreementCountResponse, AgreementResponse, AgreementsResponse, TokenInfo, InitiatedAgreementCountResponse, AcceptedAgreementCountResponse, ExecutedAgreementCountResponse, CanceledAgreementCountResponse};
use cosmwasm_std::{Addr, Deps, Order, StdResult};
use cw_storage_plus::{Bounder, Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Agreement {
    pub id: u64,
    pub initiator: Addr,
    pub initiator_token: TokenInfo,
    pub counterparty: Addr,
    pub counterparty_token: TokenInfo,
    pub status: String,
}

// Storage for agreements and agreement count
pub const AGREEMENTS: Map<u64, Agreement> = Map::new("agreements");
pub const TOTAL_AGREEMENT_COUNT: Item<u64> = Item::new("total_agreement_count");
pub const INITIATED_AGREEMENT_COUNT: Item<u64> = Item::new("initiated_agreement_count");
pub const ACCEPTED_AGREEMENT_COUNT: Item<u64> = Item::new("accepted_agreement_count");
pub const EXECUTED_AGREEMENT_COUNT: Item<u64> = Item::new("executed_agreement_count");
pub const CANCELED_AGREEMENT_COUNT: Item<u64> = Item::new("canceled_agreement_count");

// Query functions for agreements

/// Queries a specific agreement by its ID.
pub fn query_agreement(deps: Deps, id: u64) -> StdResult<AgreementResponse> {
    let agreement = AGREEMENTS.load(deps.storage, id)?;
    Ok(AgreementResponse { agreement })
}

/// Queries total agreement count.
pub fn query_total_agreement_count(deps: Deps) -> StdResult<TotalAgreementCountResponse> {
    let total_agreement_count = TOTAL_AGREEMENT_COUNT.load(deps.storage)?;
    Ok(TotalAgreementCountResponse { total_agreement_count })
}

/// Queries initiated agreement count.
pub fn query_initiated_agreement_count(deps: Deps) -> StdResult<InitiatedAgreementCountResponse> {
    let initiated_agreement_count = INITIATED_AGREEMENT_COUNT.load(deps.storage)?;
    Ok(InitiatedAgreementCountResponse { initiated_agreement_count })
}

/// Queries accepted agreement count.
pub fn query_accepted_agreement_count(deps: Deps) -> StdResult<AcceptedAgreementCountResponse> {
    let accepted_agreement_count = ACCEPTED_AGREEMENT_COUNT.load(deps.storage)?;
    Ok(AcceptedAgreementCountResponse { accepted_agreement_count })
}

/// Queries executed agreement count.
pub fn query_executed_agreement_count(deps: Deps) -> StdResult<ExecutedAgreementCountResponse> {
    let executed_agreement_count = EXECUTED_AGREEMENT_COUNT.load(deps.storage)?;
    Ok(ExecutedAgreementCountResponse { executed_agreement_count })
}

/// Queries canceled agreement count.
pub fn query_canceled_agreement_count(deps: Deps) -> StdResult<CanceledAgreementCountResponse> {
    let canceled_agreement_count = CANCELED_AGREEMENT_COUNT.load(deps.storage)?;
    Ok(CanceledAgreementCountResponse { canceled_agreement_count })
}

/// Queries agreements initiated by a specific address within a given range.
pub fn query_agreements_by_initiator(
    deps: Deps,
    initiator: Addr,
    start_after: u64,
    end_before: u64,
) -> StdResult<AgreementsResponse> {
    let agreements: Vec<Agreement> = AGREEMENTS
        .range(deps.storage, start_after.inclusive_bound(), end_before.inclusive_bound(), Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, agreement)) if agreement.initiator == initiator => Some(agreement),
            _ => None,
        })
        .collect();

    Ok(AgreementsResponse { agreements })
}

/// Queries agreements involving a specific counterparty address within a given range.
pub fn query_agreements_by_counterparty(
    deps: Deps,
    counterparty: Addr,
    start_after: u64,
    end_before: u64,
) -> StdResult<AgreementsResponse> {
    let agreements: Vec<Agreement> = AGREEMENTS
        .range(deps.storage, start_after.inclusive_bound(), end_before.inclusive_bound(), Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, agreement)) if agreement.counterparty == counterparty => Some(agreement),
            _ => None,
        })
        .collect();

    Ok(AgreementsResponse { agreements })
}

/// Queries agreements with a specific status within a given range.
pub fn query_agreements_by_status(
    deps: Deps,
    status: String,
    start_after: u64,
    end_before: u64,
) -> StdResult<AgreementsResponse> {
    let agreements: Vec<Agreement> = AGREEMENTS
        .range(deps.storage, start_after.inclusive_bound(), end_before.inclusive_bound(), Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, agreement)) if agreement.status == status => Some(agreement),
            _ => None,
        })
        .collect();

    Ok(AgreementsResponse { agreements })
}
