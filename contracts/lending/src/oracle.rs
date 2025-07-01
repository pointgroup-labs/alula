use soroban_sdk::contractimport;

// #[cfg(feature = "deploy")]
contractimport!(file = "../../target/wasm32v1-none/release/reflector-oracle.wasm");

// #[cfg(not(feature = "deploy"))]
// contractimport!(file = "../../wasms/reflector_oracle_mock.wasm");
