use {
    crate::{
        constants::{
            LCError, ACCRUAL_INIT_VALUE, BPS_IN_PERCENT, DEFAULT_LIQUIDATION_THRESHOLD,
            HEALTH_FACTOR_THRESHOLD, REFLECTOR_TESTNET_ADDRESS,
        },
        interest_rate::CompoundRates,
        oracle,
        storage::{
            self, Accrual, GlobalState, Obligation, ObligationBorrow, ObligationDeposit, Pool,
            PoolAddress, PoolConfig,
        },
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{
        contract, contractimpl, symbol_short, token, Address, BytesN, Env, String, Symbol, Vec,
    },
};

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    pub fn __constructor(
        e: Env,
        admin: Address,
        liquidation_threshold: Option<i128>,
    ) -> Result<(), LCError> {
        let liquidation_threshold = if let Some(lt) = liquidation_threshold {
            if lt <= 0 || lt > 100 {
                return Err(LCError::InvalidLiquidationThreshold);
            }
            lt
        } else {
            DEFAULT_LIQUIDATION_THRESHOLD
        };
        let liquidation_threshold_bps = liquidation_threshold * BPS_IN_PERCENT;

        let global_state = GlobalState {
            admin,
            status: true,
            liquidation_threshold_bps,
        };

        storage::set_global_state(&e, &global_state);
        Ok(())
    }

    // TODO: Experiment more with oracle
    // We must be sure that the price is relevant and is updated as frequent as possible
    pub fn test_oracle_price(e: Env) -> i128 {
        let reflector_address =
            Address::from_string(&String::from_str(&e, REFLECTOR_TESTNET_ADDRESS));
        let _reflector_contract = oracle::Client::new(&e, &reflector_address);
        let eurc_ticker = symbol_short!("EURC");
        let _eurc_asset = oracle::Asset::Other(eurc_ticker);

        todo!()
    }

    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        token_ticker: Symbol, // NB: Token Interface contains a `.symbol()` endpoint, which can be used for retrieving a token's ticker
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<PoolAddress, LCError> {
        let pool_address: PoolAddress = if let Some(salt) = salt {
            // TODO: Check some other ways of deriving an address
            e.deployer()
                .with_address(token_address.clone(), salt)
                .deployed_address()
        } else {
            token_address.clone()
        };

        if storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolAlreadyExists);
        }

        let config = if let Some(config) = pool_config {
            if !config.is_valid() {
                return Err(LCError::InvalidLoanPoolConfig);
            }

            config
        } else {
            Default::default()
        };

        // Accrual increase depends on the interest rate, so it must be stored and updated separately for each loan pool
        let accrual = Accrual {
            timestamp: e.ledger().timestamp(),
            borrow_accrual: ACCRUAL_INIT_VALUE,
            supply_accrual: ACCRUAL_INIT_VALUE,
        };

        let pool = Pool {
            config,
            accrual,
            token_ticker,
            token_address,
            supply: 0,
            borrowed: 0,
            collateral: 0,
        };

        storage::set_pool(&e, &pool_address, &pool)?;

        Ok(pool_address)
    }

    pub fn deposit(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveDeposit);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        let Pool { token_address, .. } =
            storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        storage::adjust_pool_supply(&e, &pool_address, amount)?;
        storage::adjust_obligation_deposit(&e, &user, &pool_address, amount)?;

        add_interest_to_user_obligation(&e, &user, &pool_address)?;

        Ok(())
    }

    pub fn borrow(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveDeposit);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        let Pool {
            token_address,
            borrowed,
            supply,
            ..
        } = storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        if amount >= (supply - borrowed) {
            return Err(LCError::NotEnoughPoolFunds);
        }

        // TODO: Rename this, since misleading..
        storage::adjust_obligation_borrow(&e, &user, &pool_address, amount)?;
        storage::adjust_pool_borrowed(&e, &pool_address, amount)?;

        add_interest_to_user_obligation(&e, &user, &pool_address)?;

        if !is_user_obligation_healthy(&e, &user)? {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
        }

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    // TODO: Would also be good to separate `health_factor` into a separate method
    // and write unit test for it...

    pub fn repay(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveRepay);
        }

        add_interest_to_user_obligation(&e, &user, &pool_address)?;

        let Obligation { borrows, .. } =
            storage::get_obligation(&e, &user).ok_or(LCError::ObligationDoesNotExist)?;

        let token_address = storage::get_pool(&e, &pool_address)
            .ok_or(LCError::PoolDoesNotExist)?
            .token_address;

        let borrow_position = borrows
            .get(pool_address.clone())
            .ok_or(LCError::BorrowPositionDoesNotExistForUserInPool)?;
        let adjusting_amount = i128::min(amount, borrow_position.amount);

        if adjusting_amount == borrow_position.amount {
            storage::remove_obligation_borrow(&e, &user, &pool_address);
        } else {
            storage::adjust_obligation_borrow(&e, &user, &pool_address, -adjusting_amount)?;
        }
        storage::adjust_pool_borrowed(&e, &pool_address, -adjusting_amount)?;

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    pub fn deposit_collateral(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        // WARN: We have a lot of repeating code here...
        if amount <= 0 {
            return Err(LCError::NonPositiveDeposit);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        let Pool { token_address, .. } =
            storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        storage::adjust_pool_collateral(&e, &pool_address, amount)?;
        storage::adjust_obligation_collateral(&e, &user, &pool_address, amount)?;

        Ok(())
    }

    pub fn liquidate(
        e: Env,
        user: Address,
        borrower: Address,
        pool_address: Address,
        amount: i128,
        // collateral_pool_address: Option<Address>, TODO: Add a possibility to choose which collateral the liquidator wants
    ) -> Result<(), LCError> {
        user.require_auth();
        add_interest_to_user_obligation(&e, &borrower, &pool_address)?;

        if is_user_obligation_healthy(&e, &borrower)? {
            return Err(LCError::LiquidatedPositionIsHealthy);
        }

        let pool = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;
        let obligation =
            storage::get_obligation(&e, &borrower).ok_or(LCError::ObligationDoesNotExist)?;

        let borrowed_amount = obligation
            .borrows
            .get(pool_address.clone())
            .ok_or(LCError::DepositDoesNotExist)?
            .amount;

        let liquidatable_bps = amount
            .checked_mul(10_000)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(borrowed_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if liquidatable_bps > pool.config.close_factor_bps {
            // TODO: What's the best way to set this `close_factor_bps` value?
            return Err(LCError::LiquidationExceedsCloseFactor);
        }

        // Liquidator repays the provided amount of the debt
        let token_client = token::Client::new(&e, &pool.token_address);

        token_client.transfer(&user, &e.current_contract_address(), &amount);

        let borrowed_asset_price = get_asset_price(&e, &pool.token_ticker)?;

        let liquidation_value = borrowed_asset_price
            .checked_mul(amount)
            .ok_or(LCError::OverOrUnderflow)?;

        let collateral_value_to_redeem = liquidation_value
            .fixed_mul_ceil(10_000 + pool.config.liquidation_spread_bps, 10_000)
            .ok_or(LCError::OverOrUnderflow)?;

        // Gather the sum value of all deposits(both plain and collateral deposits)
        let mut collateral_value_sum: i128 = 0;
        let mut collateral_values: Vec<i128> = Vec::new(&e);
        let mut collateral_prices: Vec<i128> = Vec::new(&e);

        for (_pool_address, obligation_deposit) in obligation.deposits.iter() {
            let ObligationDeposit {
                collateral_amount,
                amount,
                ..
            } = obligation_deposit;

            let collateral_sum = amount
                .checked_add(collateral_amount)
                .ok_or(LCError::OverOrUnderflow)?;

            let collateral_token_ticker = storage::get_pool(&e, &pool_address)
                .expect("Pool must exist for a collateral asset")
                .token_ticker;
            let collateral_token_price = get_asset_price(&e, &collateral_token_ticker)?;

            let collateral_value = collateral_sum
                .checked_mul(collateral_token_price)
                .ok_or(LCError::OverOrUnderflow)?;

            collateral_values.push_back(collateral_value);
            collateral_prices.push_back(collateral_token_price);

            collateral_value_sum = collateral_value_sum
                .checked_add(collateral_value)
                .ok_or(LCError::OverOrUnderflow)?;
        }

        // Traverse second time and make corresponding collateral withdrawals
        for (idx, (pool_address, obligation_deposit)) in obligation.deposits.iter().enumerate() {
            let deposited_amount = obligation_deposit.amount;
            let idx = idx as u32;

            let collateral_value = collateral_values
                .get(idx)
                .expect("Element with given idx must be present");

            let collateral_token_price = collateral_prices
                .get(idx)
                .expect("Element with given idx must be present");

            let value_ratio_bps = collateral_value
                .checked_mul(10_000)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_div(collateral_value_sum)
                .ok_or(LCError::OverOrUnderflow)?;

            let amount_to_transfer_to_liquidator = value_ratio_bps
                .checked_mul(collateral_value_to_redeem)
                .ok_or(LCError::OverOrUnderflow)?
                / 10_000 // TODO: Comment + stop using constant + add more tests
                / collateral_token_price;

            let token_client = token::Client::new(&e, &pool_address);
            token_client.transfer(
                &e.current_contract_address(),
                &user,
                &amount_to_transfer_to_liquidator,
            );

            if deposited_amount < amount_to_transfer_to_liquidator {
                let diff = amount_to_transfer_to_liquidator - deposited_amount;

                storage::adjust_obligation_collateral(&e, &user, &pool_address, -diff)?;
                storage::adjust_pool_collateral(&e, &pool_address, -diff)?;

                storage::adjust_obligation_deposit(
                    &e,
                    &borrower,
                    &pool_address,
                    -deposited_amount,
                )?;
                storage::adjust_pool_supply(&e, &pool_address, -deposited_amount)?;
            } else {
                storage::adjust_obligation_deposit(
                    &e,
                    &borrower,
                    &pool_address,
                    -amount_to_transfer_to_liquidator,
                )?;
                storage::adjust_pool_supply(&e, &pool_address, -amount_to_transfer_to_liquidator)?;
            }
        }

        storage::adjust_obligation_borrow(&e, &user, &pool_address, -amount)?;
        storage::adjust_pool_borrowed(&e, &pool_address, -amount)?;

        Ok(())
    }

    pub fn withdraw_collateral(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveWithdraw);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        if !storage::deposit_exists(&e, &user, &pool_address)? {
            return Err(LCError::DepositDoesNotExist);
        }

        add_interest_to_user_obligation(&e, &user, &pool_address)?;

        let Pool {
            token_address,
            collateral,
            ..
        } = storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        if amount > collateral {
            return Err(LCError::NotEnoughPoolFunds);
        }

        storage::adjust_pool_collateral(&e, &pool_address, -amount)?;
        storage::adjust_obligation_collateral(&e, &user, &pool_address, -amount)?;

        if !is_user_obligation_healthy(&e, &user)? {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
        }

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    pub fn withdraw(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveWithdraw);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        if !storage::deposit_exists(&e, &user, &pool_address)? {
            return Err(LCError::DepositDoesNotExist);
        }

        add_interest_to_user_obligation(&e, &user, &pool_address)?;

        // TODO: Compute Health Factor, right??
        // You shouldn't be able to withdraw your collateral if it backs up an existing deposit

        let Pool {
            token_address,
            supply,
            borrowed,
            ..
        } = storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        if amount > (supply - borrowed) {
            return Err(LCError::NotEnoughPoolFunds);
        }

        storage::adjust_pool_supply(&e, &pool_address, -amount)?;
        storage::adjust_obligation_deposit(&e, &user, &pool_address, -amount)?;

        if !is_user_obligation_healthy(&e, &user)? {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
        }

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    pub fn get_user_obligation(e: Env, user: Address) -> Option<Obligation> {
        // TODO: Add interest to the obligation...
        // No need to show data which aren't true anymore
        storage::get_obligation(&e, &user)
    }

    pub fn get_pool(e: Env, pool_address: Address) -> Option<Pool> {
        storage::get_pool(&e, &pool_address)
    }

    pub fn get_apy(e: Env, pool_address: Address) -> Result<CompoundRates, LCError> {
        let pool = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        pool.get_apy()
    }

    pub fn accrue_interest(e: Env, pool_address: Address) -> Result<(), LCError> {
        // TODO: check for the pool's admin signature
        storage::accrue_interest(&e, &pool_address)?;

        Ok(())
    }

    // TODO: Write tests
    pub fn add_interest_to_user_obligation(
        e: Env,
        user: Address,
        pool_address: Option<Address>,
    ) -> Result<(), LCError> {
        if let Some(pool_address) = pool_address {
            add_interest_to_user_obligation(&e, &user, &pool_address)?;
        } else {
            let obligation = storage::get_obligation(&e, &user);

            if let Some(Obligation {
                borrows, deposits, ..
            }) = obligation
            {
                for (pool_address, _) in borrows {
                    add_interest_to_user_obligation(&e, &user, &pool_address)?;
                }

                for (pool_address, _) in deposits {
                    add_interest_to_user_obligation(&e, &user, &pool_address)?;
                }
            }
        }

        Ok(())
    }

    // TODO: Write tests as well
    pub fn get_health_factor(e: Env, user: Address) -> Result<i128, LCError> {
        compute_health_factor(&e, &user)
    }

    pub fn extend_instance_ttl(e: Env) {
        storage::extend_instance_storage(&e);
    }
}

fn add_interest_to_user_obligation(
    e: &Env,
    user: &Address,
    pool_address: &Address,
) -> Result<(), LCError> {
    let Accrual {
        borrow_accrual,
        supply_accrual,
        ..
    } = storage::accrue_interest(e, pool_address)?;

    let Obligation {
        mut borrows,
        mut deposits,
    } = storage::get_obligation(e, user).ok_or(LCError::ObligationDoesNotExist)?;

    let borrow_position = borrows.get(pool_address.clone());
    if let Some(mut position) = borrow_position {
        let amount = position.amount;
        let new_amount = amount
            .checked_mul(borrow_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(position.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        position.last_accrual = borrow_accrual;
        position.amount = new_amount;

        borrows.set(pool_address.clone(), position);
    }

    let deposit_position = deposits.get(pool_address.clone());
    if let Some(mut position) = deposit_position {
        let amount = position.amount;
        let new_amount = amount
            .checked_mul(supply_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(position.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        position.last_accrual = supply_accrual;
        position.amount = new_amount;

        deposits.set(pool_address.clone(), position);
    }

    let new_obligation = Obligation { deposits, borrows };
    storage::set_obligation(e, user, &new_obligation);

    Ok(())
}

fn compute_health_factor(e: &Env, user: &Address) -> Result<i128, LCError> {
    let Obligation {
        deposits, borrows, ..
    } = storage::get_obligation(e, user).ok_or(LCError::ObligationDoesNotExist)?;
    let GlobalState {
        liquidation_threshold_bps,
        ..
    } = storage::get_global_state(e);
    // HF = ((Collateral_Value1 + ... + Collateral_ValueN) * LT) / (Borrow_Value1 + ... + BorrowValueM)
    let reflector_address = Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
    let reflector_contract = oracle::Client::new(e, &reflector_address);
    let (mut collateral_sum_value, mut borrow_sum_value) = (0i128, 0i128);

    for (pool_address, deposit_position) in deposits {
        let ObligationDeposit {
            amount,
            collateral_amount,
            ..
        } = deposit_position;

        // TODO: Maybe, get it from the token client?
        // This will introduce an additional cross contract call, but will decrease the amount of errors,
        // which can happen.
        let ticker =
            storage::get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
        let asset = oracle::Asset::Other(ticker); // TODO: What about XLM?
        let lastprice = reflector_contract
            .lastprice(&asset)
            .ok_or(LCError::OracleDoesNotKnowAssetPrice)?;

        collateral_sum_value = collateral_sum_value
            .checked_add(
                lastprice
                    .price
                    .checked_mul(amount)
                    .ok_or(LCError::OverOrUnderflow)?,
            )
            .ok_or(LCError::OverOrUnderflow)?;
        collateral_sum_value = collateral_sum_value
            .checked_add(
                lastprice
                    .price
                    .checked_mul(collateral_amount)
                    .ok_or(LCError::OverOrUnderflow)?,
            )
            .ok_or(LCError::OverOrUnderflow)?;
    }

    for (pool_address, borrow_position) in borrows {
        let ObligationBorrow { amount, .. } = borrow_position;

        let ticker =
            storage::get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
        let asset = oracle::Asset::Other(ticker);
        let lastprice = reflector_contract
            .lastprice(&asset)
            .ok_or(LCError::OracleDoesNotKnowAssetPrice)?;
        borrow_sum_value = borrow_sum_value
            .checked_add(
                lastprice
                    .price
                    .checked_mul(amount)
                    .ok_or(LCError::OverOrUnderflow)?,
            )
            .ok_or(LCError::OverOrUnderflow)?;
    }

    if borrow_sum_value == 0 {
        // If nothing is borrowed - it's the healthiest obligation it can be
        return Ok(i128::MAX);
    }

    let numerator = collateral_sum_value
        .checked_mul(liquidation_threshold_bps)
        .ok_or(LCError::OverOrUnderflow)?;
    let health_factor = numerator
        .checked_div(borrow_sum_value)
        .ok_or(LCError::OverOrUnderflow)?;

    Ok(health_factor)
}

fn is_user_obligation_healthy(e: &Env, user: &Address) -> Result<bool, LCError> {
    Ok(compute_health_factor(e, user)? >= HEALTH_FACTOR_THRESHOLD)
}

fn get_asset_price(e: &Env, ticker: &Symbol) -> Result<i128, LCError> {
    let reflector_address = Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
    let reflector_contract = oracle::Client::new(e, &reflector_address);

    let asset = oracle::Asset::Other(ticker.clone());

    let lastprice = reflector_contract
        .lastprice(&asset)
        .ok_or(LCError::OracleDoesNotKnowAssetPrice)?;

    Ok(lastprice.price)
}

#[allow(unused)]
fn get_price_decimals(e: &Env) -> u32 {
    let reflector_address = Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
    let reflector_contract = oracle::Client::new(e, &reflector_address);

    reflector_contract.decimals()
}
