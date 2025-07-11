const errorMap: Record<string, string> = {
    InternalError: 'InternalError',
    PoolAlreadyExists: 'PoolAlreadyExists',
    PoolDoesNotExist: 'PoolDoesNotExist',
    InvalidLoanPoolConfig: 'InvalidLoanPoolConfig',
    NotEnoughPoolFunds: 'NotEnoughPoolFunds',
    ObligationDoesNotExist: 'ObligationDoesNotExist',
    DepositDoesNotExist: 'DepositDoesNotExist',
    NegativeDeposit: 'NegativeDeposit',
    NegativeWithdraw: 'NegativeWithdraw',
    WithdrawOverBalance: 'WithdrawOverBalance',
    NegativeRepay: 'NegativeRepay',
    OverOrUnderflow: 'OverOrUnderflow',
    OracleDoesNotKnowAssetPrice: 'OracleDoesNotKnowAssetPrice',
    BorrowDoesNotExist: 'BorrowDoesNotExist',
    HealthFactorIsLowerThanRequiredThreshold: 'HealthFactorIsLowerThanRequiredThreshold',
    InvalidLiquidationThreshold: 'InvalidLiquidationThreshold',
    LiquidatedPositionIsHealthy: 'LiquidatedPositionIsHealthy',
    LiquidationExceedsCloseFactor: 'LiquidationExceedsCloseFactor',
    NegativeLiquidation: 'NegativeLiquidation',
    NegativeBorrow: 'NegativeBorrow',
    CollateralPoolDoesNotExist: 'CollateralPoolDoesNotExist',
    NegativeFlashLoan: 'NegativeFlashLoan',
    InvalidTimestamp: 'InvalidTimestamp',
    SelfLiquidation: 'SelfLiquidation',
    DepositPoolDoesNotExist: 'DepositPoolDoesNotExist',
    BorrowPoolDoesNotExist: 'BorrowPoolDoesNotExist',
    InvalidLeverageMultiplier: 'InvalidLeverageMultiplier',
    InvalidSwapSlippage: 'InvalidSwapSlippage',
    DependencyContractError: 'DependencyContractError',
    SupplyLimitExceeded: 'SupplyLimitExceeded',
    BorrowLimitExceeded: 'BorrowLimitExceeded',
    NegativeCollateralAddition: 'NegativeCollateralAddition',
    NegativeCollateralRemoval: 'NegativeCollateralRemoval',
}

export function getErrorMessage(error: unknown): string {
    if (typeof error === 'string') {
        return error
    }
    if (error instanceof Error) {
        return error.message
    }
    try {
        return JSON.stringify(error)
    } catch {
        return 'Unknown error'
    }
}

export function parseStellarError(error: unknown): string | undefined {
    const raw = getErrorMessage(error)

    for (const key in errorMap) {
        if (raw.includes(key)) {
            return String(errorMap[key])
        }
    }

    return String(error)
}
