use soroban_sdk::Env;

pub trait MarketPartial {
    fn update_market_status(e: Env, new_status: u32);
}
