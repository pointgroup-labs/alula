use soroban_sdk::contractimport;

#[cfg(feature = "deploy")]
contractimport!(file = "../../wasms/downloads/reflector-oracle.wasm");

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/mocks/reflector_oracle_mock.wasm");
