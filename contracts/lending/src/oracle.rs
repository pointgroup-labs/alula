use soroban_sdk::contractimport;

#[cfg(feature = "deploy")]
contractimport!(file = "../../target/wasm32-unknown-unknown/release/reflector_oracle.wasm");

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/reflector_oracle_mock_new.wasm");
