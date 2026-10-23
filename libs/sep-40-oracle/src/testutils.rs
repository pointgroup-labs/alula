mod wasm {
    soroban_sdk::contractimport!(file = "../../wasms/downloads/sep40-mock-oracle.wasm");
}

pub use wasm::{Asset, Client as MockPriceOracleClient, WASM as MockPriceOracleWASM};
