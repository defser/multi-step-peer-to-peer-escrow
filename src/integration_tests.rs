#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const INITIATOR: &str = "initiator";
    const COUNTERPARTY: &str = "counterparty";
    const ADMIN: &str = "admin";
    const TOKEN_A: &str = "TokenA";
    const TOKEN_B: &str = "TokenB";

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = App::default();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg {};
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                app.api().addr_make(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod agreement_tests {
        use super::*;
        use crate::msg::{ExecuteMsg, TokenInfo};
        use cosmwasm_std::coins;

        #[test]
        fn initiate_agreement() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let initiator_addr = app.api().addr_make(INITIATOR);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &initiator_addr.clone(), coins(10000, TOKEN_A))
                    .unwrap();
            });

            let counterparty_addr = app.api().addr_make(COUNTERPARTY);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &counterparty_addr.clone(), coins(10000, TOKEN_B))
                    .unwrap();
            });

            let msg = ExecuteMsg::InitiateAgreement {
                initiator_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_A),
                    amount: 1000u128,
                },
                counterparty_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_B),
                    amount: 2000u128,
                },
                counterparty: counterparty_addr.clone(),
            };

            let cosmos_msg = cw_template_contract
                .call(msg, coins(1000, TOKEN_A))
                .unwrap();

            app.execute(initiator_addr.clone(), cosmos_msg).unwrap();

            let initiator_token_a_balance = app
                .wrap()
                .query_balance(initiator_addr.clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(initiator_token_a_balance, 9000);

            let contract_token_a_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_a_balance, 1000);
        }

        #[test]
        fn accept_agreement() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let initiator_addr = app.api().addr_make(INITIATOR);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &initiator_addr.clone(), coins(10000, TOKEN_A))
                    .unwrap();
            });

            let counterparty_addr = app.api().addr_make(COUNTERPARTY);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &counterparty_addr.clone(), coins(10000, TOKEN_B))
                    .unwrap();
            });

            let initiate_msg = ExecuteMsg::InitiateAgreement {
                initiator_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_A),
                    amount: 1000u128,
                },
                counterparty_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_B),
                    amount: 2000u128,
                },
                counterparty: counterparty_addr.clone(),
            };

            let cosmos_msg = cw_template_contract
                .call(initiate_msg, coins(1000, TOKEN_A))
                .unwrap();

            app.execute(initiator_addr.clone(), cosmos_msg).unwrap();

            let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };

            let cosmos_msg = cw_template_contract
                .call(accept_msg, coins(2000, TOKEN_B))
                .unwrap();

            app.execute(counterparty_addr.clone(), cosmos_msg).unwrap();

            let initiator_token_a_balance = app
                .wrap()
                .query_balance(initiator_addr.clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(initiator_token_a_balance, 9000);

            let counterparty_token_b_balance = app
                .wrap()
                .query_balance(counterparty_addr.clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(counterparty_token_b_balance, 8000);

            let contract_token_a_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_a_balance, 1000);

            let contract_token_b_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_b_balance, 2000);
        }

        #[test]
        fn execute_agreement() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let initiator_addr = app.api().addr_make(INITIATOR);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &initiator_addr.clone(), coins(10000, TOKEN_A))
                    .unwrap();
            });

            let counterparty_addr = app.api().addr_make(COUNTERPARTY);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &counterparty_addr.clone(), coins(10000, TOKEN_B))
                    .unwrap();
            });

            let initiate_msg = ExecuteMsg::InitiateAgreement {
                initiator_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_A),
                    amount: 1000u128,
                },
                counterparty_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_B),
                    amount: 2000u128,
                },
                counterparty: counterparty_addr.clone(),
            };

            let cosmos_msg = cw_template_contract
                .call(initiate_msg, coins(1000, TOKEN_A))
                .unwrap();

            app.execute(initiator_addr.clone(), cosmos_msg).unwrap();

            let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };

            let cosmos_msg = cw_template_contract
                .call(accept_msg, coins(2000, TOKEN_B))
                .unwrap();

            app.execute(counterparty_addr.clone(), cosmos_msg).unwrap();

            let execute_msg = ExecuteMsg::ExecuteAgreement { id: 1 };

            let cosmos_msg = cw_template_contract.call(execute_msg, vec![]).unwrap();

            app.execute(counterparty_addr.clone(), cosmos_msg).unwrap();

            let initiator_token_a_balance = app
                .wrap()
                .query_balance(initiator_addr.clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(initiator_token_a_balance, 9000);

            let initiator_token_b_balance = app
                .wrap()
                .query_balance(initiator_addr.clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(initiator_token_b_balance, 2000);

            let counterparty_token_b_balance = app
                .wrap()
                .query_balance(counterparty_addr.clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(counterparty_token_b_balance, 8000);

            let counterparty_token_a_balance = app
                .wrap()
                .query_balance(counterparty_addr.clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(counterparty_token_a_balance, 1000);

            let contract_token_a_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_a_balance, 0);

            let contract_token_b_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_b_balance, 0);
        }

        #[test]
        fn cancel_accepted_agreement() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let initiator_addr = app.api().addr_make(INITIATOR);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &initiator_addr.clone(), coins(10000, TOKEN_A))
                    .unwrap();
            });

            let counterparty_addr = app.api().addr_make(COUNTERPARTY);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &counterparty_addr.clone(), coins(10000, TOKEN_B))
                    .unwrap();
            });

            let initiate_msg = ExecuteMsg::InitiateAgreement {
                initiator_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_A),
                    amount: 1000u128,
                },
                counterparty_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_B),
                    amount: 2000u128,
                },
                counterparty: counterparty_addr.clone(),
            };

            let cosmos_msg = cw_template_contract
                .call(initiate_msg, coins(1000, TOKEN_A))
                .unwrap();

            app.execute(initiator_addr.clone(), cosmos_msg).unwrap();

            let accept_msg = ExecuteMsg::AcceptAgreement { id: 1 };

            let cosmos_msg = cw_template_contract
                .call(accept_msg, coins(2000, TOKEN_B))
                .unwrap();

            app.execute(counterparty_addr.clone(), cosmos_msg).unwrap();

            let cancel_msg = ExecuteMsg::CancelAgreement { id: 1 };

            let cosmos_msg = cw_template_contract.call(cancel_msg, vec![]).unwrap();

            app.execute(counterparty_addr.clone(), cosmos_msg).unwrap();

            let initiator_token_a_balance = app
                .wrap()
                .query_balance(initiator_addr.clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(initiator_token_a_balance, 10000);

            let counterparty_token_b_balance = app
                .wrap()
                .query_balance(counterparty_addr.clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(counterparty_token_b_balance, 10000);

            let contract_token_a_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_a_balance, 0);

            let contract_token_b_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_b_balance, 0);
        }

        #[test]
        fn cancel_initiated_agreement() {
            let (mut app, cw_template_contract) = proper_instantiate();

            let initiator_addr = app.api().addr_make(INITIATOR);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &initiator_addr.clone(), coins(10000, TOKEN_A))
                    .unwrap();
            });

            let counterparty_addr = app.api().addr_make(COUNTERPARTY);

            app.init_modules(|router, _, storage| {
                router
                    .bank
                    .init_balance(storage, &counterparty_addr.clone(), coins(10000, TOKEN_B))
                    .unwrap();
            });

            let initiate_msg = ExecuteMsg::InitiateAgreement {
                initiator_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_A),
                    amount: 1000u128,
                },
                counterparty_token: TokenInfo {
                    address: Addr::unchecked(TOKEN_B),
                    amount: 2000u128,
                },
                counterparty: counterparty_addr.clone(),
            };

            let cosmos_msg = cw_template_contract
                .call(initiate_msg, coins(1000, TOKEN_A))
                .unwrap();

            app.execute(initiator_addr.clone(), cosmos_msg).unwrap();

            let cancel_msg = ExecuteMsg::CancelAgreement { id: 1 };

            let cosmos_msg = cw_template_contract.call(cancel_msg, vec![]).unwrap();

            app.execute(counterparty_addr.clone(), cosmos_msg).unwrap();

            let initiator_token_a_balance = app
                .wrap()
                .query_balance(initiator_addr.clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(initiator_token_a_balance, 10000);

            let counterparty_token_b_balance = app
                .wrap()
                .query_balance(counterparty_addr.clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(counterparty_token_b_balance, 10000);

            let contract_token_a_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_A)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_a_balance, 0);

            let contract_token_b_balance = app
                .wrap()
                .query_balance(cw_template_contract.addr().clone(), TOKEN_B)
                .unwrap()
                .amount
                .u128();
            assert_eq!(contract_token_b_balance, 0);
        }
    }
}
