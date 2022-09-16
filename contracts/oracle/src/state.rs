use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Price {
    pub amount: Uint128,
    pub timestamp: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

//key is base pair, quote pair
pub const CURRENT_PRICES: Map<(&str, &str), Price> = Map::new("curent_prices");
//key is base_pair, quote_pair, timestamp
pub const OLD_PRICES: Map<(&str, &str, &str), Price> = Map::new("old_prices");
