use crate::msg::{AgreementResponse, AgreementsResponse, TokenInfo};
use cosmwasm_std::{Addr, Deps, StdResult};
use cw_storage_plus::{Item, Map};
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

pub const AGREEMENTS: Map<u64, Agreement> = Map::new("agreements");
pub const AGREEMENT_COUNT: Item<u64> = Item::new("agreement_count");

pub fn query_agreement(deps: Deps, id: u64) -> StdResult<AgreementResponse> {
    let agreement = AGREEMENTS.load(deps.storage, id)?;
    Ok(AgreementResponse { agreement })
}

pub fn query_agreements_by_initiator(deps: Deps, initiator: Addr) -> StdResult<AgreementsResponse> {
    let agreements: Vec<Agreement> = AGREEMENTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, agreement)) if agreement.initiator == initiator => Some(agreement),
            _ => None,
        })
        .collect();

    Ok(AgreementsResponse { agreements })
}

pub fn query_agreements_by_counterparty(deps: Deps, counterparty: Addr) -> StdResult<AgreementsResponse> {
    let agreements: Vec<Agreement> = AGREEMENTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, agreement)) => {
                if agreement.counterparty == counterparty {
                    Some(agreement)
                } else {
                    None
                }
            },
            Err(_) => None,
        })
        .collect();

    Ok(AgreementsResponse { agreements })
}
