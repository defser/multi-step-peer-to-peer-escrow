use crate::msg::TokenInfo;
use crate::ContractError;
use cosmwasm_std::{Addr, Coin, DepsMut, Env, Uint128};

// Asserts that the sender matches the expected counterparty address.
pub fn assert_sender_matches_counterparty(
    sender: &Addr,
    counterparty: &Addr,
) -> Result<(), ContractError> {
    // Check if the sender address matches the counterparty address
    if sender != counterparty {
        // If not, return an Unauthorized error indicating the expected and found addresses
        return Err(ContractError::Unauthorized {
            expected: counterparty.to_string(),
            found: sender.to_string(),
        });
    }

    // Return Ok(()) if addresses match
    Ok(())
}

// Asserts that the sender is different from the counterparty address.
pub fn assert_sender_is_different_from_counterparty(
    sender: &Addr,
    counterparty: &Addr,
) -> Result<(), ContractError> {
    // Check if the sender address is the same as the counterparty address
    if sender == counterparty {
        // If they are the same, return an InvalidCounterparty error indicating the addresses
        return Err(ContractError::InvalidCounterparty {
            initiator: sender.to_string(),
            counterparty: counterparty.to_string(),
        });
    }

    // Return Ok(()) if addresses are different
    Ok(())
}

// Asserts that the sender is in the list of authorized senders.
pub fn assert_sender_authorized(
    sender: &Addr,
    authorized_senders: &[&Addr],
) -> Result<(), ContractError> {
    // Check if the sender address is in the list of authorized senders
    if !authorized_senders.contains(&sender) {
        // If not, return an Unauthorized error indicating expected and found addresses
        return Err(ContractError::Unauthorized {
            expected: authorized_senders
                .iter()
                .map(|addr| addr.to_string())
                .collect::<Vec<_>>()
                .join(" or "),
            found: sender.to_string(),
        });
    }

    // Return Ok(()) if sender is authorized
    Ok(())
}

// Asserts that the agreement status matches one of the allowed statuses.
pub fn assert_agreement_has_status(
    agreement_status: &str,
    allowed_statuses: &[&str],
) -> Result<(), ContractError> {
    // Check if the agreement status is in the list of allowed statuses
    if !allowed_statuses.contains(&agreement_status) {
        // If not, return an InvalidAgreementStatus error indicating expected and found statuses
        return Err(ContractError::InvalidAgreementStatus {
            expected: allowed_statuses.join(", "),
            found: agreement_status.to_string(),
        });
    }

    // Return Ok(()) if agreement status is valid
    Ok(())
}

// Asserts that the funds match the expected token amount.
pub fn assert_funds_match_token_amount(
    funds: &Vec<Coin>,
    token: &TokenInfo,
) -> Result<(), ContractError> {
    // Convert token amount from u128 to Uint128 for consistency
    let token_amount = Uint128::from(token.amount);

    // Iterate through each coin in the provided funds
    for coin in funds.iter() {
        // Check if the denomination of the coin matches the token's address
        if coin.denom != token.address.to_string() {
            // If not, return an error indicating unexpected funds
            return Err(ContractError::UnexpectedFunds {
                expected: token.address.to_string(),
                found: coin.denom.clone(),
            });
        }
    }

    // Find the specific coin that matches the token's address
    let sent_funds = funds
        .iter()
        .find(|coin| coin.denom == token.address.to_string());

    // Match against the found coin
    match sent_funds {
        Some(coin) if coin.amount == token_amount => Ok(()), // If amount matches, return Ok
        Some(coin) => Err(ContractError::IncorrectFundsAmount {
            // If amount doesn't match, return error
            expected: token_amount.to_string(),
            found: coin.amount.to_string(),
        }),
        None => Err(ContractError::InsufficientFunds {}), // If no funds are found for the token, return insufficient funds error
    }
}

// Asserts that the contract has sufficient funds of a specific token.
pub fn assert_contract_has_sufficient_funds(
    deps: &DepsMut,
    env: &Env,
    token_info: &TokenInfo,
) -> Result<(), ContractError> {
    // Clone the contract address from the environment
    let contract_addr = env.contract.address.clone();

    // Query the balance of the contract for the specified token
    let contract_balance = deps
        .querier
        .query_balance(&contract_addr, &token_info.address)?;

    // Convert token amount from u128 to Uint128 for consistency
    let token_amount = Uint128::from(token_info.amount);

    // Check if the contract balance is less than the required token amount
    if contract_balance.amount < token_amount {
        // If insufficient funds, return an InsufficientContractFunds error indicating expected and found amounts
        return Err(ContractError::InsufficientContractFunds {
            expected: token_amount.to_string(),
            found: contract_balance.amount.to_string(),
        });
    }

    // Return Ok(()) if contract has sufficient funds
    Ok(())
}
