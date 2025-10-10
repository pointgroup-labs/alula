use soroban_sdk::{Address, Env, contractclient};

#[contractclient(name = "FlashLoanClient")]
pub trait ModErc3156 {
    fn exec_op(env: Env, caller: Address, token: Address, amount: i128, fee: i128);
}
