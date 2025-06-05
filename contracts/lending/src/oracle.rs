use soroban_sdk::contractimport;

#[cfg(feature = "testing")]
contractimport!(file = "../../target/wasm32-unknown-unknown/release/reflector_oracle_mock.wasm");

#[cfg(not(feature = "testing"))]
contractimport!(file = "../../target/wasm32-unknown-unknown/release/reflector_oracle.wasm");
