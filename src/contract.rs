#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, BankMsg, coin, to_json_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use crate::state::{Agreement, AGREEMENTS, TOTAL_AGREEMENT_COUNT, query_agreement, query_agreements_by_initiator, query_agreements_by_counterparty, query_agreements_by_status, query_total_agreement_count, INITIATED_AGREEMENT_COUNT, ACCEPTED_AGREEMENT_COUNT, EXECUTED_AGREEMENT_COUNT, CANCELED_AGREEMENT_COUNT, query_initiated_agreement_count, query_accepted_agreement_count, query_executed_agreement_count, query_canceled_agreement_count};
use crate::utils::{assert_agreement_has_status, assert_contract_has_sufficient_funds, assert_funds_match_token_amount, assert_sender_authorized, assert_sender_is_different_from_counterparty, assert_sender_matches_counterparty};

const CONTRACT_NAME: &str = "crates.io:peer-to-peer-token-swap";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const STATUS_INITIATED: &str = "initiated";
pub const STATUS_ACCEPTED: &str = "accepted";
pub const STATUS_EXECUTED: &str = "executed";
pub const STATUS_CANCELED: &str = "canceled";

/// Handles contract instantiation, initializing necessary storage.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Set contract version and initialize agreement count
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    TOTAL_AGREEMENT_COUNT.save(deps.storage, &0)?;
    INITIATED_AGREEMENT_COUNT.save(deps.storage, &0)?;
    ACCEPTED_AGREEMENT_COUNT.save(deps.storage, &0)?;
    EXECUTED_AGREEMENT_COUNT.save(deps.storage, &0)?;
    CANCELED_AGREEMENT_COUNT.save(deps.storage, &0)?;

    // Return success response with attributes
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

/// Executes contract functions based on incoming messages.
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
        ExecuteMsg::CancelAgreement { id } => cancel_agreement(deps, env, info, id),
    }
}

/// Initiates a new agreement between initiator and counterparty.
fn initiate_agreement(
    deps: DepsMut,
    info: MessageInfo,
    initiator_token: TokenInfo,
    counterparty_token: TokenInfo,
    counterparty: Addr,
) -> Result<Response, ContractError> {
    // Verify that initiator's funds match the token amount provided
    assert_funds_match_token_amount(&info.funds, &initiator_token)?;

    // Ensure initiator is different from counterparty
    assert_sender_is_different_from_counterparty(&info.sender, &counterparty)?;

    // Generate new agreement ID and update agreement status counts
    let id = TOTAL_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;
    INITIATED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    // Create agreement struct and save to storage
    let agreement = Agreement {
        id,
        initiator: info.sender.clone(),
        counterparty: counterparty.clone(),
        initiator_token: initiator_token.clone(),
        counterparty_token: counterparty_token.clone(),
        status: STATUS_INITIATED.to_string(),
    };
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    // Return success response with attributes
    Ok(Response::new()
        .add_attribute("method", "initiate_agreement")
        .add_attribute("id", id.to_string())
        .add_attribute("status", agreement.status.to_string())
        .add_attribute("initiator", info.sender.clone())
        .add_attribute("counterparty", counterparty.clone())
        .add_attribute("initiator_token", initiator_token.into_string())
        .add_attribute("counterparty_token", counterparty_token.into_string()))
}

/// Accepts an agreement by its ID, progressing its status to accepted.
fn accept_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    // Load agreement from storage by ID
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    // Verify sender matches the counterparty of the agreement
    assert_sender_matches_counterparty(&info.sender, &agreement.counterparty)?;

    // Verify sender's funds match the counterparty's token amount
    assert_funds_match_token_amount(&info.funds, &agreement.counterparty_token)?;

    // Assert agreement status is INITIATED before accepting
    assert_agreement_has_status(&agreement.status, &[STATUS_INITIATED])?;

    // Update agreement status counts
    INITIATED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count - 1) })?;
    ACCEPTED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    // Update agreement status to ACCEPTED and save back to storage
    agreement.status = STATUS_ACCEPTED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    // Return success response with attributes
    Ok(Response::new()
        .add_attribute("method", "accept_agreement")
        .add_attribute("id", id.to_string())
        .add_attribute("status", agreement.status.to_string())
        .add_attribute("initiator", agreement.initiator)
        .add_attribute("counterparty", agreement.counterparty)
        .add_attribute("initiator_token", agreement.initiator_token.into_string())
        .add_attribute("counterparty_token", agreement.counterparty_token.into_string()))
}

