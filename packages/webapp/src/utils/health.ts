import type { ObligationArray } from '@alula/client-sdk'
import type { PoolData } from '@alula/market-sdk'
import { bpsToNumber } from '@alula/client-sdk'
import { calcUserTotalStakeInUsd } from '@alula/client-sdk/src/utils'

/**
 * Computes Σ(Vb_j × LF_j) — total borrowed weighted by Liability Factor.
 */
export function calcWeightedBorrowedUsd(
  obligation: ObligationArray,
  poolsData: PoolData[],
  assetDecimals: number,
  oraclePriceDecimals: number,
): number {
  let borrowedWithLF = 0
  for (const [poolAddress, data] of obligation.borrows) {
    const pool = poolsData.find(p => p.pool.pool_address === poolAddress)
    if (!pool) { continue }
    const price = pool.oracle_asset_price
      ? bigintToNumber(pool.oracle_asset_price, oraclePriceDecimals)
      : 0
    const borrowBps = bigintToNumber(data.d_tokens * BigInt(pool.d_token_rate_ceil_bps), assetDecimals)
    const lf = bpsToNumber(Number(pool.pool.config.health_config.liability_factor_bps))
    borrowedWithLF += bpsToNumber(Number(borrowBps)) * Number(price) * lf
  }
  return borrowedWithLF
}

/**
 * Computes Health Factor = Σ(Vc × closeLTV) / Σ(Vb × LF), capped at 10.
 *
 * @param obligation
 * @param poolsData
 * @param assetDecimals
 * @param oraclePriceDecimals
 * @param depositAdjustUsd - subtract from weighted deposit (e.g. withdraw: amount × price × closeLTV)
 * @param borrowAdjustUsd  - add to weighted borrow (e.g. borrow: +amount × price × LF; repay: -amount × price × LF)
 */
export function calcHealthFactor(
  obligation: ObligationArray,
  poolsData: PoolData[],
  assetDecimals: number,
  oraclePriceDecimals: number,
  depositAdjustUsd = 0,
  borrowAdjustUsd = 0,
): number {
  if (obligation.borrows.length === 0 && borrowAdjustUsd <= 0) {
    return 10
  }

  const depositWithCloseLTV = calcUserTotalStakeInUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals, 'close')
  const adjustedDeposit = Math.max(depositWithCloseLTV - depositAdjustUsd, 0)

  const borrowedWithLF = calcWeightedBorrowedUsd(obligation, poolsData, assetDecimals, oraclePriceDecimals)
  const adjustedBorrow = Math.max(borrowedWithLF + borrowAdjustUsd, 0)

  const hf = adjustedBorrow > 0 ? adjustedDeposit / adjustedBorrow : 10
  return Math.min(hf, 10)
}
