use soroban_sdk::{Address, Env, contractclient};

// SEP-56 Tokenized Vault interface. Deviations from the spec are called out at
// the implementation site
#[contractclient(name = "TokenizedVaultClient")]
pub trait TokenizedVault {
    // Returns the total number of shares in circulation
    fn total_supply(e: Env) -> i128;

    // Returns the address of the underlying asset the vault manages
    fn query_asset(e: Env) -> Address;

    // Returns the total amount of underlying assets backing the outstanding shares
    fn total_assets(e: Env) -> i128;

    // Converts assets to shares at the current rate, rounded down
    fn convert_to_shares(e: Env, assets: i128) -> i128;

    // Converts shares to assets at the current rate, rounded down
    fn convert_to_assets(e: Env, shares: i128) -> i128;

    // Maximum assets that can currently be deposited for `receiver`
    fn max_deposit(e: Env, receiver: Address) -> i128;

    // Shares that would be minted for `assets`, rounded down
    fn preview_deposit(e: Env, assets: i128) -> i128;

    // Deposits `assets` and mints shares to `receiver`. Returns the shares minted
    fn deposit(e: Env, assets: i128, receiver: Address, from: Address, operator: Address) -> i128;

    // Maximum shares that can currently be minted for `receiver`
    fn max_mint(e: Env, receiver: Address) -> i128;

    // Assets required to mint exactly `shares`, rounded up
    fn preview_mint(e: Env, shares: i128) -> i128;

    // Mints exactly `shares` to `receiver`. Returns the assets consumed
    fn mint(e: Env, shares: i128, receiver: Address, from: Address, operator: Address) -> i128;

    // Maximum assets `owner` can currently withdraw
    fn max_withdraw(e: Env, owner: Address) -> i128;

    // Shares that would be burned to withdraw `assets`, rounded up
    fn preview_withdraw(e: Env, assets: i128) -> i128;

    // Withdraws exactly `assets` to `receiver`, burning `owner`'s shares. Returns shares burned
    fn withdraw(e: Env, assets: i128, receiver: Address, owner: Address, operator: Address)
    -> i128;

    // Maximum shares `owner` can currently redeem
    fn max_redeem(e: Env, owner: Address) -> i128;

    // Assets that would be received for `shares`, rounded down
    fn preview_redeem(e: Env, shares: i128) -> i128;

    // Redeems `shares` for assets to `receiver`. Returns the assets received
    fn redeem(e: Env, shares: i128, receiver: Address, owner: Address, operator: Address) -> i128;
}
