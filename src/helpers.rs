use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CwTemplateContract(pub Addr);

#[cfg(test)]
use cosmwasm_std::{Attribute, coin, coins, from_json, OwnedDeps};
#[cfg(test)]
use cosmwasm_std::testing::{message_info, mock_dependencies_with_balances, mock_env, MockApi, MockQuerier, MockStorage};
#[cfg(test)]
use crate::contract::{execute, instantiate, query};
#[cfg(test)]
use crate::contract::STATUS_INITIATED;
#[cfg(test)]
use crate::msg::{AcceptedAgreementCountResponse, CanceledAgreementCountResponse, ExecutedAgreementCountResponse, ExecuteMsg, InitiatedAgreementCountResponse, InstantiateMsg, QueryMsg, TokenInfo, TotalAgreementCountResponse};


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{CosmosMsg, StdResult, WasmMsg, to_json_binary, Coin};
    use crate::msg::ExecuteMsg;

    impl CwTemplateContract {
        pub fn addr(&self) -> Addr {
            self.0.clone()
        }

        pub fn call<T: Into<ExecuteMsg>>(&self, msg: T, funds: Vec<Coin>) -> StdResult<CosmosMsg> {
            let msg = to_json_binary(&msg.into())?;
            Ok(WasmMsg::Execute {
                contract_addr: self.addr().into(),
                msg,
                funds,
            }
                .into())
        }
    }
}

// Helper function to check agreement counts
#[cfg(test)]
pub fn check_agreement_counts(deps: &OwnedDeps<MockStorage, MockApi, MockQuerier>, total: u64, initiated: u64, accepted: u64, executed: u64, canceled: u64) {
    let query_total_msg = QueryMsg::GetTotalAgreementCount;
    let query_total_res = query(deps.as_ref(), mock_env(), query_total_msg).unwrap();
    let total_response: TotalAgreementCountResponse = from_json(&query_total_res).unwrap();
    assert_eq!(total_response.total_agreement_count, total);

    let query_initiated_msg = QueryMsg::GetInitiatedAgreementCount;
    let query_initiated_res = query(deps.as_ref(), mock_env(), query_initiated_msg).unwrap();
    let initiated_response: InitiatedAgreementCountResponse = from_json(&query_initiated_res).unwrap();
    assert_eq!(initiated_response.initiated_agreement_count, initiated);

    let query_accepted_msg = QueryMsg::GetAcceptedAgreementCount;
    let query_accepted_res = query(deps.as_ref(), mock_env(), query_accepted_msg).unwrap();
    let accepted_response: AcceptedAgreementCountResponse = from_json(&query_accepted_res).unwrap();
    assert_eq!(accepted_response.accepted_agreement_count, accepted);

    let query_executed_msg = QueryMsg::GetExecutedAgreementCount;
    let query_executed_res = query(deps.as_ref(), mock_env(), query_executed_msg).unwrap();
    let executed_response: ExecutedAgreementCountResponse = from_json(&query_executed_res).unwrap();
    assert_eq!(executed_response.executed_agreement_count, executed);

    let query_canceled_msg = QueryMsg::GetCanceledAgreementCount;
    let query_canceled_res = query(deps.as_ref(), mock_env(), query_canceled_msg).unwrap();
    let canceled_response: CanceledAgreementCountResponse = from_json(&query_canceled_res).unwrap();
    assert_eq!(canceled_response.canceled_agreement_count, canceled);
}

// Helper function to initialize the contract
#[cfg(test)]
pub fn initialize_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies_with_balances(&[
        ((&Addr::unchecked("initiator")).as_ref(), &[coin(1000, "tokenA")]),
        ((&Addr::unchecked("counterparty")).as_ref(), &[coin(2000, "tokenB")]),
        ((&Addr::unchecked("cosmos2contract")).as_ref(), &[coin(1000, "tokenA"), coin(2000, "tokenB")]),
    ]);

    let msg = InstantiateMsg {};
    let info = message_info(&Addr::unchecked("creator"), &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, _res.messages.len());
    deps
}

// Helper function to initiate an agreement
#[cfg(test)]
pub fn initiate_new_agreement(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>, initiator: &str, initiator_amount: u128, counterparty: &str, counterparty_amount: u128) -> (TokenInfo, TokenInfo, Addr) {
    let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: initiator_amount };
    let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: counterparty_amount };
    let counterparty_addr = Addr::unchecked(counterparty);

    let msg = ExecuteMsg::InitiateAgreement {
        initiator_token: initiator_token.clone(),
        counterparty_token: counterparty_token.clone(),
        counterparty: counterparty_addr.clone(),
    };
    let info = message_info(&Addr::unchecked(initiator), &coins(initiator_amount, "tokenA"));
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    assert_eq!(res.attributes, vec![
        Attribute { key: "method".to_string(), value: "initiate_agreement".to_string() },
        Attribute { key: "id".to_string(), value: "1".to_string() },
        Attribute { key: "status".to_string(), value: STATUS_INITIATED.to_string() },
        Attribute { key: "initiator".to_string(), value: initiator.to_string() },
        Attribute { key: "counterparty".to_string(), value: counterparty.to_string() },
        Attribute { key: "initiator_token".to_string(), value: initiator_token.clone().into_string() },
        Attribute { key: "counterparty_token".to_string(), value: counterparty_token.clone().into_string() }
    ]);

    // Check agreement status counts
    check_agreement_counts(&deps, 1, 1, 0, 0, 0);

    (initiator_token, counterparty_token, counterparty_addr)
}
