#![no_std]

pub mod mock_oracle {
    use soroban_sdk::{Address, Env, Symbol, contract, contractimpl, contracttype};

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
        pub fn lastprice(e: Env, _asset: Asset) -> Option<PriceData> {
            Some(PriceData {
                price: 100_000_000_000_000,
                timestamp: e.ledger().timestamp(),
            })
        }

        pub fn decimals() -> u32 {
            14
        }
    }
}
