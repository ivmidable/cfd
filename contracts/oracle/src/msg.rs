use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Addr, Uint128};
use crate::state::Price;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceMsg {
    pub base_asset: String,
    pub quote_asset: String,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponseMsg {
    pub base_asset: String,
    pub quote_asset: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {owner:String},
    SetSinglePrice {base_asset:String, quote_asset:String, amount:Uint128},
    SetBatchPrice { prices: Vec<PriceMsg> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetCurrentPrice { base_asset: String, quote_asset: String },
    GetCurrentBatchPrices { prices:Vec<PriceResponseMsg> },
    GetOldPrices {base_asset:String, quote_asset:String},
    //GetCurrentAllPrices { },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetConfigResponse {
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetCurrentPriceResponse {
    pub price: Price,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetCurrentBatchPricesResponse {
    pub prices: Vec<Price>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetOldPricesResponse {
    pub prices: Vec<(String, Price)>,
}


