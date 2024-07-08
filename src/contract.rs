#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, to_json_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{AgreementResponse, AgreementsResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};
use crate::state::{Agreement, AGREEMENTS, AGREEMENT_COUNT};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:multi-token-agreement";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::InitiateAgreement { token_a, token_b, counterparty } => {
            try_initiate_agreement(deps, info, token_a, token_b, counterparty)
        }
        ExecuteMsg::AcceptAgreement { id } => try_accept_agreement(deps, info, id),
        ExecuteMsg::ExecuteAgreement { id } => try_execute_agreement(deps, info, id),
        ExecuteMsg::CancelAgreement { id } => try_cancel_agreement(deps, info, id),
    }
}

fn try_initiate_agreement(
    deps: DepsMut,
    info: MessageInfo,
    token_a: TokenInfo,
    token_b: TokenInfo,
    counterparty: Addr,
) -> Result<Response, ContractError> {
    let id = AGREEMENT_COUNT.update(deps.storage, |count| -> StdResult<_> { Ok(count + 1) })?;

    let agreement = Agreement {
        id,
        initiator: info.sender.clone(),
        counterparty,
        token_a,
        token_b,
        status: "initiated".to_string(),
    };

    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "initiate_agreement")
        .add_attribute("id", id.to_string()))
}

fn try_accept_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    if info.sender != agreement.counterparty {
        return Err(ContractError::Unauthorized {});
    }

    agreement.status = "accepted".to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "accept_agreement")
        .add_attribute("id", id.to_string()))
}

fn try_execute_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut agreement = AGREEMENTS.load(deps.storage, id)?;

    if info.sender != agreement.initiator && info.sender != agreement.counterparty {
        return Err(ContractError::Unauthorized {});
    }

    if agreement.status != "accepted" {
        return Err(ContractError::InvalidAgreementStatus {});
    }

    // Ensure both tokens are deposited (this logic is simplified and assumes prior token transfers)

    agreement.status = "executed".to_string();
    AGREEMENTS.save(deps.storage, id, &agreement)?;

    Ok(Response::new()
        .add_attribute("method", "execute_agreement")
        .add_attribute("id", id.to_string()))
}

fn try_cancel_agreement(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let agreement = AGREEMENTS.load(deps.storage, id)?;

    if info.sender != agreement.initiator {
        return Err(ContractError::Unauthorized {});
    }

    if agreement.status != "initiated" {
        return Err(ContractError::InvalidAgreementStatus {});
    }

    AGREEMENTS.remove(deps.storage, id);

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

fn query_agreement(deps: Deps, id: u64) -> StdResult<AgreementResponse> {
    let agreement = AGREEMENTS.load(deps.storage, id)?;
    Ok(AgreementResponse { agreement })
}

fn query_agreements_by_initiator(deps: Deps, initiator: Addr) -> StdResult<AgreementsResponse> {
    let agreements: Vec<Agreement> = AGREEMENTS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .filter_map(|item| match item {
            Ok((_, agreement)) => {
                if agreement.initiator == initiator {
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

fn query_agreements_by_counterparty(deps: Deps, counterparty: Addr) -> StdResult<AgreementsResponse> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, message_info};
    use cosmwasm_std::{coins, from_json};

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

        let token_a = TokenInfo { address: Addr::unchecked("tokenA"), amount: 100u128 };
        let token_b = TokenInfo { address: Addr::unchecked("tokenB"), amount: 200u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { token_a: token_a.clone(), token_b: token_b.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "earth"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "initiate_agreement"), ("id", "1")]);

        let msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let info = message_info(&Addr::unchecked("counterparty"), &coins(1000, "earth"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "accept_agreement"), ("id", "1")]);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreement { id: 1 }).unwrap();
        let value: AgreementResponse = from_json(&res).unwrap();
        assert_eq!(value.agreement.id, 1);
        assert_eq!(value.agreement.initiator, Addr::unchecked("initiator"));
        assert_eq!(value.agreement.counterparty, Addr::unchecked("counterparty"));
        assert_eq!(value.agreement.token_a, token_a);
        assert_eq!(value.agreement.token_b, token_b);
        assert_eq!(value.agreement.status, "accepted");
    }

    #[test]
    fn cancel_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let token_a = TokenInfo { address: Addr::unchecked("tokenA"), amount: 100u128 };
        let token_b = TokenInfo { address: Addr::unchecked("tokenB"), amount: 200u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { token_a: token_a.clone(), token_b: token_b.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "earth"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "initiate_agreement"), ("id", "1")]);

        let msg = ExecuteMsg::CancelAgreement { id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes, vec![("method", "cancel_agreement"), ("id", "1")]);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreement { id: 1 });
        assert!(res.is_err());
    }

    #[test]
    fn query_agreements_by_initiator() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let token_a = TokenInfo { address: Addr::unchecked("tokenA"), amount: 100u128 };
        let token_b = TokenInfo { address: Addr::unchecked("tokenB"), amount: 200u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { token_a: token_a.clone(), token_b: token_b.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { token_a: token_a.clone(), token_b: token_b.clone(), counterparty: Addr::unchecked("counterparty2") };
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

        let token_a = TokenInfo { address: Addr::unchecked("tokenA"), amount: 100u128 };
        let token_b = TokenInfo { address: Addr::unchecked("tokenB"), amount: 200u128 };

        let msg = ExecuteMsg::InitiateAgreement { token_a: token_a.clone(), token_b: token_b.clone(), counterparty: Addr::unchecked("counterparty") };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { token_a: token_a.clone(), token_b: token_b.clone(), counterparty: Addr::unchecked("counterparty2") };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreementsByCounterparty { counterparty: Addr::unchecked("counterparty") }).unwrap();
        let value: AgreementsResponse = from_json(&res).unwrap();
        assert_eq!(value.agreements.len(), 1);
    }
}
