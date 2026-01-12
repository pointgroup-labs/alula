import type { ObligationUI } from '@alula/client-sdk'
import type { MultiplyAccountTableItem, MultiplyTableItem } from '~/types/table'
import Decimal from 'decimal.js'

export function checkIsHaveMultiply(
  obligations: ObligationUI,
  tableData: MultiplyTableItem[] | MultiplyAccountTableItem[],
  poolAddress: string,
  market: string,
) {
  const deposits: any = obligations[market]?.deposits ?? []
  const borrows: any = obligations[market]?.borrows ?? []
  const poolData = tableData?.find(item => item.pool_address === poolAddress && item.market === market)
  if (deposits.length === 0 || borrows.length === 0 || !poolData) {
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
  return borrowAvailableInUsd / (poolPrice * (selectedMultiplier - 1))
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
