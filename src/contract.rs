#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, coin, to_json_binary};
use cw2::{set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use crate::state::{Agreement, AGREEMENTS, AGREEMENT_COUNT, query_agreement, query_agreements_by_initiator, query_agreements_by_counterparty};
use crate::utils::{assert_agreement_has_status, assert_contract_has_sufficient_funds, assert_funds_match_token_amount, assert_sender_authorized, assert_sender_match_counterparty};

const CONTRACT_NAME: &str = "crates.io:native-token-exchange-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INITIATED: &str = "initiated";
const ACCEPTED: &str = "accepted";
const EXECUTED: &str = "executed";
const CANCELED: &str = "canceled";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    AGREEMENT_COUNT.save(deps.storage, &0)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::InitiateAgreement { initiator_token, counterparty_token, counterparty } => {
            try_initiate_agreement(deps, _info, initiator_token, counterparty_token, counterparty)
        }
        ExecuteMsg::AcceptAgreement { id } => try_accept_agreement(deps, _info, id),
        ExecuteMsg::ExecuteAgreement { id } => try_execute_agreement(deps, _env, _info, id),
        ExecuteMsg::CancelAgreement { id } => try_cancel_agreement(deps, _info, id),
    }
}

fn try_initiate_agreement(
    deps: DepsMut,
    info: MessageInfo,
    initiator_token: TokenInfo,
    counterparty_token: TokenInfo,
    counterparty: Addr,
) -> Result<Response, ContractError> {
    assert_funds_match_token_amount(&info.funds, &initiator_token)?;

    let id = AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    let agreement = Agreement {
        id,
        initiator: info.sender.clone(),
        counterparty,
        initiator_token,
        counterparty_token,
        status: INITIATED.to_string(),
    };

    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "initiate_agreement")
        .add_attribute("id", id.to_string()))
}

fn try_accept_agreement(
    deps: DepsMut,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    assert_sender_match_counterparty(&_info.sender, &agreement.counterparty)?;

    assert_funds_match_token_amount(&_info.funds, &agreement.counterparty_token)?;

    assert_agreement_has_status(&agreement.status, &[INITIATED])?;

    agreement.status = ACCEPTED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "accept_agreement")
        .add_attribute("id", id.to_string()))
}

fn try_execute_agreement(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    assert_sender_authorized(&_info.sender, &[&agreement.initiator, &agreement.counterparty])?;
    assert_agreement_has_status(&agreement.status, &[ACCEPTED])?;
    assert_contract_has_sufficient_funds(&deps, &_env, &agreement.initiator_token)?;
    assert_contract_has_sufficient_funds(&deps, &_env, &agreement.counterparty_token)?;

    let mut messages = vec![];

    messages.push(BankMsg::Send {
        to_address: agreement.counterparty.to_string(),
        amount: vec![coin(agreement.initiator_token.amount, &agreement.initiator_token.address)],
    });

    messages.push(BankMsg::Send {
        to_address: agreement.initiator.to_string(),
        amount: vec![coin(agreement.counterparty_token.amount, &agreement.counterparty_token.address)],
    });

    agreement.status = EXECUTED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "execute_agreement")
        .add_attribute("id", id.to_string()))
}

