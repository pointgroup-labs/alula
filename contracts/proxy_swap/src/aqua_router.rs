use soroban_sdk::{Address, Env};

use crate::{error::PSCError, swap_trait::Swap};

#[allow(unused)]
pub struct AquaRouter(pub Address);

impl Swap for AquaRouter {
    fn swap_exact(
        &self,
        _e: &Env,
        _to: &Address,
        _token_in: &Address,
        _token_out: &Address,
        _amount_in: i128,
        _min_amount_out: i128,
    ) -> Result<i128, PSCError> {
        todo!()
    }

    fn swap_for_exact(
        &self,
        _e: &Env,
        _to: &Address,
        _token_in: &Address,
        _token_out: &Address,
        _max_amount_in: i128,
        _amount_out: i128,
    ) -> Result<i128, PSCError> {
        todo!()
    }
}