/// Executes an accepted agreement, transferring tokens between parties.
fn execute_agreement(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    // Load agreement from storage by ID
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    // Verify sender is authorized (either initiator or counterparty)
    assert_sender_authorized(&info.sender, &[&agreement.initiator, &agreement.counterparty])?;

    // Assert agreement status is ACCEPTED before execution
    assert_agreement_has_status(&agreement.status, &[STATUS_ACCEPTED])?;

    // Verify contract has sufficient funds for initiator and counterparty tokens
    assert_contract_has_sufficient_funds(&deps, &env, &agreement.initiator_token)?;
    assert_contract_has_sufficient_funds(&deps, &env, &agreement.counterparty_token)?;

    // Update agreement status counts
    ACCEPTED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count - 1) })?;
    EXECUTED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    // Define messages to send tokens from one party to the other
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

    // Update agreement status to EXECUTED and save back to storage
    agreement.status = STATUS_EXECUTED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    // Return success response with messages and attributes
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "execute_agreement")
        .add_attribute("id", id.to_string())
        .add_attribute("status", agreement.status.to_string())
        .add_attribute("initiator", agreement.initiator)
        .add_attribute("counterparty", agreement.counterparty)
        .add_attribute("initiator_token", agreement.initiator_token.into_string())
        .add_attribute("counterparty_token", agreement.counterparty_token.into_string()))
}

/// Cancels an initiated or accepted agreement, refunding tokens if necessary.
fn cancel_agreement(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    // Load agreement from storage by ID
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    // Verify sender is authorized (either initiator or counterparty)
    assert_sender_authorized(&info.sender, &[&agreement.initiator, &agreement.counterparty])?;

    // Assert agreement status is INITIATED or ACCEPTED before cancellation
    assert_agreement_has_status(&agreement.status, &[STATUS_INITIATED, STATUS_ACCEPTED])?;

    // Update agreement status counts based on agreement status
    if agreement.status == STATUS_INITIATED {
        INITIATED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count - 1) })?;
    }
    if agreement.status == STATUS_ACCEPTED {
        ACCEPTED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count - 1) })?;
    }
    CANCELED_AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    // Vector to hold refund messages
    let mut messages: Vec<BankMsg> = Vec::new();

    // Refund initiator's tokens if they have sufficient funds stored in the contract
    if assert_contract_has_sufficient_funds(&deps, &env, &agreement.initiator_token).is_ok() {
        messages.push(BankMsg::Send {
            to_address: agreement.initiator.to_string(),
            amount: vec![coin(agreement.initiator_token.amount, &agreement.initiator_token.address)],
        });
    }

    // Refund counterparty's tokens if they have sufficient funds stored in the contract
    if assert_contract_has_sufficient_funds(&deps, &env, &agreement.counterparty_token).is_ok() {
        messages.push(BankMsg::Send {
            to_address: agreement.counterparty.to_string(),
            amount: vec![coin(agreement.counterparty_token.amount, &agreement.counterparty_token.address)],
        });
    }

    // Update agreement status to CANCELED and save back to storage
    agreement.status = STATUS_CANCELED.to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    // Return success response with refund messages and attributes
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "cancel_agreement")
        .add_attribute("id", id.to_string())
        .add_attribute("status", agreement.status.to_string())
        .add_attribute("initiator", agreement.initiator)
        .add_attribute("counterparty", agreement.counterparty)
        .add_attribute("initiator_token", agreement.initiator_token.into_string())
        .add_attribute("counterparty_token", agreement.counterparty_token.into_string()))
}

/// Handles incoming queries to retrieve agreement information.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAgreement { id } => to_json_binary(&query_agreement(deps, id)?),
        QueryMsg::GetTotalAgreementCount {} => to_json_binary(&query_total_agreement_count(deps)?),
        QueryMsg::GetInitiatedAgreementCount {} => to_json_binary(&query_initiated_agreement_count(deps)?),
        QueryMsg::GetAcceptedAgreementCount {} => to_json_binary(&query_accepted_agreement_count(deps)?),
        QueryMsg::GetExecutedAgreementCount {} => to_json_binary(&query_executed_agreement_count(deps)?),
        QueryMsg::GetCanceledAgreementCount {} => to_json_binary(&query_canceled_agreement_count(deps)?),
        QueryMsg::GetAgreementsByInitiator { initiator, page, page_size } => {
            let start_after = page.checked_mul(page_size).unwrap_or(0);
            let end_before = start_after.checked_add(page_size).unwrap_or(u64::MAX);

            to_json_binary(&query_agreements_by_initiator(deps, initiator, start_after, end_before)?)
        },
        QueryMsg::GetAgreementsByCounterparty { counterparty, page, page_size } => {
            let start_after = page.checked_mul(page_size).unwrap_or(0);
            let end_before = start_after.checked_add(page_size).unwrap_or(u64::MAX);

            to_json_binary(&query_agreements_by_counterparty(deps, counterparty, start_after, end_before)?)
        }
        QueryMsg::GetAgreementsByStatus { status, page, page_size } => {
            let start_after = page.checked_mul(page_size).unwrap_or(0);
            let end_before = start_after.checked_add(page_size).unwrap_or(u64::MAX);

            to_json_binary(&query_agreements_by_status(deps, status, start_after, end_before)?)
        },
    }
}
