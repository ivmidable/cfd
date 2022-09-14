#[cfg(test)]
mod tests {
    use crate::helpers::CwTemplateContract;
    use crate::msg::{InstantiateMsg, QueryMsg, GetCurrentPriceResponse, GetOldPricesResponse, PriceMsg};
    use crate::state::Price;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
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

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

        let msg = InstantiateMsg { owner: USER.to_string() };
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    fn query_get_current_price(app:&App, contract: &CwTemplateContract, base_asset:String, quote_asset:String) -> GetCurrentPriceResponse {
        app.wrap().query_wasm_smart(contract.addr(), &QueryMsg::GetCurrentPrice { base_asset, quote_asset }).unwrap()
    }

    fn query_get_old_prices(app:&App, contract: &CwTemplateContract, base_asset:String, quote_asset:String) -> GetOldPricesResponse {
        app.wrap().query_wasm_smart(contract.addr(), &QueryMsg::GetOldPrices { base_asset, quote_asset }).unwrap()
    }

    #[test]
    fn instantiate_and_add_single_price() {
        let (mut app, cw_template_contract) = proper_instantiate();

        let msg = crate::msg::ExecuteMsg::SetSinglePrice {
            base_asset: "base".to_string(),
            quote_asset: "quote".to_string(),
            amount: Uint128::new(100),
        };
        
        app
            .execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.0.clone(),
                &msg,
                &[],
            )
            .unwrap();

        let res = query_get_current_price(&app, &cw_template_contract, "base".to_string(), "quote".to_string());
        println!("res: {:?}", res);
        assert_eq!(res.price.amount, Uint128::new(100));

        let msg = crate::msg::ExecuteMsg::SetSinglePrice {
            base_asset: "base".to_string(),
            quote_asset: "quote".to_string(),
            amount: Uint128::new(1000),
        };
        
        app
            .execute_contract(
                Addr::unchecked(USER),
                cw_template_contract.0.clone(),
                &msg,
                &[],
            )
            .unwrap();

            let res = query_get_old_prices(&app, &cw_template_contract, "base".to_string(), "quote".to_string());
            println!("res: {:?}", res);
            let res = query_get_current_price(&app, &cw_template_contract, "base".to_string(), "quote".to_string());
            println!("res: {:?}", res);
            assert_eq!(res.price.amount, Uint128::new(1000));

    }
}
