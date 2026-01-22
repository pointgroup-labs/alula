use soroban_sdk::{Address, Env, vec};

use crate::{error::PSCError, swap_trait::Swap};

#[allow(clippy::module_inception)]
#[allow(clippy::too_many_arguments)] // Omitting Soroswap's clippy warnings
mod soroswap_router {
    use soroban_sdk::contractimport;

    #[cfg(feature = "deploy")]
    contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");

    #[cfg(not(feature = "deploy"))]
    contractimport!(file = "../../wasms/mocks/soroswap_router_mock.wasm");
}

pub struct SoroswapRouter(pub Address);

impl Swap for SoroswapRouter {
    fn get_amount_out(
        &self,
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
    ) -> Result<i128, PSCError> {
        let router_client = soroswap_router::Client::new(e, &self.0);
        let path = vec![e, token_in.clone(), token_out.clone()];

        let amounts_out = router_client.router_get_amounts_out(&amount_in, &path);
        let amount_out = amounts_out.last().ok_or(PSCError::DependencyContractError)?;

        Ok(amount_out)
    }

    fn get_amount_in(
        &self,
        e: &Env,
        token_in: &Address,
        token_out: &Address,
        amount_out: i128,
    ) -> Result<i128, PSCError> {
        let router_client = soroswap_router::Client::new(e, &self.0);
        let path = vec![e, token_in.clone(), token_out.clone()];

        let amounts_in = router_client.router_get_amounts_in(&amount_out, &path);
        let amount_in = amounts_in.first().ok_or(PSCError::DependencyContractError)?;

        Ok(amount_in)
    }

    fn swap_exact(
        &self,
        e: &Env,
        to: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> Result<i128, PSCError> {
        let router_client = soroswap_router::Client::new(e, &self.0);
        let path = vec![e, token_in.clone(), token_out.clone()];

        let swap_amounts = router_client.swap_exact_tokens_for_tokens(
            &amount_in,
            &min_amount_out,
            &path,
            to,
            &u64::MAX,
        );
        let received_amount = swap_amounts.last().ok_or(PSCError::DependencyContractError)?;

        Ok(received_amount)
    }

    fn swap_for_exact(
        &self,
        e: &Env,
        to: &Address,
        token_in: &Address,
        token_out: &Address,
        max_amount_in: i128,
        amount_out: i128,
    ) -> Result<i128, PSCError> {
        let router_client = soroswap_router::Client::new(e, &self.0);
        let path = vec![e, token_in.clone(), token_out.clone()];

        let swap_amounts = router_client.swap_tokens_for_exact_tokens(
            &amount_out,
            &max_amount_in,
            &path,
            to,
            &u64::MAX,
        );
        let spent_amount = swap_amounts.first().ok_or(PSCError::DependencyContractError)?;

        Ok(spent_amount)
    }
}
