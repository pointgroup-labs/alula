use soroban_sdk::contractimport;

#[cfg(feature = "deploy")]
contractimport!(file = "../../wasms/deploy_optimized/proxy_swap.optimized.wasm");

#[cfg(not(feature = "deploy"))]
contractimport!(file = "../../wasms/mocks/proxy_swap_mock.wasm");
