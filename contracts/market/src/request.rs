use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, Map, Vec, contracttype, map as smap,
    token::{self, TokenClient},
};

use crate::{
    constants::BPS_FACTOR, error::MCError, events, math_utils::MathUtils,
    obligation::ObligationKey, pool::Pool,
};
#[contracttype]
pub struct StandardRequest {
    pub amount: i128,
    pub pool_address: Address,
}

#[contracttype]
pub struct SwapExactTokensRequest {
    pub swap_provider: Address,
    pub path: Vec<Address>,
    pub amount_in: i128,
    pub min_amount_out: i128,
}

#[contracttype]
pub struct SwapForExactTokensRequest {
    pub swap_provider: Address,
    pub path: Vec<Address>,
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
}

pub struct RequestTransfers<'a> {
    pub e: &'a Env,
    pub user: Address,
    pub referrer: Option<Address>,
    pub market_transfers: Map<Address, i128>,
    pub user_transfers: Map<Address, i128>,
    pub referrer_fee_transfers: Map<Address, i128>,
    // Records if flash repay must be made
    pub flash_borrow_request: Option<StandardRequest>,
}

impl<'a> RequestTransfers<'a> {
    pub fn new(
        e: &'a Env,
        user: Address,
        referrer: Option<Address>,
        market_transfers: Map<Address, i128>,
        user_transfers: Map<Address, i128>,
        referrer_fee_transfers: Map<Address, i128>,
        flash_borrow_request: Option<StandardRequest>,
    ) -> Self {
        Self {
            e,
            user,
            user_transfers,
            market_transfers,
            referrer,
            referrer_fee_transfers,
            flash_borrow_request,
        }
    }

    pub fn empty(e: &'a Env, user: Address) -> Self {
        Self { e, user, user_transfers: Map::new(e), market_transfers: Map::new(e), referrer: None, referrer_fee_transfers: Map::new(e), flash_borrow_request: None }
    }

    pub fn new_with_user_transfers(
        e: &'a Env,
        user: Address,
        referrer: Option<Address>,
        user_transfers: Map<Address, i128>,
        referrer_fee_transfers: Map<Address, i128>,
    ) -> Self {
        Self {
            e,
            user,
            referrer,
            user_transfers,
            market_transfers: smap![e],
            referrer_fee_transfers,
            flash_borrow_request: None,
        }
    }

    pub fn new_with_market_transfers(
        e: &'a Env,
        user: Address,
        referrer: Option<Address>,
        market_transfers: Map<Address, i128>,
        referrer_fee_transfers: Map<Address, i128>,
    ) -> Self {
        Self {
            e,
            user,
            referrer,
            market_transfers,
            user_transfers: smap![e],
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
        for (token_address, amount) in other.referrer_fee_transfers.iter() {
            let old = self.referrer_fee_transfers.get(token_address.clone()).unwrap_or(0);
            let new = old.checked_add(amount).map_over_or_underflow()?;
            self.referrer_fee_transfers.set(token_address, new);
        }

        if let Some(request) = other.flash_borrow_request {
            if self.flash_borrow_request.is_some() {
                return Err(MCError::FlashBorrowAlreadyRegistered);
            }

            self.flash_borrow_request = Some(request);
        }

        Ok(())
    }

    pub fn execute_transfers(self, e: &Env) -> Result<(), MCError> {
        self.execute_market_user_transfers(e)?;

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

    pub fn flush_transfers(&mut self, e: &Env) -> Result<(), MCError> {
        self.execute_market_user_transfers(e)?;
        self.user_transfers = smap![self.e];
        self.market_transfers = smap![self.e];
        self.referrer_fee_transfers = smap![self.e];
        Ok(())
    }

    fn execute_market_user_transfers(&self, e: &Env) -> Result<(), MCError> {
        for (token_address, amount) in self.user_transfers.iter() {
            TokenClient::new(self.e, &token_address).transfer(
                &self.user,
                self.e.current_contract_address(),
                &amount,
            );
        }

        for (token_address, amount) in self.market_transfers.iter() {
            TokenClient::new(self.e, &token_address).transfer(
                &self.e.current_contract_address(),
                &self.user,
                &amount,
            );
        }

        if let Some(referrer) = &self.referrer {
            for (token_address, amount) in self.referrer_fee_transfers.iter() {
                TokenClient::new(self.e, &token_address).transfer(
                    &self.e.current_contract_address(),
                    referrer,
                    &amount,
                );
            }
        } else if !self.referrer_fee_transfers.is_empty() {
            events::referrer_is_unexpectedly_missing(e);

            return Err(MCError::InternalError);
        }

        Ok(())
    }
}
