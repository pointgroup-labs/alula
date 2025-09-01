use crate::MMError;
use lending::contract::LendingContractClient;
use soroban_sdk::{
    contract, contractclient, contractimpl, vec, Address, BytesN, Env, String, Symbol, Vec,
};

#[contractclient(name = "MarketManagerClient")]
pub trait MarketManager {
    fn deploy(e: Env, salt: BytesN<32>, admin: Address, name: Symbol) -> Result<Address, MMError>;

    fn get_market_list() -> Vec<Address>;
}

#[contract]
pub struct MarketManagerContract;

impl MarketManager for MarketManagerContract {
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address, /*config? */
        name: Symbol, // should it be String or Symbol? By the way, I don't still understand the proper difference...
    ) -> Result<Address, MMError> {
        // 2. We must write to the storage list/set?

        // 3. What about admin?

        // 4. We must issue `deploy`

        todo!()
    }

    fn get_market_list() -> Vec<Address> {
        todo!()
    }
}
