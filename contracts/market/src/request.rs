use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, Map, contracttype, map as smap,
    token::{self, TokenClient},
};

use crate::{
    constants::BPS_FACTOR, error::MCError, events, obligation::ObligationKey, pool::Pool,
    utils::MathUtils,
};

#[contracttype]
pub struct StandardRequest {
    // Should we keep 'user' here?
    pub amount: i128,
    pub pool_address: Address,
}

#[contracttype]
pub struct ModErc3156FlashLoanRequest {
    pub amount: i128,
    pub contract: Address,
    pub pool_address: Address,
}

#[contracttype]
pub struct SwapExactTokensRequest {
    pub swap_provider: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: i128,
    pub min_amount_out: i128,
}

#[contracttype]
pub struct SwapForExactTokensRequest {
    pub swap_provider: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub max_amount_in: i128,
    pub amount_out: i128,
}

#[contracttype]
pub struct LiquidateRequest {
    pub borrower_obligation_key: ObligationKey,
    pub borrow_pool_address: Address,
    pub collateral_pool_address: Address,
    pub repay_amount: i128,
    pub min_demanded_collateral_amount: i128,
}

#[contracttype]
pub enum Request {
    Deposit(StandardRequest),
    Borrow(StandardRequest),
    Withdraw(StandardRequest),
    Repay(StandardRequest),
    AddCollateral(StandardRequest),
    RemoveCollateral(StandardRequest),

    FlashBorrow(StandardRequest),
    SwapExactTokens(SwapExactTokensRequest),
    SwapForExactTokens(SwapForExactTokensRequest),

    Liquidate(LiquidateRequest),
    ModErc3156FlashLoan(ModErc3156FlashLoanRequest),
}

pub struct RequestTransfers<'a> {
    pub e: &'a Env,
    pub user: Address,
    pub market_transfers: Map<Address, i128>,
    pub user_transfers: Map<Address, i128>,
    pub referrer: Option<Address>,
    pub referrer_fee_transfers: Option<Map<Address, i128>>,
    // Records if flash repay must be made
    pub flash_borrow_request: Option<StandardRequest>,
}

impl<'a> RequestTransfers<'a> {
    pub fn new(
        e: &'a Env,
        user: Address,
        market_transfers: Map<Address, i128>,
        user_transfers: Map<Address, i128>,
        referrer: Option<Address>,
    ) -> Self {
        let referrer_fee_transfers = if referrer.is_some() { Some(smap![e]) } else { None };

        Self {
            e,
            user,
            user_transfers,
            market_transfers,
            referrer,
            referrer_fee_transfers,
            flash_borrow_request: None,
        }
    }

    pub fn empty(e: &'a Env, user: Address) -> Self {
        Self {
            e,
            user,
            user_transfers: Map::new(e),
            market_transfers: Map::new(e),
            referrer: None,
            referrer_fee_transfers: None,
            flash_borrow_request: None,
        }
    }

    pub fn new_with_flash_borrow_request(
        e: &'a Env,
        user: Address,
        flash_borrow_request: StandardRequest,
    ) -> Self {
        Self {
            e,
            user,
            user_transfers: Map::new(e),
            market_transfers: Map::new(e),
            referrer: None,
            referrer_fee_transfers: None,
            flash_borrow_request: Some(flash_borrow_request),
        }
    }

    pub fn new_with_user_transfers(
        e: &'a Env,
        user: Address,
        user_transfers: Map<Address, i128>,
        referrer: Option<Address>,
    ) -> Self {
        let referrer_fee_transfers = if referrer.is_some() { Some(smap![e]) } else { None };

        Self {
            e,
            user,
            user_transfers,
            market_transfers: smap![e],
            referrer,
            referrer_fee_transfers,
            flash_borrow_request: None,
        }
    }

    pub fn new_with_market_transfers(
        e: &'a Env,
        user: Address,
        market_transfers: Map<Address, i128>,
        referrer: Option<Address>,
    ) -> Self {
        let referrer_fee_transfers = if referrer.is_some() { Some(smap![e]) } else { None };

        Self {
            e,
            user,
            market_transfers,
            user_transfers: smap![e],
            referrer,
            referrer_fee_transfers,
            flash_borrow_request: None,
        }
    }

    pub fn merge(&mut self, other: RequestTransfers<'a>) -> Result<(), MCError> {
        // Merge Market Transfers (Market -> User)
        for (token_address, amount) in other.market_transfers.iter() {
            let old = self.market_transfers.get(token_address.clone()).unwrap_or(0);
            let new = old.checked_add(amount).map_over_or_underflow()?;
            self.market_transfers.set(token_address, new);
        }

        // Merge User Transfers (User -> Market)
        for (token_address, amount) in other.user_transfers.iter() {
            let old = self.user_transfers.get(token_address.clone()).unwrap_or(0);
            let new = old.checked_add(amount).map_over_or_underflow()?;
            self.user_transfers.set(token_address, new);
        }

        // Merge Referrer Fees (Market -> Referrer)
        if let Some(other_fees) = other.referrer_fee_transfers {
            let my_fees: &mut Map<Address, i128> =
                self.referrer_fee_transfers.get_or_insert_with(|| Map::new(self.e));

            for (token_address, amount) in other_fees.iter() {
                let old = my_fees.get(token_address.clone()).unwrap_or(0);
                let new = old.checked_add(amount).map_over_or_underflow()?;
                my_fees.set(token_address, new);
            }
        }

        if let Some(request) = other.flash_borrow_request {
            if self.flash_borrow_request.is_some() {
                return Err(MCError::InternalError);
            }

            self.flash_borrow_request = Some(request);
        }

        Ok(())
    }

    pub fn execute_transfers_final(mut self, e: &Env) -> Result<(), MCError> {
        self.execute_transfers(e)?;

        if let Some(StandardRequest { amount, pool_address }) = self.flash_borrow_request {
            let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;
            let token_client = token::Client::new(e, &pool.token_address);

            let flash_loan_fee = amount
                .fixed_mul_ceil(pool.config.fee_config.flash_loan_fee_bps as i128, BPS_FACTOR)
                .map_over_or_underflow()?;
            let repay_amount = amount.checked_add(flash_loan_fee).map_over_or_underflow()?;

            token_client.transfer(&self.user, e.current_contract_address(), &repay_amount);
            pool.adjust_total_available(e, amount)?;
            pool.adjust_operation_fees_sum(e, flash_loan_fee)?;

            pool.set(e);
        }

        Ok(())
    }

    pub fn execute_transfers(&mut self, e: &Env) -> Result<(), MCError> {
        for (token_address, amount) in self.user_transfers.iter() {
            let token_client = TokenClient::new(self.e, &token_address);
            token_client.transfer(&self.user, self.e.current_contract_address(), &amount);
        }

        for (token_address, amount) in self.market_transfers.iter() {
            let token_client = TokenClient::new(self.e, &token_address);
            token_client.transfer(&self.e.current_contract_address(), &self.user, &amount);
        }

        if let Some(referrer_fee_transfers) = &self.referrer_fee_transfers {
            let referrer = self.referrer.as_ref().ok_or_else(|| {
                events::referrer_is_unexpectedly_missing(e);

                MCError::InternalError
            })?;

            for (token_address, amount) in referrer_fee_transfers.iter() {
                let token_client = TokenClient::new(self.e, &token_address);
                token_client.transfer(&self.e.current_contract_address(), referrer, &amount);
            }
        }

        self.market_transfers = smap![e];
        self.user_transfers = smap![e];
        self.referrer_fee_transfers = None;

        Ok(())
    }
}
