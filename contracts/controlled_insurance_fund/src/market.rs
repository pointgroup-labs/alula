use soroban_sdk::{Env, contractclient};

#[contractclient(name = "MarketPartialClient")]
pub trait MarketPartial {
    fn update_market_status(e: Env, new_status: u32);
}
