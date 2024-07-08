#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, coin, to_json_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use crate::state::{Agreement, AGREEMENTS, AGREEMENT_COUNT, query_agreement, query_agreements_by_initiator, query_agreements_by_counterparty};
use crate::utils::{assert_agreement_has_status, assert_contract_has_sufficient_funds, assert_funds_match_token_amount, assert_sender_authorized, assert_sender_matches_counterparty};

const CONTRACT_NAME: &str = "crates.io:native-token-exchange-escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const STATUS_INITIATED: &str = "initiated";
pub const STATUS_ACCEPTED: &str = "accepted";
pub const STATUS_EXECUTED: &str = "executed";
pub const STATUS_CANCELED: &str = "canceled";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    AGREEMENT_COUNT.save(deps.storage, &0)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitiateAgreement { initiator_token, counterparty_token, counterparty } => {
            initiate_agreement(deps, info, initiator_token, counterparty_token, counterparty)
        }
        ExecuteMsg::AcceptAgreement { id } => accept_agreement(deps, info, id),
        ExecuteMsg::ExecuteAgreement { id } => execute_agreement(deps, env, info, id),
        ExecuteMsg::CancelAgreement { id } => cancel_agreement(deps, info, id),
    }
}

fn initiate_agreement(
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
        status: STATUS_INITIATED.to_string(),
    };

    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "initiate_agreement")
        .add_attribute("id", id.to_string()))
}

fn accept_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    assert_sender_matches_counterparty(&info.sender, &agreement.counterparty)?;
    assert_funds_match_token_amount(&info.funds, &agreement.counterparty_token)?;
    assert_agreement_has_status(&agreement.status, &[STATUS_INITIATED])?;

    agreement.status = STATUS_ACCEPTED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "accept_agreement")
        .add_attribute("id", id.to_string()))
}

fn execute_agreement(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    assert_sender_authorized(&info.sender, &[&agreement.initiator, &agreement.counterparty])?;
    assert_agreement_has_status(&agreement.status, &[STATUS_ACCEPTED])?;
    assert_contract_has_sufficient_funds(&deps, &env, &agreement.initiator_token)?;
    assert_contract_has_sufficient_funds(&deps, &env, &agreement.counterparty_token)?;

    let messages = vec![
        BankMsg::Send {
            to_address: agreement.counterparty.to_string(),
            amount: vec![coin(agreement.initiator_token.amount, &agreement.initiator_token.address)],
        },
        BankMsg::Send {
            to_address: agreement.initiator.to_string(),
            amount: vec![coin(agreement.counterparty_token.amount, &agreement.counterparty_token.address)],
        },
    ];

    agreement.status = STATUS_EXECUTED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "execute_agreement")
        .add_attribute("id", id.to_string()))
}

fn cancel_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    assert_sender_authorized(&info.sender, &[&agreement.initiator, &agreement.counterparty])?;
    assert_agreement_has_status(&agreement.status, &[STATUS_INITIATED, STATUS_ACCEPTED])?;

    let messages = vec![
        BankMsg::Send {
            to_address: agreement.initiator.to_string(),
            amount: vec![coin(agreement.initiator_token.amount, &agreement.initiator_token.address)],
        },
        BankMsg::Send {
            to_address: agreement.counterparty.to_string(),
            amount: vec![coin(agreement.counterparty_token.amount, &agreement.counterparty_token.address)],
        },
    ];

    agreement.status = STATUS_CANCELED.to_string();
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
