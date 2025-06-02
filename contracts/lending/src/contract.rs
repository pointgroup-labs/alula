use {
    crate::{
        constants::{
            LCError, ACCRUAL_INIT, BPS_FACTOR, BPS_IN_PERCENT, DEFAULT_LIQUIDATION_THRESHOLD,
            HEALTH_FACTOR_THRESHOLD, REFLECTOR_TESTNET_ADDRESS,
        },
        interest_rate::CompoundRates,
        oracle,
        storage::{
            self, Accrual, BorrowObligation, DepositObligation, GlobalState, Obligation, Pool,
            PoolAddress, PoolConfig,
        },
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{
        contract, contractimpl, symbol_short, token, Address, BytesN, Env, String, Symbol, Vec,
    },
};

#[contract]
/// Lending Smart Contract. Allows users to lend and borrow other users' assets
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    /// Constructs the lending contract
    ///
    /// ### Arguments
    /// * `admin` - contract's administrator
    /// * `liquidation_threshold_percent` - threshold percentage used for health factor calculation
    pub fn __constructor(
        e: Env,
        admin: Address,
        liquidation_threshold_percent: Option<i128>,
    ) -> Result<(), LCError> {
        let liquidation_threshold_percent = if let Some(lt) = liquidation_threshold_percent {
            if lt <= 0 || lt > 100 {
                return Err(LCError::InvalidLiquidationThreshold);
            }

            lt
        } else {
            DEFAULT_LIQUIDATION_THRESHOLD
        };
        let liquidation_threshold_bps = liquidation_threshold_percent * BPS_IN_PERCENT;

        let global_state = GlobalState {
            admin,
            status: true,
            liquidation_threshold_bps,
        };

        storage::set_global_state(&e, &global_state);

        Ok(())
    }

    /// Gets the contract's global state
    ///
    /// ### Arguments
    pub fn get_global_state(e: Env) -> GlobalState {
        storage::get_global_state(&e)
    }

    /// Initializes a loan pool for a specific asset
    ///
    /// ### Arguments
    /// * `token_address` - address of a corresponding Soroban Asset Contract
    /// * `token_symbol` - symbol which represents a pool's token
    /// * `salt` - optional salt data, which when provided is used along with `token_address` to derive a deterministic pool address
    /// * `pool_config` - optional `PoolConfig` data. If not provided - a default pool config is used
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

        let accrual = Accrual {
            timestamp: e.ledger().timestamp(),
            borrow_accrual: ACCRUAL_INIT,
            deposit_accrual: ACCRUAL_INIT,
        };

        let pool = Pool {
            config,
            accrual,
            token_ticker,
            token_address,
            deposited: 0,
            borrowed: 0,
            collateral: 0,
        };

        storage::set_pool(&e, &pool_address, &pool)?;

        Ok(pool_address)
    }

    /// Deposits tokens into the loan pool
    ///
    /// ### Arguments
    /// * `user` - user which deposits a token
    /// * `pool_address` - address of a pool to which the deposit happens
    /// * `amount` - amount of tokens which are going to be deposited
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

        let mut obligation = storage::get_obligation(&e, &user).unwrap_or(Obligation::new(&e));

        obligation.accrue_interest(&e)?;

        let Some(Pool {
            token_address,
            accrual,
            ..
        }) = storage::get_pool(&e, &pool_address)
        else {
            return Err(LCError::PoolDoesNotExist);
        };

        let mut deposit_obligation =
            obligation
                .deposits
                .get(pool_address.clone())
                .unwrap_or(DepositObligation {
                    last_accrual: accrual.deposit_accrual,
                    ..Default::default()
                });

        deposit_obligation.deposited = deposit_obligation
            .deposited
            .checked_add(amount)
            .ok_or(LCError::OverOrUnderflow)?;

        obligation
            .deposits
            .set(pool_address.clone(), deposit_obligation);

        storage::adjust_pool_deposited(&e, &pool_address, amount)?;
        storage::set_obligation(&e, &user, &obligation);

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        Ok(())
    }

    /// Borrows tokens from the loan pool
    ///
    /// ### Arguments
    /// * `user` - user which borrows a token
    /// * `pool_address` - address of a pool from which the borrow happens
    /// * `amount` - amount of tokens which are going to be borrowed
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

        let mut obligation =
            storage::get_obligation(&e, &user).ok_or(LCError::ObligationDoesNotExist)?;

        obligation.accrue_interest(&e)?;

        let Some(Pool {
            token_address,
            accrual,
            deposited,
            borrowed,
            ..
        }) = storage::get_pool(&e, &pool_address)
        else {
            return Err(LCError::PoolDoesNotExist);
        };

        if amount > (deposited - borrowed) {
            return Err(LCError::NotEnoughPoolFunds);
        }

        let mut borrow_obligation =
            obligation
                .borrows
                .get(pool_address.clone())
                .unwrap_or(BorrowObligation {
                    last_accrual: accrual.borrow_accrual,
                    ..Default::default()
                });

        borrow_obligation.borrowed = borrow_obligation
            .borrowed
            .checked_add(amount)
            .ok_or(LCError::OverOrUnderflow)?;

        obligation
            .borrows
            .set(pool_address.clone(), borrow_obligation);

        if !obligation.is_healthy(&e)? {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
        }

        storage::adjust_pool_borrowed(&e, &pool_address, amount)?;
        storage::set_obligation(&e, &user, &obligation);

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    /// Deposits tokens into the loan pool as collateral only.
    /// This implies that they are always available for a healthy withdrawal for the
    /// cost of not accruing an interest rate
    ///
    /// ### Arguments
    /// * `user` - user which deposits a token
    /// * `pool_address` - address of a pool to which the collateral deposit happens
    /// * `amount` - amount of tokens which are being deposited as a collateral
    pub fn deposit_collateral(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveDeposit);
        }

        let mut obligation = storage::get_obligation(&e, &user).unwrap_or(Obligation::new(&e));

        obligation.accrue_interest(&e)?;

        let Some(Pool {
            token_address,
            accrual,
            ..
        }) = storage::get_pool(&e, &pool_address)
        else {
            return Err(LCError::PoolDoesNotExist);
        };

        let mut deposit_obligation =
            obligation
                .deposits
                .get(pool_address.clone())
                .unwrap_or(DepositObligation {
                    last_accrual: accrual.deposit_accrual,
                    ..Default::default()
                });

        deposit_obligation.collateral = deposit_obligation
            .collateral
            .checked_add(amount)
            .ok_or(LCError::OverOrUnderflow)?;

        obligation
            .deposits
            .set(pool_address.clone(), deposit_obligation);

        storage::adjust_pool_collateral(&e, &pool_address, amount)?;
        storage::set_obligation(&e, &user, &obligation);

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        Ok(())
    }

    /// Repays borrowed tokens
    ///
    /// ### Arguments
    /// * `user` - user which repays borrowed tokens
    /// * `pool_address` - address of a pool from which the borrow happened
    /// * `amount` - amount of repaid tokens
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

        let Pool { token_address, .. } =
            storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        let mut obligation =
            storage::get_obligation(&e, &user).ok_or(LCError::ObligationDoesNotExist)?;

        obligation.accrue_interest(&e)?;

        let mut borrow_obligation = obligation
            .borrows
            .get(pool_address.clone())
            .ok_or(LCError::BorrowDoesNotExist)?;

        let amount = i128::min(amount, borrow_obligation.borrowed);

        if amount == borrow_obligation.borrowed {
            obligation.borrows.remove(pool_address.clone());
        } else {
            borrow_obligation.borrowed = borrow_obligation
                .borrowed
                .checked_add(-amount)
                .ok_or(LCError::OverOrUnderflow)?;
            obligation
                .borrows
                .set(pool_address.clone(), borrow_obligation);
        }

        if obligation.is_empty() {
            // NB: This will never be hit because of the collateral required?
            storage::remove_obligation(&e, &user);
        } else {
            storage::set_obligation(&e, &user, &obligation);
        }

        storage::adjust_pool_borrowed(&e, &pool_address, -amount)?;

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        Ok(())
    }

    /// Liquidates borrower's position if position's health factor criterion isn't met
    ///
    /// ### Arguments
    /// * `user` - user which liquidates the borrower's position
    /// * `pool_address` - address of a pool whose tokens are repaid by the liquidator
    /// * `amount` - amount of repaid tokens
    pub fn liquidate(
        e: Env,
        user: Address,
        borrower: Address,
        pool_address: Address,
        amount: i128,
        // collateral_pool_address: Option<Address>, TODO: Add a possibility to choose which collateral the liquidator wants
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveLiquidation);
        }

        let Pool {
            token_address,
            token_ticker,
            config:
                PoolConfig {
                    close_factor_bps,
                    liquidation_spread_bps,
                    ..
                },
            ..
        } = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        let mut obligation =
            storage::get_obligation(&e, &borrower).ok_or(LCError::ObligationDoesNotExist)?;

        obligation.accrue_interest(&e)?;
        if obligation.is_healthy(&e)? {
            return Err(LCError::LiquidatedPositionIsHealthy);
        }

        let mut borrow_obligation = obligation
            .borrows
            .get(pool_address.clone())
            .ok_or(LCError::BorrowDoesNotExist)?;

        let liquidatable_bps = amount
            .fixed_div_ceil(borrow_obligation.borrowed, BPS_FACTOR)
            .ok_or(LCError::OverOrUnderflow)?;
        if liquidatable_bps > close_factor_bps {
            // TODO: What's the best way to set `close_factor_bps` value?
            return Err(LCError::LiquidationExceedsCloseFactor);
        }

        let borrowed_asset_price = get_asset_price(&e, &token_ticker)?;
        let liquidation_value = borrowed_asset_price
            .checked_mul(amount)
            .ok_or(LCError::OverOrUnderflow)?;
        let collateral_value_to_redeem = liquidation_value
            .fixed_mul_ceil(BPS_FACTOR + liquidation_spread_bps, BPS_FACTOR)
            .ok_or(LCError::OverOrUnderflow)?;

        let mut collateral_value_sum: i128 = 0;
        let mut collateral_values: Vec<i128> = Vec::new(&e);
        let mut collateral_prices: Vec<i128> = Vec::new(&e);

        for (collateral_pool_address, deposit_obligation) in obligation.deposits.iter() {
            let DepositObligation {
                collateral,
                deposited,
                ..
            } = deposit_obligation;

            let collateral_sum = deposited
                .checked_add(collateral)
                .ok_or(LCError::OverOrUnderflow)?;

            let collateral_token_ticker = storage::get_pool(&e, &collateral_pool_address)
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
        for (idx, (collateral_pool_address, mut deposit_obligation)) in
            obligation.deposits.iter().enumerate()
        {
            let idx = idx as u32;
            let deposited = deposit_obligation.deposited;
            let collateral_value = collateral_values.get(idx).unwrap(); // safe
            let collateral_token_price = collateral_prices.get(idx).unwrap(); // safe

            let value_ratio_bps = collateral_value
                .fixed_div_floor(collateral_value_sum, BPS_FACTOR)
                .ok_or(LCError::OverOrUnderflow)?;
            let amount_to_transfer_to_liquidator = value_ratio_bps
                .checked_mul(collateral_value_to_redeem)
                .ok_or(LCError::OverOrUnderflow)?
                / (BPS_FACTOR * collateral_token_price);

            let token_client = token::Client::new(&e, &collateral_pool_address);
            token_client.transfer(
                &e.current_contract_address(),
                &user,
                &amount_to_transfer_to_liquidator,
            );

            if deposited < amount_to_transfer_to_liquidator {
                let diff = amount_to_transfer_to_liquidator - deposited;

                deposit_obligation.collateral = deposit_obligation
                    .collateral
                    .checked_add(-diff)
                    .ok_or(LCError::OverOrUnderflow)?;
                deposit_obligation.deposited = deposit_obligation
                    .deposited
                    .checked_add(-deposited)
                    .ok_or(LCError::OverOrUnderflow)?;

                obligation
                    .deposits
                    .set(collateral_pool_address.clone(), deposit_obligation);
                storage::adjust_pool_deposited(&e, &collateral_pool_address, -deposited)?;
            } else {
                deposit_obligation.deposited = deposit_obligation
                    .deposited
                    .checked_add(-amount_to_transfer_to_liquidator)
                    .ok_or(LCError::OverOrUnderflow)?;

                obligation
                    .deposits
                    .set(collateral_pool_address.clone(), deposit_obligation);
                storage::adjust_pool_deposited(
                    &e,
                    &collateral_pool_address,
                    -amount_to_transfer_to_liquidator,
                )?;
            }
        }

        borrow_obligation.borrowed = borrow_obligation
            .borrowed
            .checked_add(-amount)
            .ok_or(LCError::OverOrUnderflow)?;
        obligation.borrows.set(pool_address, borrow_obligation);

        storage::set_obligation(&e, &borrower, &obligation);

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        Ok(())
    }

    /// Withdraws collateral tokens from the loan pool to the user
    ///
    /// ### Arguments
    /// * `user` - user which withdraws collateral tokens
    /// * `pool_address` - address of a pool from which the withdrawal happens
    /// * `amount` - amount of withdrawn tokens
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

        let Pool {
            token_address,
            collateral: pool_collateral,
            ..
        } = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        let mut obligation =
            storage::get_obligation(&e, &user).ok_or(LCError::ObligationDoesNotExist)?;

        obligation.accrue_interest(&e)?;

        let mut deposit_obligation = obligation
            .deposits
            .get(pool_address.clone())
            .ok_or(LCError::DepositDoesNotExist)?;

        if amount > deposit_obligation.collateral {
            return Err(LCError::WithdrawOverBalance);
        }

        if amount > pool_collateral {
            return Err(LCError::NotEnoughPoolFunds);
        }

        deposit_obligation.collateral = deposit_obligation
            .collateral
            .checked_add(-amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if deposit_obligation.deposited == 0 && deposit_obligation.collateral == 0 {
            obligation.deposits.remove(pool_address.clone());
        } else {
            obligation
                .deposits
                .set(pool_address.clone(), deposit_obligation);
        }

        if !obligation.is_healthy(&e)? {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold)?;
        }

        if obligation.is_empty() {
            storage::remove_obligation(&e, &user);
        } else {
            storage::set_obligation(&e, &user, &obligation);
        }

        storage::adjust_pool_collateral(&e, &pool_address, -amount)?;

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    /// Withdraws deposited tokens from the loan pool to the user
    ///
    /// ### Arguments
    /// * `user` - user which withdraws deposited tokens
    /// * `pool_address` - address of a pool from which the withdrawal happens
    /// * `amount` - amount of withdrawn tokens
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

        let Pool {
            token_address,
            deposited: pool_deposited,
            borrowed: pool_borrowed,
            ..
        } = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        let mut obligation =
            storage::get_obligation(&e, &user).ok_or(LCError::ObligationDoesNotExist)?;

        obligation.accrue_interest(&e)?;

        let mut deposit_obligation = obligation
            .deposits
            .get(pool_address.clone())
            .ok_or(LCError::DepositDoesNotExist)?;

        if amount > deposit_obligation.deposited {
            return Err(LCError::WithdrawOverBalance);
        }

        if amount > (pool_deposited - pool_borrowed) {
            return Err(LCError::NotEnoughPoolFunds);
        }

        deposit_obligation.deposited = deposit_obligation
            .deposited
            .checked_add(-amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if deposit_obligation.deposited == 0 && deposit_obligation.collateral == 0 {
            obligation.deposits.remove(pool_address.clone());
        } else {
            obligation
                .deposits
                .set(pool_address.clone(), deposit_obligation);
        }

        if !obligation.is_healthy(&e)? {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold)?;
        }

        if obligation.is_empty() {
            storage::remove_obligation(&e, &user);
        } else {
            storage::set_obligation(&e, &user, &obligation);
        }

        storage::adjust_pool_deposited(&e, &pool_address, -amount)?;

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    pub fn get_user_obligation(e: Env, user: Address) -> Result<Obligation, LCError> {
        if let Some(mut obligation) = storage::get_obligation(&e, &user) {
            obligation.accrue_interest(&e)?;
            storage::set_obligation(&e, &user, &obligation);

            Ok(obligation)
        } else {
            Err(LCError::ObligationDoesNotExist)
        }
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
        deposit_accrual,
        ..
    } = storage::accrue_interest(e, pool_address)?;

    let Obligation {
        mut borrows,
        mut deposits,
    } = storage::get_obligation(e, user).ok_or(LCError::ObligationDoesNotExist)?;

    let borrow_position = borrows.get(pool_address.clone());
    if let Some(mut position) = borrow_position {
        let amount = position.borrowed;
        let new_amount = amount
            .checked_mul(borrow_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(position.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        position.last_accrual = borrow_accrual;
        position.borrowed = new_amount;

        borrows.set(pool_address.clone(), position);
    }

    let deposit_position = deposits.get(pool_address.clone());
    if let Some(mut position) = deposit_position {
        let amount = position.deposited;
        let new_amount = amount
            .checked_mul(deposit_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(position.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        position.last_accrual = deposit_accrual;
        position.deposited = new_amount;

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
        let DepositObligation {
            deposited,
            collateral,
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
                    .checked_mul(deposited)
                    .ok_or(LCError::OverOrUnderflow)?,
            )
            .ok_or(LCError::OverOrUnderflow)?;
        collateral_sum_value = collateral_sum_value
            .checked_add(
                lastprice
                    .price
                    .checked_mul(collateral)
                    .ok_or(LCError::OverOrUnderflow)?,
            )
            .ok_or(LCError::OverOrUnderflow)?;
    }

    for (pool_address, borrow_position) in borrows {
        let BorrowObligation { borrowed, .. } = borrow_position;

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
                    .checked_mul(borrowed)
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
