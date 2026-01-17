use soroban_sdk::{Address, Env};

use crate::{error::PSCError, swap_trait::Swap};

pub struct SoroswapRouter(pub Address);

impl Swap for SoroswapRouter {
    fn swap_exact(
        &self,
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> Result<i128, PSCError> {
        todo!()
    }

    fn swap_for_exact(
        &self,
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        max_amount_in: i128,
        amount_out: i128,
    ) -> Result<i128, PSCError> {
        todo!()
    }
}
