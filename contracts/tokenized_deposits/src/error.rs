use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TDError {
    // Core errors (0-99)
    InternalError = 0,
    OverOrUnderflow = 1,
    NegativeAmount = 2,
    NotInitialized = 3,
    InvalidInitialization = 4,

    // Authorization errors (100-199)
    NoPendingAdmin = 100,
    DepositsPaused = 101,

    // Balance and allowance errors (200-299)
    AllowanceExpired = 200,
    InsufficientBalance = 201,
    InsufficientAllowance = 202,

    // Vault errors (300-399)
    // A deposit converted to zero shares. Accepting it would silently donate the assets to
    // existing holders
    ZeroShares = 300,
    // A redemption converted to zero assets
    ZeroAssets = 301,
    // The requested deposit exceeds `max_deposit` (the market's supply cap)
    ExceedsMaxDeposit = 302, /* This must definitely panic. Okay, maybe panicking sometimes is good here */
    // The requested withdrawal exceeds `max_withdraw` (the owner's balance, or the liquidity the
    // market can currently honor)
    ExceedsMaxWithdraw = 303, // Same here
    // The market returned less than the vault asked for. Left unchecked this would let the vault
    // burn a holder's shares while paying out someone else's liquidity
    MarketReturnedLess = 304,
}
