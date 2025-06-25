use soroban_sdk::contractimport;

#[cfg(feature = "deploy")]
contractimport!(file = "../../target/wasm32-unknown-unknown/release/soroswap_aggregator.wasm");

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/soroswap_aggregator_mock.wasm");
