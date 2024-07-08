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

pub const INITIATED: &str = "initiated";
pub const ACCEPTED: &str = "accepted";
pub const EXECUTED: &str = "executed";
pub const CANCELED: &str = "canceled";

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

    let mut messages = vec![];

    messages.push(BankMsg::Send {
        to_address: agreement.initiator.to_string(),
        amount: vec![coin(agreement.initiator_token.amount, &agreement.initiator_token.address)],
    });

    messages.push(BankMsg::Send {
        to_address: agreement.counterparty.to_string(),
        amount: vec![coin(agreement.counterparty_token.amount, &agreement.counterparty_token.address)],
    });

    agreement.status = CANCELED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_messages(messages)
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
