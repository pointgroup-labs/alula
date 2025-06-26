use soroban_sdk::contractimport;

// #[cfg(feature = "deploy")]
contractimport!(file = "../../target/wasm32v1-none/release/soroswap-router.wasm");

// TODO: Move downloaded contracts into a separate folder or into mock contracts folder as well

// #[cfg(not(feature = "deploy"))]
// contractimport!(file = "../../wasms/soroswap_aggregator_mock.wasm");
