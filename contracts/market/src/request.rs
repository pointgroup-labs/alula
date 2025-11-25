use soroban_sdk::{Address, Env, Map, contracttype, token::TokenClient};

use crate::{error::MCError, math_utils::MathUtils};

/// A request from the submission batch
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
    DepositIntoEarnObligation = 6,
    WithdrawFromEarnObligation = 7,
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
            6 => DepositIntoEarnObligation,
            7 => WithdrawFromEarnObligation,
            _ => return Err(MCError::IncorrectRequestType),
        };

        Ok(req_type)
    }
}

pub struct RequestTransfers<'a> {
    pub e: &'a Env,
    pub user: Address,
    pub market_transfers: Map<Address, i128>,
    pub user_transfers: Map<Address, i128>,
}

impl<'a> RequestTransfers<'a> {
    pub fn new(
        e: &'a Env,
        user: Address,
        market_transfers: Map<Address, i128>,
        user_transfers: Map<Address, i128>,
    ) -> Self {
        Self { e, user, user_transfers, market_transfers }
    }

    pub fn new_with_user_transfers(
        e: &'a Env,
        user: Address,
        user_transfers: Map<Address, i128>,
    ) -> Self {
        Self { e, user, user_transfers, market_transfers: Map::new(e) }
    }

    pub fn new_with_market_transfers(
        e: &'a Env,
        user: Address,
        market_transfers: Map<Address, i128>,
    ) -> Self {
        Self { e, user, market_transfers, user_transfers: Map::new(e) }
    }

    pub fn add_user_transfer(
        &mut self,
        token_address: &Address,
        amount: i128,
    ) -> Result<(), MCError> {
        let prev = self.user_transfers.get(token_address.clone()).unwrap_or(0);
        let new = prev.checked_add(amount).map_over_or_underflow()?;

        self.user_transfers.set(token_address.clone(), new);

        Ok(())
    }

    pub fn add_market_transfer(
        &mut self,
        token_address: &Address,
        amount: i128,
    ) -> Result<(), MCError> {
        let prev = self.market_transfers.get(token_address.clone()).unwrap_or(0);
        let new = prev.checked_add(amount).map_over_or_underflow()?;

        self.market_transfers.set(token_address.clone(), new);

        Ok(())
    }

    pub fn merge(&mut self, other: RequestTransfers<'a>) -> Result<(), MCError> {
        for (token_address, amount) in other.market_transfers.iter() {
            let old = self.market_transfers.get(token_address.clone()).unwrap_or_default();
            let new = old.checked_add(amount).map_over_or_underflow()?;

            self.market_transfers.set(token_address, new);
        }

        for (token_address, amount) in other.user_transfers.iter() {
            let old = self.user_transfers.get(token_address.clone()).unwrap_or_default();
            let new = old.checked_add(amount).map_over_or_underflow()?;

            self.user_transfers.set(token_address, new);
        }

        Ok(())
    }

    pub fn execute_transfers(self) {
        for (token_address, amount) in self.user_transfers {
            let token_client = TokenClient::new(self.e, &token_address);
            token_client.transfer(&self.user, self.e.current_contract_address(), &amount);
        }

        for (token_address, amount) in self.market_transfers {
            let token_client = TokenClient::new(self.e, &token_address);
            token_client.transfer(&self.e.current_contract_address(), &self.user, &amount);
        }
    }
}
