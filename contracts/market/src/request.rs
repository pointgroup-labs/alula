use soroban_sdk::{Address, Env, Map, contracttype, map as smap, token::TokenClient};

use crate::{error::MCError, events, math_utils::MathUtils};

// A request from the submission batch
#[contracttype]
pub struct Request {
    pub request_type: u32,
    pub pool_address: Address,
    pub amount: i128,
}

#[contracttype]
pub enum RequestType {
    Deposit = 0,
    Borrow = 1,
    Withdraw = 2,
    Repay = 3,
    AddCollateral = 4,
    RemoveCollateral = 5,
    // TODO: Liquidate, Leverage, Flash Loan ...
}

impl TryFrom<u32> for RequestType {
    type Error = MCError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        use RequestType::*;

        let req_type = match value {
            0 => Deposit,
            1 => Borrow,
            2 => Withdraw,
            3 => Repay,
            4 => AddCollateral,
            5 => RemoveCollateral,
            _ => return Err(MCError::IncorrectRequestType),
        };

        Ok(req_type)
    }
}

impl From<RequestType> for u32 {
    fn from(value: RequestType) -> Self {
        value as u32 // safe
    }
}

pub struct RequestTransfers<'a> {
    pub e: &'a Env,
    pub user: Address,
    pub market_transfers: Map<Address, i128>,
    pub user_transfers: Map<Address, i128>,
    pub referrer: Option<Address>,
    pub referrer_fee_transfers: Option<Map<Address, i128>>,
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

        Self { e, user, user_transfers, market_transfers, referrer, referrer_fee_transfers }
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
        }
    }

    pub fn merge(&mut self, other: RequestTransfers<'a>) -> Result<(), MCError> {
        // 1. Merge Market Transfers (Market -> User)
        for (token_address, amount) in other.market_transfers.iter() {
            let old = self.market_transfers.get(token_address.clone()).unwrap_or(0);
            let new = old.checked_add(amount).map_over_or_underflow()?;
            self.market_transfers.set(token_address, new);
        }

        // 2. Merge User Transfers (User -> Market)
        for (token_address, amount) in other.user_transfers.iter() {
            let old = self.user_transfers.get(token_address.clone()).unwrap_or(0);
            let new = old.checked_add(amount).map_over_or_underflow()?;
            self.user_transfers.set(token_address, new);
        }

        // 3. Merge Referrer Fees (Market -> Referrer)
        if let Some(other_fees) = other.referrer_fee_transfers {
            let my_fees: &mut Map<Address, i128> =
                self.referrer_fee_transfers.get_or_insert_with(|| Map::new(self.e));

            for (token_address, amount) in other_fees.iter() {
                let old = my_fees.get(token_address.clone()).unwrap_or(0);
                let new = old.checked_add(amount).map_over_or_underflow()?;
                my_fees.set(token_address, new);
            }
        }

        Ok(())
    }

    pub fn execute_transfers(self, e: &Env) -> Result<(), MCError> {
        // TODO: Will re-using token clients here improve the performance?

        for (token_address, amount) in self.user_transfers {
            let token_client = TokenClient::new(self.e, &token_address);
            token_client.transfer(&self.user, self.e.current_contract_address(), &amount);
        }

        for (token_address, amount) in self.market_transfers {
            let token_client = TokenClient::new(self.e, &token_address);
            token_client.transfer(&self.e.current_contract_address(), &self.user, &amount);
        }

        if let Some(referrer_fee_transfers) = self.referrer_fee_transfers {
            let referrer = self.referrer.ok_or_else(|| {
                events::referrer_is_unexpectedly_missing(e);

                MCError::InternalError
            })?;

            for (token_address, amount) in referrer_fee_transfers {
                let token_client = TokenClient::new(self.e, &token_address);
                token_client.transfer(&self.e.current_contract_address(), &referrer, &amount);
            }
        }

        Ok(())
    }
}
