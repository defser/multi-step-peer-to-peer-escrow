use cosmwasm_std::{Addr, Coin, DepsMut, Env, Uint128};
use crate::ContractError;
use crate::msg::TokenInfo;

pub fn assert_sender_matches_counterparty(sender: &Addr, counterparty: &Addr) -> Result<(), ContractError> {
    if sender != counterparty {
        return Err(ContractError::Unauthorized {
            expected: counterparty.to_string(),
            found: sender.to_string()
        });
    }

    Ok(())
}

pub fn assert_sender_authorized(sender: &Addr, authorized_senders: &[&Addr]) -> Result<(), ContractError> {
    if !authorized_senders.contains(&sender) {
        return Err(ContractError::Unauthorized {
            expected: authorized_senders.iter().map(|addr| addr.to_string()).collect::<Vec<_>>().join(" or "),
            found: sender.to_string(),
        });
    }
    Ok(())
}

pub fn assert_agreement_has_status(agreement_status: &str, allowed_statuses: &[&str]) -> Result<(), ContractError> {
    if !allowed_statuses.contains(&agreement_status) {
        return Err(ContractError::InvalidAgreementStatus {
            expected: allowed_statuses.join(", "),
            found: agreement_status.to_string(),
        });
    }

    Ok(())
}


pub fn assert_funds_match_token_amount(funds: &Vec<Coin>, token: &TokenInfo) -> Result<(), ContractError> {
    let token_amount = Uint128::from(token.amount);

    for coin in funds.iter() {
        if coin.denom != token.address.to_string() {
            return Err(ContractError::UnexpectedFunds {
                expected: token.address.to_string(),
                found: coin.denom.clone(),
            });
        }
    }

    let sent_funds = funds.iter().find(|coin| coin.denom == token.address.to_string());

    match sent_funds {
        Some(coin) if coin.amount == token_amount => Ok(()),
        Some(coin) => Err(ContractError::IncorrectFundsAmount {
            expected: token_amount.to_string(),
            found: coin.amount.to_string(),
        }),
        None => Err(ContractError::InsufficientFunds {}),
    }
}

pub fn assert_contract_has_sufficient_funds(deps: &DepsMut, env: &Env, token_info: &TokenInfo) -> Result<(), ContractError> {
    let contract_addr = env.contract.address.clone();

    let contract_balance = deps
        .querier
        .query_balance(&contract_addr, &token_info.address)?;

    let token_amount = Uint128::from(token_info.amount);

    if contract_balance.amount < token_amount {
        return Err(ContractError::InsufficientContractFunds {
            expected: token_amount.to_string(),
            found: contract_balance.amount.to_string(),
        });
    }

    Ok(())
}
