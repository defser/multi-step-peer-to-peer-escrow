use crate::msg::TokenInfo;
use cosmwasm_std::Addr;
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
