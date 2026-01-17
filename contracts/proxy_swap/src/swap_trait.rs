use enum_dispatch::enum_dispatch;
use soroban_sdk::{Address, Env};

use crate::{SoroswapRouter, aqua_router::AquaRouter, error::PSCError};

#[enum_dispatch]
pub trait Swap {
    fn swap_exact(
        &self,
        e: &Env,
        user: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> Result<i128, PSCError>;

    fn swap_for_exact(
        &self,
        e: &Env,
        user: &Address,
        token_in: &Address,
        token_out: &Address,
        max_amount_in: i128,
        amount_out: i128,
    ) -> Result<i128, PSCError>;
}

#[enum_dispatch(Swap)]
pub enum SwapProvider {
    SoroswapRouter(SoroswapRouter),
    AquaRouter(AquaRouter),
}
