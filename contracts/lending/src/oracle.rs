use soroban_sdk::contractimport;

#[cfg(feature = "integration_tests")]
contractimport!(file = "../../target/wasm32-unknown-unknown/release/reflector_oracle_mock.wasm");

#[cfg(not(feature = "integration_tests"))]
contractimport!(file = "../../target/wasm32-unknown-unknown/release/reflector_oracle.wasm");
