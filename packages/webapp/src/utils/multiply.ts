import type { MultiplyObligationUI } from '~/store/user'
import type { MultiplyTableItem } from '~/types/table'
import { bpsToNumber } from '@alula/client-sdk'
import Decimal from 'decimal.js'
import { buildMultiplyPairKey } from '~/utils/obligation'

export function checkIsHaveMultiply(
  obligations: MultiplyObligationUI,
  tableData: MultiplyTableItem[],
  poolAddress: string,
  market: string,
) {
  const poolData = tableData?.find(item => item.pool_address === poolAddress && item.market === market)
  if (!poolData) {
    return false
  }

  const pairKey = poolData.pairKey || buildMultiplyPairKey(
    poolData.depositPoolData.pool.pool_address,
    poolData.borrowPoolData.pool.pool_address,
  )
  const obligation = obligations[market]?.[pairKey]
  const deposits: any = obligation?.deposits ?? []
  const borrows: any = obligation?.borrows ?? []

  if (deposits.length === 0 || borrows.length === 0) {
    return false
  }

  const depositPoolAddress = poolData.depositPoolData.pool.pool_address
  const borrowPoolAddress = poolData.borrowPoolData.pool.pool_address

  const isDeposits = deposits.some((deposit: any) => deposit.includes(depositPoolAddress))
  const isBorrows = borrows.some((deposit: any) => deposit.includes(borrowPoolAddress))
  return isDeposits && isBorrows
}

/**
 * @param ltvByBps — LTV в basis points (0…10000)
 * @returns number ≥1, max multiplier
 */
export function calculateMaxMultiplierFromBps(ltvByBps: number): number {
  if (!Number.isInteger(ltvByBps) || ltvByBps < 0 || ltvByBps >= 10_000) {
    throw new Error(`ltvByBps must be integer in [0,10000), got ${ltvByBps}`)
  }
  const openLtv = new Decimal(ltvByBps).div(10_000)
  return openLtv.eq(1)
    ? Infinity
    : new Decimal(1).div(new Decimal(1).minus(openLtv)).toNumber()
}

/**
 * Calculate the remaining amount of asset that can be supplied to a pool based on the provided parameters.
 * This function calculates the maximum deposit amount (in tokens) considering:
 * - Available borrow liquidity in USD
 * - Deposit token price
 * - Leverage multiplier
 * - Flash loan fee
 *
 * Formula (simplified, assuming 1:1 swap):
 * maxDeposit = borrowAvailableInUsd / (depositTokenPrice * (multiplier - 1) * (1 + flashLoanFee))
 *
 * @param borrowAvailableInUsd - The amount of asset that can be borrowed in USD
 * @param depositTokenPrice - The price of the deposit token in USD
 * @param selectedMultiplier - The multiplier to use for the calculation (e.g., 2.5)
 * @param flashLoanFeeBps - Flash loan fee in basis points (e.g., 50 = 0.5%)
 * @returns The maximum amount of deposit tokens that can be supplied
 */
export function calcRemainingMultiplyUSD(
  borrowAvailableInUsd: number,
  depositTokenPrice: number,
  selectedMultiplier: number,
  flashLoanFeeBps: number = 0,
): number {
  if (selectedMultiplier <= 1) {
    // If multiplier is 1 or less, no leverage is used
    return Infinity
  }

  // Calculate flash loan fee multiplier (1 + fee)
  const flashLoanFeeMultiplier = 1 + bpsToNumber(flashLoanFeeBps)
  // Calculate max deposit considering:
  // - User deposits D tokens
  // - Contract borrows D * (M - 1) * depositTokenPrice in USD
  // - With flash loan fee: D * (M - 1) * depositTokenPrice * (1 + fee)
  // - Available borrow in USD must be >= D * (M - 1) * depositTokenPrice * (1 + fee)
  // - Therefore: D <= borrowAvailableInUsd / ((M - 1) * depositTokenPrice * (1 + fee))
  return borrowAvailableInUsd / (depositTokenPrice * (selectedMultiplier - 1) * flashLoanFeeMultiplier)
}

export function calculateCurrentMultiplier(
  deposited: number,
  depositedPrice: number,
  borrowed: number,
  borrowedPrice: number,
) {
  const totalValue = deposited * depositedPrice
  const initialValue = totalValue - borrowed * borrowedPrice

  if (initialValue <= 0) {
    return Infinity
  }

  return totalValue / initialValue
}

export function calcMultiplyObligationNetApy({
  suppliedUsd,
  borrowedUsd,
  supplyApy,
  borrowApy,
}: {
  suppliedUsd: number
  borrowedUsd: number
  supplyApy: number
  borrowApy: number
}) {
  if (suppliedUsd <= 0) {
    return 0
  }

  const equity = suppliedUsd - borrowedUsd

  if (equity <= 0) {
    return 0
  }

  const multiplier = suppliedUsd / equity

  return supplyApy * multiplier - borrowApy * (multiplier - 1)
}
