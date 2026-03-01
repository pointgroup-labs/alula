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
    pub referrer: Option<Address>,
    pub market_transfers: Map<Address, i128>,
    pub user_transfers: Map<Address, i128>,
    pub referrer_fee_transfers: Map<Address, i128>,
}

impl<'a> RequestTransfers<'a> {
    pub fn new(
        e: &'a Env,
        user: Address,
        referrer: Option<Address>,
        market_transfers: Map<Address, i128>,
        user_transfers: Map<Address, i128>,
        referrer_fee_transfers: Map<Address, i128>,
    ) -> Self {
        Self { e, user, user_transfers, market_transfers, referrer, referrer_fee_transfers }
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

        Ok(())
    }

    pub fn execute_transfers(self, e: &Env) -> Result<(), MCError> {
        for (token_address, amount) in self.user_transfers {
            TokenClient::new(self.e, &token_address).transfer(
                &self.user,
                self.e.current_contract_address(),
                &amount,
            );
        }

        for (token_address, amount) in self.market_transfers {
            TokenClient::new(self.e, &token_address).transfer(
                &self.e.current_contract_address(),
                &self.user,
                &amount,
            );
        }

        if let Some(referrer) = self.referrer {
            for (token_address, amount) in self.referrer_fee_transfers {
                TokenClient::new(self.e, &token_address).transfer(
                    &self.e.current_contract_address(),
                    &referrer,
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
