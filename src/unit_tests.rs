#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, message_info, mock_dependencies_with_balances};
    use cosmwasm_std::{Addr, Attribute, coin, coins, from_json};
    use crate::contract::{STATUS_ACCEPTED, STATUS_CANCELED, STATUS_EXECUTED, STATUS_INITIATED, execute, instantiate, query};
    use crate::ContractError;
    use crate::msg::{AgreementResponse, AgreementsResponse, ExecuteMsg, InstantiateMsg, QueryMsg, TokenInfo};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreement { id: 1 });
        assert!(res.is_err());
    }

    #[test]
    fn initiate_and_accept_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![
            Attribute { key: "method".to_string(), value: "initiate_agreement".to_string() },
            Attribute { key: "id".to_string(), value: "1".to_string() },
            Attribute { key: "initiator".to_string(), value: "initiator".to_string() },
            Attribute { key: "counterparty".to_string(), value: "counterparty".to_string() },
            Attribute { key: "initiator_token".to_string(), value: "TokenInfo { address: Addr(\"tokenA\"), amount: 1000 }".to_string() },
            Attribute { key: "counterparty_token".to_string(), value: "TokenInfo { address: Addr(\"tokenB\"), amount: 2000 }".to_string() }
        ]);

        let msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let info = message_info(&Addr::unchecked("counterparty"), &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes, vec![
            Attribute { key: "method".to_string(), value: "accept_agreement".to_string() },
            Attribute { key: "id".to_string(), value: "1".to_string() },
            Attribute { key: "initiator".to_string(), value: "initiator".to_string() },
            Attribute { key: "counterparty".to_string(), value: "counterparty".to_string() },
            Attribute { key: "initiator_token".to_string(), value: "TokenInfo { address: Addr(\"tokenA\"), amount: 1000 }".to_string() },
            Attribute { key: "counterparty_token".to_string(), value: "TokenInfo { address: Addr(\"tokenB\"), amount: 2000 }".to_string() }
        ]);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreement { id: 1 }).unwrap();
        let value: AgreementResponse = from_json(&res).unwrap();
        assert_eq!(value.agreement.id, 1);
        assert_eq!(value.agreement.initiator, Addr::unchecked("initiator"));
        assert_eq!(value.agreement.counterparty, Addr::unchecked("counterparty"));
        assert_eq!(value.agreement.initiator_token, initiator_token);
        assert_eq!(value.agreement.counterparty_token, counterparty_token);
        assert_eq!(value.agreement.status, STATUS_ACCEPTED);
    }

    #[test]
    fn incorrect_funds_initiate_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
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
    fn insufficient_funds_initiate_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &[]); // Insufficient funds
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::InsufficientFunds {} => {},
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn unexpected_funds_initiate_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "someToken"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::UnexpectedFunds { expected, found } => {
                assert_eq!(expected, "tokenA");
                assert_eq!(found, "someToken");
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn insufficient_funds_accept_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_ok());

        // Accept the agreement
        let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let accept_info = message_info(&Addr::unchecked("counterparty"), &[]);
        let res = execute(deps.as_mut(), mock_env(), accept_info.clone(), accept_msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::InsufficientFunds {} => {},
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn unexpected_funds_accept_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_ok());

        // Accept the agreement
        let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let accept_info = message_info(&Addr::unchecked("counterparty"), &coins(2000, "someToken"));
        let res = execute(deps.as_mut(), mock_env(), accept_info.clone(), accept_msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::UnexpectedFunds { expected, found } => {
                assert_eq!(expected, "tokenB");
                assert_eq!(found, "someToken");
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn unauthorized_accept_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res= execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_ok());

        // Accept the agreement
        let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let accept_info = message_info(&Addr::unchecked("some-other-counterparty"), &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), accept_info.clone(), accept_msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::Unauthorized { expected, found } => {
                assert_eq!(expected, "counterparty");
                assert_eq!(found, "some-other-counterparty");
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn unauthorized_execute_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res= execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_ok());

        // Accept the agreement
        let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let accept_info = message_info(&Addr::unchecked("counterparty"), &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), accept_info.clone(), accept_msg);

        assert!(res.is_ok());

        // Execute the agreement
        let execute_msg = ExecuteMsg::ExecuteAgreement { id: 1 };
        let execute_info = message_info(&Addr::unchecked("some-other-counterparty"), &[]);
        let res = execute(deps.as_mut(), mock_env(), execute_info.clone(), execute_msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::Unauthorized { expected, found } => {
                assert_eq!(expected, "initiator or counterparty");
                assert_eq!(found, "some-other-counterparty");
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn unauthorized_cancel_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let res= execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_ok());

        // Accept the agreement
        let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let accept_info = message_info(&Addr::unchecked("counterparty"), &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), accept_info.clone(), accept_msg);

        assert!(res.is_ok());

        // Cancel the agreement
        let execute_msg = ExecuteMsg::CancelAgreement { id: 1 };
        let execute_info = message_info(&Addr::unchecked("some-other-counterparty"), &[]);
        let res = execute(deps.as_mut(), mock_env(), execute_info.clone(), execute_msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::Unauthorized { expected, found } => {
                assert_eq!(expected, "initiator or counterparty");
                assert_eq!(found, "some-other-counterparty");
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

        assert_eq!(agreement_response.agreement.status, STATUS_EXECUTED);
    }

    #[test]
    fn execute_agreement_insufficient_funds_in_contract() {
        // Arrange
        let mut deps = mock_dependencies_with_balances(&[
            ((&Addr::unchecked("initiator")).as_ref(), &[coin(1000, "tokenA")]),
            ((&Addr::unchecked("counterparty")).as_ref(), &[coin(2000, "tokenB")]),
            ((&Addr::unchecked("cosmos2contract")).as_ref(), &[coin(500, "tokenA"), coin(1000, "tokenB")]),
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
        let res = execute(deps.as_mut(), mock_env(), execute_info.clone(), execute_msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::InsufficientContractFunds { expected, found } => {
                assert_eq!(expected, "1000");
                assert_eq!(found, "500");
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn cancel_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
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

        assert_eq!(res.attributes, vec![
            Attribute { key: "method".to_string(), value: "initiate_agreement".to_string() },
            Attribute { key: "id".to_string(), value: "1".to_string() },
            Attribute { key: "initiator".to_string(), value: "initiator".to_string() },
            Attribute { key: "counterparty".to_string(), value: "counterparty".to_string() },
            Attribute { key: "initiator_token".to_string(), value: "TokenInfo { address: Addr(\"tokenA\"), amount: 1000 }".to_string() },
            Attribute { key: "counterparty_token".to_string(), value: "TokenInfo { address: Addr(\"tokenB\"), amount: 2000 }".to_string() }
        ]);

        let msg = ExecuteMsg::CancelAgreement { id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.attributes, vec![
            Attribute { key: "method".to_string(), value: "cancel_agreement".to_string() },
            Attribute { key: "id".to_string(), value: "1".to_string() },
            Attribute { key: "initiator".to_string(), value: "initiator".to_string() },
            Attribute { key: "counterparty".to_string(), value: "counterparty".to_string() },
            Attribute { key: "initiator_token".to_string(), value: "TokenInfo { address: Addr(\"tokenA\"), amount: 1000 }".to_string() },
            Attribute { key: "counterparty_token".to_string(), value: "TokenInfo { address: Addr(\"tokenB\"), amount: 2000 }".to_string() }
        ]);

        let query_msg = QueryMsg::GetAgreement { id: 1 };
        let query_res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let agreement_response: AgreementResponse = from_json(&query_res).unwrap();

        assert_eq!(agreement_response.agreement.status, STATUS_CANCELED);
    }


    #[test]
    fn accept_cancelled_agreement() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CancelAgreement { id: 1 };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        assert_eq!(res.attributes, vec![
            Attribute { key: "method".to_string(), value: "cancel_agreement".to_string() },
            Attribute { key: "id".to_string(), value: "1".to_string() },
            Attribute { key: "initiator".to_string(), value: "initiator".to_string() },
            Attribute { key: "counterparty".to_string(), value: "counterparty".to_string() },
            Attribute { key: "initiator_token".to_string(), value: "TokenInfo { address: Addr(\"tokenA\"), amount: 1000 }".to_string() },
            Attribute { key: "counterparty_token".to_string(), value: "TokenInfo { address: Addr(\"tokenB\"), amount: 2000 }".to_string() }
        ]);

        let msg = ExecuteMsg::AcceptAgreement { id: 1 };
        let info = message_info(&counterparty, &coins(2000, "tokenB"));
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);

        assert!(res.is_err());
        match res.err().unwrap() {
            ContractError::InvalidAgreementStatus { expected, found } => {
                assert_eq!(expected, format!("{}", STATUS_INITIATED));
                assert_eq!(found, STATUS_CANCELED);
            },
            _ => panic!("Unexpected error"),
        }
    }

    #[test]
    fn query_agreements_by_initiator() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };
        let counterparty = Addr::unchecked("counterparty");

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: counterparty.clone() };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty2") };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreementsByInitiator { initiator: Addr::unchecked("initiator"), page: 0, page_size: 10 }).unwrap();
        let value: AgreementsResponse = from_json(&res).unwrap();
        assert_eq!(value.agreements.len(), 2);
    }

    #[test]
    fn query_agreements_by_counterparty() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty") };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty2") };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreementsByCounterparty { counterparty: Addr::unchecked("counterparty"), page: 0, page_size: 10 }).unwrap();
        let value: AgreementsResponse = from_json(&res).unwrap();
        assert_eq!(value.agreements.len(), 1);
    }

    #[test]
    fn query_agreements_by_status() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {};
        let info = message_info(&Addr::unchecked("creator"), &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let initiator_token = TokenInfo { address: Addr::unchecked("tokenA"), amount: 1000u128 };
        let counterparty_token = TokenInfo { address: Addr::unchecked("tokenB"), amount: 2000u128 };

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty") };
        let info = message_info(&Addr::unchecked("initiator"), &coins(1000, "tokenA"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::InitiateAgreement { initiator_token: initiator_token.clone(), counterparty_token: counterparty_token.clone(), counterparty: Addr::unchecked("counterparty2") };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAgreementsByStatus { status: STATUS_INITIATED.to_string(), page: 0, page_size: 10 }).unwrap();
        let value: AgreementsResponse = from_json(&res).unwrap();
        assert_eq!(value.agreements.len(), 2);
    }
}
