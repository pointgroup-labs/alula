use {
    crate::error::LendingContractError,
    soroban_sdk::{contracttype, Address, Env, Map},
};

#[contracttype]
pub struct GlobalState {
    pub admin: Address,
    pub status: bool,
    // Oracle address + ....
}

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(PoolAddress),
    Obligation(UserAddress),
}

#[contracttype]
pub struct Pool {
    pub token_address: Address, // <------------ this will be read frequently
    pub liquidation_threshold: i128,
    pub balance: i128, // <---- this will be changed frequently
}

// #[contracttype]
// pub struct ObligationBorrow {
//     pub pool_address: Address,
//     pub amount: i128,
//     // @TODO: Should there be some currency info?
// }

// #[contracttype]
// pub struct ObligationDeposit {
//     pub pool_address: Address,
//     pub amount: i128,
// }

#[contracttype]
pub struct Obligation {
    pub deposits: Map<PoolAddress, i128>,
    pub borrows: Map<PoolAddress, i128>,
}

#[allow(unused)]
pub(crate) fn read_global_state(e: &Env) -> GlobalState {
    e.storage().instance().get(&DataKey::GlobalState).unwrap() // @TODO: unwrap()
}

pub(crate) fn write_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);
}

// --- Pool ---
pub(crate) fn set_pool(
    e: &Env,
    pool_address: &PoolAddress,
    token_address: &Address,
    liquidation_threshold: i128,
) {
    e.storage().instance().set(
        &DataKey::Pool(pool_address.clone()),
        &Pool {
            liquidation_threshold,
            token_address: token_address.clone(),
            balance: 0,
        },
    )
}

pub(crate) fn pool_exists(e: &Env, pool_address: &PoolAddress) -> bool {
    e.storage()
        .instance()
        .has(&DataKey::Pool(pool_address.clone()))
}

pub(crate) fn get_pool_data(e: &Env, pool_address: &PoolAddress) -> Option<Pool> {
    e.storage()
        .instance()
        .get(&DataKey::Pool(pool_address.clone()))
}

pub(crate) fn set_pool_data(e: &Env, pool_address: &Address, pool_data: &Pool) {
    e.storage()
        .instance()
        .set(&DataKey::Pool(pool_address.clone()), pool_data);
}

pub(crate) fn adjust_pool_balance(
    e: &Env,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<(), LendingContractError> {
    let Pool {
        token_address,
        liquidation_threshold,
        balance,
    } = get_pool_data(e, pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;
    let new_balance = balance
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    let new_pool_data = Pool {
        token_address,
        liquidation_threshold,
        balance: new_balance,
    };
    set_pool_data(e, pool_address, &new_pool_data);

    Ok(())
}

// --- Obligation ---
pub(crate) fn set_obligation(e: &Env, user: &Address, obligation: &Obligation) {
    e.storage()
        .instance()
        .set(&DataKey::Obligation(user.clone()), obligation);
}

pub(crate) fn get_obligation(e: &Env, user: &Address) -> Option<Obligation> {
    e.storage()
        .instance()
        .get(&DataKey::Obligation(user.clone()))
}

pub(crate) fn adjust_deposit(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<i128, LendingContractError> {
    let Obligation {
        mut deposits,
        borrows,
    } = get_obligation(e, user).unwrap_or(Obligation {
        deposits: Map::new(e),
        borrows: Map::new(e),
    });
    let pool_obligation_deposit = deposits.get(pool_address.clone()).unwrap_or(0);
    let new_deposit_amount = pool_obligation_deposit
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    deposits.set(pool_address.clone(), new_deposit_amount);
    let new_obligation = Obligation { deposits, borrows }; // @TODO: this sucks a bit
    set_obligation(e, user, &new_obligation);

    Ok(new_deposit_amount)
}

pub(crate) fn deposit_exists(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
) -> Result<bool, LendingContractError> {
    let Obligation {
        deposits,
        borrows: _,
    } = get_obligation(e, user).ok_or(LendingContractError::MisslingObligation)?;

    Ok(deposits.contains_key(pool_address.clone()))
}
