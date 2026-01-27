use soroban_sdk::{Address, BytesN, Env, contractevent};

use crate::state::Farm;

#[contractevent]
struct InitializeFarm {
    #[topic]
    farm_id: u64,
    #[topic]
    farm_admin: Address,
    farm: Farm,
}

// -- Functions --

pub fn initialize_farm(e: &Env, farm: Farm) {
    InitializeFarm { farm_id: farm.id, farm_admin: farm.config.admin.clone(), farm }.publish(e);
}
