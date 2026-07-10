/* eslint-disable e18e/prefer-object-has-own */
import { MCError } from '@alula/market-sdk'

const errorMap: Record<string, string> = {
  InternalError: 'An unexpected internal error occurred.',
  PoolAlreadyExists: 'This pool already exists.',
  PoolDoesNotExist: 'No lending pool found with the provided address.',
  InvalidLoanPoolConfig: 'InvalidLoanPoolConfig',
  NotEnoughPoolFunds: 'The loan pool configuration is invalid or incomplete.',
  ObligationDoesNotExist: 'The specified obligation does not exist or has not been created.',
  DepositDoesNotExist: 'No deposit found.',
  NegativeDeposit: 'The deposit amount is negative and not allowed.”',
  NegativeWithdraw: 'The withdraw amount is negative and not permitted.',
  WithdrawOverBalance: 'Attempted to withdraw more than the available balance.',
  NegativeRepay: 'The repayment amount is negative and not allowed.',
  OverOrUnderflow: 'An arithmetic overflow or underflow was detected during calculation.',
  OracleDoesNotKnowAssetPrice: 'The oracle has no price data for the specified asset.',
  BorrowDoesNotExist: 'No borrow record found.',
  HealthFactorIsLowerThanRequiredThreshold: 'Health factor is below the required threshold. Please try entering a smaller borrow amount.',
  InvalidLiquidationThreshold: 'Invalid liquidation threshold.”',
  LiquidatedPositionIsHealthy: 'This position is healthy and cannot be liquidated.',
  LiquidationExceedsCloseFactor: 'Liquidation amount exceeds the allowed close factor.',
  NegativeLiquidation: 'The liquidation amount provided is negative and not allowed.',
  NegativeBorrow: 'The borrow amount is negative and not permitted.',
  CollateralPoolDoesNotExist: 'No collateral pool found with the provided address or ID.',
  NegativeFlashLoan: 'The flash loan amount provided is negative and not allowed.',
  InvalidTimestamp: 'The specified timestamp is incorrect or out of the valid range.',
  SelfLiquidation: 'You cannot liquidate your own position.',
  DepositPoolDoesNotExist: 'No deposit pool found with the provided address or ID.',
  BorrowPoolDoesNotExist: 'No borrow pool found with the provided address or ID.',
  InvalidLeverageMultiplier: 'Invalid leverage multiplier.',
  InvalidSwapSlippage: 'Invalid swap slippage value.',
  DependencyContractError: 'A dependent contract call failed or returned an error during execution.',
  SupplyLimitExceeded: 'Supply limit exceeded for this pool.',
  BorrowLimitExceeded: 'Borrow limit exceeded for this pool.',
  NegativeCollateralAddition: 'The collateral addition amount is negative and not permitted.',
  NegativeCollateralRemoval: 'The collateral removal amount is negative and not allowed.',
  OperationForbiddenOnPool: 'This operation is not available because the pool has active restrictions. Please check the pool status for details.',
}

function extractContractErrorIndex(input: string): string | null {
  const match = input.match(/Error\(Contract,\s*#(\d+)\)/)
  const contractErrorIndex = match ? Number(match[1]) : null
  if (typeof contractErrorIndex !== 'number') {
    return null
  }
  const index = Number(contractErrorIndex)
  if (Object.prototype.hasOwnProperty.call(MCError, index)) {
    const contractError = (MCError as Record<number, { message: string }>)[index]
    return contractError?.message ?? ''
  }
  return `Contract unknown error - ${index}`
}

export function getErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    const contractError = extractContractErrorIndex(error)
    return contractError ?? error
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

export function parseStellarError(error: unknown): string {
  if (!error) {
    return 'Unknown error'
  }

  const raw = getErrorMessage(error)
  console.log('%c[raw error]', 'color: #FB4747', raw)

  for (const key of Object.keys(errorMap)) {
    if (raw.includes(key)) {
      return String(errorMap[key])
    }
  }

  return raw
}
