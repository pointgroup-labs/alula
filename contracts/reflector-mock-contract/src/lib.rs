#![no_std]

pub mod mock_oracle {
    use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

    #[contracttype]
    #[derive(Debug)]
    pub enum Asset {
        Stellar(Address),
        Other(Symbol),
    }

    #[contracttype]
    #[derive(Debug)]
    pub struct PriceData {
        pub price: i128,
        pub timestamp: u64,
    }

    #[contract]
    pub struct MockOracleContract;

    #[contractimpl]
    impl MockOracleContract {
        pub fn lastprice(_e: Env, _asset: Asset) -> Option<PriceData> {
            Some(PriceData {
                price: 100_000_000_000_000,
                timestamp: 0,
            })
        }

        pub fn decimals() -> u32 {
            14
        }
    }
}
