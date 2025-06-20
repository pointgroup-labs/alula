use soroban_sdk::contractimport;

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/flash_loan_utilizer_mock.wasm");