fn try_cancel_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    assert_sender_authorized(&info.sender, &[&agreement.initiator, &agreement.counterparty])?;

    assert_agreement_has_status(&agreement.status, &[ACCEPTED, INITIATED])?;

    agreement.status = CANCELED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "cancel_agreement")
        .add_attribute("id", id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAgreement { id } => to_json_binary(&query_agreement(deps, id)?),
        QueryMsg::GetAgreementsByInitiator { initiator } => to_json_binary(&query_agreements_by_initiator(deps, initiator)?),
        QueryMsg::GetAgreementsByCounterparty { counterparty } => to_json_binary(&query_agreements_by_counterparty(deps, counterparty)?),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, message_info, mock_dependencies_with_balances};
    use cosmwasm_std::{coin, coins, from_json};
    use crate::msg::{AgreementResponse, AgreementsResponse};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreement { id: 1 });
        assert!(res.is_err());
    }

    #[test]
    fn initiate_and_accept_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "initiate_agreement"), ("id", "1")]);

        let msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let info = message_info(&Addr::unchecked("counterparty"), &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "accept_agreement"), ("id", "1")]);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreement { id: 1 }).unwrap();
        let value: AgreementResponse = from_json(&res).unwrap();
        assert_eq!(value.agreement.id, 1);
        assert_eq!(value.agreement.initiator, Addr::unchecked("initiator"));
        assert_eq!(value.agreement.counterparty, Addr::unchecked("counterparty"));
        assert_eq!(value.agreement.initiator_token, initiator_token);
        assert_eq!(value.agreement.counterparty_token, counterparty_token);
        assert_eq!(value.agreement.status, ACCEPTED);
    }

    #[test]
    fn insufficient_funds_initiate_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(500, "tokenA")); // Insufficient funds
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::IncorrectFundsAmount { expected, found } => {
                assert_eq!(expected, "1000");
                assert_eq!(found, "500");
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn execute_agreement_success() {
        // Arrange
        let mut deps = mock_dependencies_with_balances(&[
            ((&Addr::unchecked("initiator")).as_ref(), &[coin(1000, "tokenA")]),
            ((&Addr::unchecked("counterparty")).as_ref(), &[coin(2000, "tokenB")]),
            ((&Addr::unchecked("cosmos2contract")).as_ref(), &[coin(1000, "tokenA"), coin(2000, "tokenB")]),
        ]);

        // Initialize the contract
        let init_msg = InstantiateMsg {};
        let init_info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), init_info.clone(), init_msg).unwrap();

        // Initiate an agreement
        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let initiate_msg = ExecuteMsg::InitiateAgreement {
            initiator_token: initiator_token.clone(),
            counterparty_token: counterparty_token.clone(),
            counterparty: counterparty.clone(),
        };
        let initiate_info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), initiate_info.clone(), initiate_msg).unwrap();

        // Accept the agreement
        let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let accept_info = message_info(&Addr::unchecked("counterparty"), &coins(2000, "tokenB"));
        let _res = execute(deps.as_mut(), mock_env(), accept_info.clone(), accept_msg).unwrap();

        // Execute the agreement
        let execute_msg = ExecuteMsg::ExecuteAgreement { id: 1 };
        let execute_info = message_info(&Addr::unchecked("initiator"), &[]);
        let res = execute(deps.as_mut(), mock_env(), execute_info.clone(), execute_msg).unwrap();

        // Check the response
        assert_eq!(res.messages.len(), 2);

        // Check if the agreement status is executed
        let query_msg = QueryMsg::GetAgreement { id: 1 };
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let agreement_response: AgreementResponse = from_json(&query_res).unwrap();

        assert_eq!(agreement_response.agreement.status, EXECUTED);
    }

    #[test]
    fn cancel_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128,};
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128, };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement {
            initiator_token: initiator_token.clone(),
            counterparty_token: counterparty_token.clone(),
            counterparty: counterparty.clone(),
        };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "initiate_agreement"), ("id", "1")]);

        let msg = ExecuteMsg::CancelAgreement { id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "cancel_agreement"), ("id", "1")]);

        let query_msg = QueryMsg::GetAgreement { id: 1 };
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let agreement_response: AgreementResponse = from_json(&query_res).unwrap();

        assert_eq!(agreement_response.agreement.status, CANCELED);
    }


    #[test]
    fn accept_cancelled_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CancelAgreement { id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "cancel_agreement"), ("id", "1")]);

        let msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let info = message_info(&counterparty, &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::InvalidAgreementStatus { expected, found } => {
                assert_eq!(expected, format!("{}", INITIATED));
                assert_eq!(found, CANCELED);
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn query_agreements_by_initiator() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty2") };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreementsByInitiator { initiator: Addr::unchecked("initiator") }).unwrap();
        let value: AgreementsResponse = from_json(&res).unwrap();
        assert_eq!(value.agreements.len(), 2);
    }

    #[test]
    fn query_agreements_by_counterparty() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty") };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty2") };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreementsByCounterparty { counterparty: Addr::unchecked("counterparty") }).unwrap();
        let value: AgreementsResponse = from_json(&res).unwrap();
        assert_eq!(value.agreements.len(), 1);
    }
}
