#![no_std]
use soroban_sdk::{Env, contract, contractevent, contractimpl, contracttype, symbol_short};

// Define the event using the `contractevent` attribute macro.
#[contractevent]
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct Increment {
    // Mark fields as topics, for the value to be included in the events topic list so
    // that downstream systems know to index it.
    #[topic]
    pub change: u32,
    // Fields not marked as topics will appear in the events data section.
    pub count: u32,
}

#[contracttype]
#[derive(Clone, Default, Debug, Eq, PartialEq)]
pub struct State {
    pub count: u32,
    pub last_incr: u32,
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Increment increments an internal counter, and returns the value.
    /// Publishes an event about the change in the counter.
    pub fn increment(env: Env, incr: u32) -> u32 {
        // Get the current count.
        let mut state = Self::get_state(env.clone());

        // Increment the count.
        state.count += incr;
        state.last_incr = incr;

        // Save the count.
        env.storage().persistent().set(&symbol_short!("STATE"), &state);

        // Publish an event about the change.
        Increment { change: incr, count: state.count }.publish(&env);

        // Return the count to the caller.
        state.count
    }

    /// Return the current state.
    pub fn get_state(env: Env) -> State {
        env.storage().persistent().get(&symbol_short!("STATE")).unwrap_or_default() // If no value set, assume 0.
    }
}

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.increment(&1), 1);
    assert_eq!(client.increment(&10), 11);
    assert_eq!(client.get_state(), State { count: 11, last_incr: 10 },);
}
