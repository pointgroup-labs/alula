use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TDError {
    // Core errors (0-99)
    OverOrUnderflow = 1,
    NegativeAmount = 2,
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
    // The requested withdrawal exceeds what the market can currently honor for the owner --
    // either their balance, the pool's liquidity, or their obligation's health headroom
    ExceedsMaxWithdraw = 303,
    // The pool's `jToken` rate could not be read. The rate is recovered from a withdrawal
    // simulation, so it is unavailable while the vault holds no position at all
    RateUnavailable = 305,
    // A transfer would leave the sender's obligation unhealthy. Shares encumbered by a borrow
    // cannot be moved -- the market's own health check is the authority here
    TransferWouldBeUnhealthy = 306,
}
