import type { Obligation } from '@alula/market-sdk'
import type { MultiplyTableItem } from '~/types/table'
import Decimal from 'decimal.js'

export function checkIsHaveMultiply(
  obligations: Record<string, Obligation>,
  tableData: MultiplyTableItem[],
  poolAddress: string,
  market: string,
) {
  const deposits: any = obligations[market]?.deposits ?? []
  const borrows: any = obligations[market]?.borrows ?? []
  const pool = tableData.find(item => item.pool_address === poolAddress && item.market === market)
  if (deposits.length === 0 || borrows.length === 0 || !pool) {
    return false
  }
  const depositPoolAddress = pool.depositPool.pool_address
  const borrowPoolAddress = pool.borrowPool.pool_address

  const isDeposits = deposits.some((deposit: any) => deposit.includes(depositPoolAddress))
  const isBorrows = borrows.some((deposit: any) => deposit.includes(borrowPoolAddress))
  return isDeposits && isBorrows
}

/**
 * @param ltvByBps — LTV в basis points (0…10000)
 * @returns number ≥1, max multiplyer
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
 * @param borrowAvailableInUsd - The amount of asset that can be borrowed in USD
 * @param poolPrice - The price of the asset in the pool
 * @param selectedMultiplier - The multiplier to use for the calculation
 * @returns The remaining amount of asset that can be supplied in USD
 */
export function calcRemainingMultiplyUSD(
  borrowAvailableInUsd: number,
  poolPrice: number,
  selectedMultiplier: number,
): number {
  if (selectedMultiplier <= 1) {
    return borrowAvailableInUsd
  }
  return borrowAvailableInUsd / poolPrice / selectedMultiplier
}
