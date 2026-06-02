use soroban_sdk::{Address, contractevent};

#[contractevent]
pub struct FeedAdded {
    #[topic]
    pub token_address: Address,
    #[topic]
    pub price_feed: Address,
}

#[contractevent]
pub struct ResolutionUpdated {
    pub resolution: u32,
}

#[contractevent]
pub struct FuturePriceRejected {
    #[topic]
    pub token_address: Address,
    pub feed_ts_secs: u64,
    pub now: u64,
}

#[contractevent]
pub struct StalePriceRejected {
    #[topic]
    pub token_address: Address,
    pub feed_ts_secs: u64,
    pub now: u64,
    pub resolution: u32,
}

#[contractevent]
pub struct AdminProposed {
    #[topic]
    pub proposed_admin: Address,
}

#[contractevent]
pub struct AdminAccepted {
    #[topic]
    pub new_admin: Address,
}
