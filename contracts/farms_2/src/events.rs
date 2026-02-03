use crate::state::{Farm, FarmConfig};
use soroban_sdk::{Address, Env, contractevent};

// -- Event structs --

#[contractevent]
struct InitializeFarm {
    #[topic]
    id: u64,
    #[topic]
    admin: Address,
    farm_config: FarmConfig,
}

// -- Emitting functions --

pub fn initialize_farm(e: &Env, farm: Farm) {
    InitializeFarm {
        id: farm.id,
        admin: farm.config.admin.clone(),
        farm_config: farm.config.clone(),
    }
    .publish(e);
}
