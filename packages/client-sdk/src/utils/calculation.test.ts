import type { PoolData } from '@alula/market-sdk'
import type { ObligationArray } from '../types'
import { describe, expect, it } from 'vitest'
import { calcUserTotalBorrowedInUsd, calcUserTotalStakeInUsd } from './calculation'

const ORACLE_DECIMALS = 14
const ONE_USD = 10n ** BigInt(ORACLE_DECIMALS)

function poolData(address: string, oracleAssetPrice: bigint): PoolData {
  return {
    d_token_rate_ceil_bps: 10000n,
    oracle_asset_price: oracleAssetPrice,
    total_available_adjusted: 0n,
    pool: {
      pool_address: address,
      token_decimals: 7,
      total_j_tokens: 0n,
      total_borrowed: 0n,
      config: { health_config: { open_ltv_bps: 8000, close_ltv_bps: 9000 } },
    },
  } as unknown as PoolData
}

function obligation(deposits: Array<[string, bigint]>, borrows: Array<[string, bigint]>): ObligationArray {
  return {
    deposits: deposits.map(([pool, collateral]) => [pool, { collateral, j_tokens: 0n }]),
    borrows: borrows.map(([pool, d_tokens]) => [pool, { d_tokens }]),
  } as unknown as ObligationArray
}

describe('calcUserTotalBorrowedInUsd', () => {
  it('totals debt when every leg is priced', () => {
    const pools = [poolData('POOL_A', ONE_USD), poolData('POOL_B', ONE_USD)]
    const result = calcUserTotalBorrowedInUsd(obligation([], [['POOL_A', 100_0000000n], ['POOL_B', 100_0000000n]]), pools, ORACLE_DECIMALS)

    expect(result.priced).toBe(true)
    expect(result.priced && result.usd).toBe(200)
  })

  // The defect: the unpriced leg used to be skipped, reporting 100 instead of 200 and
  // making the position read as half as indebted — i.e. safer — than it is.
  it('withholds the total when a debt leg has no oracle price', () => {
    const pools = [poolData('POOL_A', ONE_USD), poolData('POOL_B', 0n)]
    const result = calcUserTotalBorrowedInUsd(obligation([], [['POOL_A', 100_0000000n], ['POOL_B', 100_0000000n]]), pools, ORACLE_DECIMALS)

    expect(result.priced).toBe(false)
    expect(!result.priced && result.unpricedPools).toEqual(['POOL_B'])
  })

  it('withholds the total when a debt leg has no pool at all', () => {
    const result = calcUserTotalBorrowedInUsd(obligation([], [['POOL_A', 100_0000000n], ['GONE', 100_0000000n]]), [poolData('POOL_A', ONE_USD)], ORACLE_DECIMALS)

    expect(result.priced).toBe(false)
    expect(!result.priced && result.unpricedPools).toEqual(['GONE'])
  })

  it('reports an empty obligation as a priced zero', () => {
    const result = calcUserTotalBorrowedInUsd(obligation([], []), [], ORACLE_DECIMALS)

    expect(result.priced).toBe(true)
    expect(result.priced && result.usd).toBe(0)
  })
})

describe('calcUserTotalStakeInUsd', () => {
  it('totals collateral when every leg is priced', () => {
    const pools = [poolData('POOL_A', ONE_USD), poolData('POOL_B', ONE_USD)]
    const result = calcUserTotalStakeInUsd(obligation([['POOL_A', 50_0000000n], ['POOL_B', 50_0000000n]], []), pools, ORACLE_DECIMALS)

    expect(result.priced).toBe(true)
    expect(result.priced && result.usd).toBe(100)
  })

  it('applies the close-LTV weighting to a priced total', () => {
    const result = calcUserTotalStakeInUsd(obligation([['POOL_A', 100_0000000n]], []), [poolData('POOL_A', ONE_USD)], ORACLE_DECIMALS, 'close')

    expect(result.priced && result.usd).toBe(90)
  })

  it('withholds the total when a collateral leg has no oracle price', () => {
    const pools = [poolData('POOL_A', ONE_USD), poolData('POOL_B', 0n)]
    const result = calcUserTotalStakeInUsd(obligation([['POOL_A', 50_0000000n], ['POOL_B', 50_0000000n]], []), pools, ORACLE_DECIMALS)

    expect(result.priced).toBe(false)
    expect(!result.priced && result.unpricedPools).toEqual(['POOL_B'])
  })

  it('flags an unpriced borrow leg so a health factor cannot mix a full numerator with a short denominator', () => {
    const pools = [poolData('POOL_A', ONE_USD), poolData('POOL_B', 0n)]
    const obl = obligation([['POOL_A', 50_0000000n]], [['POOL_B', 100_0000000n]])

    expect(calcUserTotalStakeInUsd(obl, pools, ORACLE_DECIMALS, 'close').priced).toBe(true)
    expect(calcUserTotalBorrowedInUsd(obl, pools, ORACLE_DECIMALS).priced).toBe(false)
  })
})
