#![cfg(test)]

use market_manager::{
    contract::{MarketManagerClient, MarketManagerContract, MarketManagerContractClient},
    storage::Config,
};
use soroban_sdk::{Address, BytesN, Env, String, testutils::Address as _};

use crate::get_default_env;

mod market {
    use market::{
        LCError,
        pool::{PoolAddress, UserAddress},
        storage::{BorrowPoolAddress, DepositPoolAddress},
    };
    use soroban_sdk::contractimport;

    contractimport!(file = "/home/sonny_m00re/src/jpool/jlending/wasms/market.wasm");
}

struct ManagerSetup<'a> {
    e: Env,
    manager_client: MarketManagerClient<'a>,
    manager_address: Address,
    manager_admin: Address,
}

impl<'a> ManagerSetup<'a> {
    fn new() -> Self {
        let e = get_default_env();

        let manager_admin = Address::generate(&e);
        let market_contract_wasm_hash = e.deployer().upload_contract_wasm(market::WASM);

        let config = Config {
            admin: manager_admin.clone(),
            market_contract_wasm_hash,
        };

        let manager_address = e.register(MarketManagerContract, (config,));
        let manager_client = MarketManagerClient::new(&e, &manager_address);

        Self {
            e,
            manager_client,
            manager_address,
            manager_admin,
        }
    }
}

#[test]
fn test_manager_no_pools_after_deployment() {
    let ManagerSetup { manager_client, .. } = ManagerSetup::new();

    let pool_addresses = manager_client.get_market_list();

    assert!(pool_addresses.is_none());
}

#[test]
fn test_manager_deploy_pool() {
    let ManagerSetup {
        e, manager_client, ..
    } = ManagerSetup::new();

    let salt = BytesN::from_array(&e, &[0; 32]);
    let pool_admin = Address::generate(&e);
    let oracle = Address::generate(&e);

    let name = String::from_str(&e, "pool_1");

    manager_client.deploy(&salt, &pool_admin, &name, &oracle);
}
