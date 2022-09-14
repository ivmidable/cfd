#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, GetConfigResponse, GetCurrentBatchPricesResponse, GetCurrentPriceResponse,
    InstantiateMsg, PriceMsg, PriceResponseMsg, QueryMsg, GetOldPricesResponse
};
use crate::state::{Config, Price, CONFIG, CURRENT_PRICES, OLD_PRICES};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:oracle";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_validate(msg.owner.as_str())?,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", msg.owner))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => try_update_config(deps, info, owner),
        ExecuteMsg::SetSinglePrice {
            base_asset,
            quote_asset,
            amount,
        } => try_set_single_price(deps, info, env, base_asset, quote_asset, amount),
        ExecuteMsg::SetBatchPrice { prices } => try_set_batch_price(deps, info, env, prices),
    }
}

pub fn try_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    config.owner = deps.api.addr_validate(owner.as_str())?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "update_config")
        .add_attribute("owner", owner))
}

pub fn try_set_single_price(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    base_asset: String,
    quote_asset: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let price = Price {
        timestamp: env.block.time.seconds(),
        amount,
    };

    match CURRENT_PRICES.load(deps.storage, (base_asset.as_str(), quote_asset.as_str())) {
        Ok(price_data) => {
            OLD_PRICES.save(
                deps.storage,
                (
                    base_asset.as_str(),
                    quote_asset.as_str(),
                    price_data.timestamp.to_string().as_str(),
                ),
                &price_data,
            )?;
            CURRENT_PRICES.save(
                deps.storage,
                (base_asset.as_str(), quote_asset.as_str()),
                &price,
            )?;
        }
        Err(_) => {
            CURRENT_PRICES.save(
                deps.storage,
                (base_asset.as_str(), quote_asset.as_str()),
                &price,
            )?;
        }
    }

    Ok(Response::new()
        .add_attribute("method", "set_single_price")
        .add_attribute("base_asset", base_asset)
        .add_attribute("quote_asset", quote_asset)
        .add_attribute("amount", amount.to_string()))
}

pub fn try_set_batch_price(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    prices: Vec<PriceMsg>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    for price in prices {
        let price_data = Price {
            timestamp: env.block.time.seconds(),
            amount: price.amount,
        };

        match CURRENT_PRICES.load(
            deps.storage,
            (price.base_asset.as_str(), price.quote_asset.as_str()),
        ) {
            Ok(price_data) => {
                OLD_PRICES.save(
                    deps.storage,
                    (
                        price.base_asset.as_str(),
                        price.quote_asset.as_str(),
                        price_data.timestamp.to_string().as_str(),
                    ),
                    &price_data,
                )?;
                CURRENT_PRICES.save(
                    deps.storage,
                    (price.base_asset.as_str(), price.quote_asset.as_str()),
                    &price_data,
                )?;
            }
            Err(_) => {
                CURRENT_PRICES.save(
                    deps.storage,
                    (price.base_asset.as_str(), price.quote_asset.as_str()),
                    &price_data,
                )?;
            }
        }
    }

    Ok(Response::new().add_attribute("method", "set_batch_price"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetCurrentPrice {
            base_asset,
            quote_asset,
        } => to_binary(&query_current_price(deps, base_asset, quote_asset)?),
        QueryMsg::GetCurrentBatchPrices { prices } => {
            to_binary(&query_current_batch_prices(deps, prices)?)
        }
        QueryMsg::GetOldPrices {
            base_asset,
            quote_asset,
        } => to_binary(&query_old_prices(deps, base_asset, quote_asset)?),
        //QueryMsg::GetCurrentAllPrices {  } => to_binary(&query_current_all_prices(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<GetConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(GetConfigResponse {
        owner: config.owner.to_string(),
    })
}

fn query_current_price(
    deps: Deps,
    base_asset: String,
    quote_asset: String,
) -> StdResult<GetCurrentPriceResponse> {
    let price = CURRENT_PRICES.load(deps.storage, (base_asset.as_str(), quote_asset.as_str()))?;
    Ok(GetCurrentPriceResponse { price })
}

fn query_current_batch_prices(
    deps: Deps,
    prices: Vec<PriceResponseMsg>,
) -> StdResult<GetCurrentBatchPricesResponse> {
    let mut price_list: Vec<Price> = vec![];
    for price in prices {
        let price_data = CURRENT_PRICES.load(
            deps.storage,
            (price.base_asset.as_str(), price.quote_asset.as_str()),
        )?;
        price_list.push(price_data);
    }
    Ok(GetCurrentBatchPricesResponse { prices: price_list })
}

fn query_old_prices(
    deps: Deps,
    base_asset: String,
    quote_asset: String,
) -> StdResult<GetOldPricesResponse> {
    let res: StdResult<_> = OLD_PRICES
        .prefix((base_asset.as_str(), quote_asset.as_str()))
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let prices = res?;
    Ok(GetOldPricesResponse { prices })
}

/*fn query_current_all_prices(deps: Deps) -> StdResult<GetCurrentBatchPricesResponse> {
    let price_list = CURRENT_PRICES.range(deps.storage, None, None, Order::Ascending).collect();
    Ok(GetCurrentBatchPricesResponse {
        prices: price_list
    })
}*/
