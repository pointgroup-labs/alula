use soroban_sdk::contractimport;

// Integration tests compile the 'lending' crate as a dependency, hence #[cfg(not(test))] is not hit for them.
// TODO: Automate switching between `reflector_oracle` for deploy build and `reflector_oracle_mock`
// for integration tests build
contractimport!(file = "../../target/wasm32-unknown-unknown/release/reflector_oracle_mock.wasm");
