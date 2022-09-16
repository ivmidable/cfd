#[cfg(test)]
mod tests {
    use crate::helpers::OracleContract;
    use crate::msg::{
        GetAllPricesResponse, GetCurrentPriceResponse, GetOldPricesResponse, InstantiateMsg,
        PriceMsg, QueryMsg,
    };
    use crate::state::Price;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn oracle_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "juno10c3slrqx3369mfsr9670au22zvq082jaej8ve4";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, OracleContract) {
        let mut app = mock_app();
        let oracle_id = app.store_code(oracle_contract());

        let msg = InstantiateMsg {
            owner: USER.to_string(),
        };
        let oracle_contract_addr = app
            .instantiate_contract(oracle_id, Addr::unchecked(ADMIN), &msg, &[], "test", None)
            .unwrap();

        let oracle_contract = OracleContract(oracle_contract_addr);

        (app, oracle_contract)
    }

    fn query_get_current_price(
        app: &App,
        contract: &OracleContract,
        base_asset: String,
        quote_asset: String,
    ) -> GetCurrentPriceResponse {
        app.wrap()
            .query_wasm_smart(
                contract.addr(),
                &QueryMsg::GetCurrentPrice {
                    base_asset,
                    quote_asset,
                },
            )
            .unwrap()
    }

    fn query_get_old_prices(
        app: &App,
        contract: &OracleContract,
        base_asset: String,
        quote_asset: String,
    ) -> GetOldPricesResponse {
        app.wrap()
            .query_wasm_smart(
                contract.addr(),
                &QueryMsg::GetOldPrices {
                    base_asset,
                    quote_asset,
                },
            )
            .unwrap()
    }

    fn query_get_all_current_prices(app: &App, contract: &OracleContract) -> GetAllPricesResponse {
        app.wrap()
            .query_wasm_smart(contract.addr(), &QueryMsg::GetAllCurrentPrices {})
            .unwrap()
    }

    #[test]
    fn instantiate_and_add_single_price() {
        let (mut app, cw_template_contract) = proper_instantiate();

        let msg = crate::msg::ExecuteMsg::SetSinglePrice {
            base_asset: "base".to_string(),
            quote_asset: "quote".to_string(),
            amount: Uint128::new(100),
        };

        app.execute_contract(
            Addr::unchecked(USER),
            cw_template_contract.0.clone(),
            &msg,
            &[],
        )
        .unwrap();

        let res = query_get_current_price(
            &app,
            &cw_template_contract,
            "base".to_string(),
            "quote".to_string(),
        );
        //println!("res: {:?}", res);
        assert_eq!(res.price.amount, Uint128::new(100));

        let msg = crate::msg::ExecuteMsg::SetSinglePrice {
            base_asset: "base".to_string(),
            quote_asset: "quote".to_string(),
            amount: Uint128::new(1000),
        };

        app.execute_contract(
            Addr::unchecked(USER),
            cw_template_contract.0.clone(),
            &msg,
            &[],
        )
        .unwrap();

        let res = query_get_old_prices(
            &app,
            &cw_template_contract,
            "base".to_string(),
            "quote".to_string(),
        );
        //println!("res: {:?}", res);
        assert_eq!(res.prices[0].1.amount, Uint128::new(100));
        let res = query_get_current_price(
            &app,
            &cw_template_contract,
            "base".to_string(),
            "quote".to_string(),
        );
        //println!("res: {:?}", res);
        assert_eq!(res.price.amount, Uint128::new(1000));
    }

    #[test]
    fn instantiate_and_add_batch_price() {
        let (mut app, oracle_contract) = proper_instantiate();

        let msg = crate::msg::ExecuteMsg::SetBatchPrice {
            prices: vec![
                PriceMsg {
                    base_asset: "base".to_string(),
                    quote_asset: "quote".to_string(),
                    amount: Uint128::new(100),
                },
                PriceMsg {
                    base_asset: "base2".to_string(),
                    quote_asset: "quote2".to_string(),
                    amount: Uint128::new(200),
                },
            ],
        };

        app.execute_contract(Addr::unchecked(USER), oracle_contract.0.clone(), &msg, &[])
            .unwrap();

        let res = query_get_current_price(
            &app,
            &oracle_contract,
            "base".to_string(),
            "quote".to_string(),
        );

        assert_eq!(res.price.amount, Uint128::new(100));

        let res = query_get_current_price(
            &app,
            &oracle_contract,
            "base2".to_string(),
            "quote2".to_string(),
        );

        assert_eq!(res.price.amount, Uint128::new(200));

        let res = query_get_all_current_prices(&app, &oracle_contract);
        assert_eq!(res.prices.len(), 2);
        assert_eq!(res.prices[0].0, ("base".to_string(), "quote".to_string()));
        assert_eq!(res.prices[0].1.amount, Uint128::new(100));
        assert_eq!(res.prices[1].0, ("base2".to_string(), "quote2".to_string()));
        assert_eq!(res.prices[1].1.amount, Uint128::new(200));
    }
}
