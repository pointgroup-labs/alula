use soroban_sdk::contractimport;

// TODO: It's possible to use `sep-40-oracle` for a mock client.
// As an advantage - it contains a mock client that allows you to configure
// mock prices per different assets

#[cfg(feature = "deploy")]
contractimport!(file = "../../wasms/downloads/reflector-oracle.wasm");

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/mocks/reflector_oracle_mock.wasm");
