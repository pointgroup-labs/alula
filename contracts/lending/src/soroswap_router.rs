#![allow(clippy::too_many_arguments)] // Omitting Soroswap's clippy warnings

use soroban_sdk::contractimport;

#[cfg(feature = "deploy")]
contractimport!(file = "../../target/wasm32v1-none/release/soroswap-router.wasm");

// TODO: Maybe move downloaded contracts into a separate or mock contracts directory
#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/soroswap_router_mock.wasm");
