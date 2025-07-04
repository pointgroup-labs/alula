#![allow(clippy::too_many_arguments)] // Omitting Soroswap's clippy warnings

use soroban_sdk::contractimport;

#[cfg(feature = "deploy")]
contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/mocks/soroswap_router_mock.wasm");
