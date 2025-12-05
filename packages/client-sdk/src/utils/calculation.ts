import type { Obligation, PoolData } from '@alula/market-sdk'
import Decimal from 'decimal.js'
import { bigintToNumber, bpsToNumber } from './format'

export function calcUserTotalStakeInUsd(obligation: ObligationArray, poolsData: PoolData[], assetDecimals: number, oraclePriceDecimals: number) {
  const deposits = [...obligation?.deposits]
  if (!deposits || deposits.length === 0) {
    return 0
  }

  let userDepositsInUsd = 0

  for (const deposit of deposits) {
    const [depositedPoolAddress, data] = deposit
    const depositedPool = poolsData?.find(data => data.pool.pool_address === depositedPoolAddress)

    if (!depositedPool) {
      continue
    }

    const price = depositedPool?.oracle_asset_price ? bigintToNumber(depositedPool?.oracle_asset_price, oraclePriceDecimals) : 0

    const collateral = data?.collateral || 0
    userDepositsInUsd += Number(bigintToNumber(BigInt(collateral), assetDecimals)) * Number(price ?? 0)
    const j_tokens = data?.j_tokens
    if (!depositedPool || !j_tokens) {
      userDepositsInUsd += 0
      continue
    }

    const userAvailable = calculateTotalStake(
      j_tokens,
      {
        total_j_tokens: depositedPool.pool.total_j_tokens,
        total_borrowed: depositedPool.pool.total_borrowed,
        total_available: depositedPool.pool.total_available,
      },
      assetDecimals,
    )
    const availableInUsd = Number(userAvailable) * Number(price)
    userDepositsInUsd += availableInUsd || 0
  }
  return userDepositsInUsd
}

export function calcUserTotalBorrowedInUsd(obligation: ObligationArray, poolsData: PoolData[], assetDecimals: number, oraclePriceDecimals: number) {
  const borrows = [...obligation?.borrows]
  if (!borrows || borrows.length === 0) {
    return 0
  }

  let userBorrowedInUsd = 0

  for (const borrow of borrows) {
    const [borrowedPoolAddress, data] = borrow
    const borrowedPool = poolsData?.find(data => data.pool.pool_address === borrowedPoolAddress)

    if (!borrowedPool) {
      continue
    }

    const price = borrowedPool?.oracle_asset_price ? bigintToNumber(borrowedPool?.oracle_asset_price, oraclePriceDecimals) : 0

    const borrowBps = bigintToNumber(data.d_tokens * borrowedPool.d_token_rate_ceil_bps, assetDecimals)
    const userBorrow = bpsToNumber(Number(borrowBps))
    if (!borrowedPool || !userBorrow) {
      userBorrowedInUsd += 0
      continue
    }
    const borrowedInUsd = Number(userBorrow) * Number(price)
    userBorrowedInUsd += borrowedInUsd || 0
  }
  return userBorrowedInUsd
}

export function calculateTotalStake(
  j_tokens: bigint,
  depositedPool: {
    total_j_tokens: bigint
    total_borrowed: bigint
    total_available: bigint
  },
  decimals = 7,
) {
  if (depositedPool.total_j_tokens === 0n) {
    return new Decimal(0)
  }
  const j = new Decimal(j_tokens.toString())
  const totalJ = new Decimal(depositedPool.total_j_tokens.toString())
  const borrowed = new Decimal(depositedPool.total_borrowed.toString())
  const available = new Decimal(depositedPool.total_available.toString())
  const raw = j.div(totalJ).mul(borrowed.add(available))
  return raw.div(new Decimal(10).pow(decimals))
}

export function calculateBorrow(
  d_tokens: bigint,
  pool: { total_d_tokens: bigint, total_borrowed: bigint },
  decimals = 7,
): number {
  const d = new Decimal(d_tokens.toString())
  const totalD = new Decimal(pool.total_d_tokens.toString())
  const borrowed = new Decimal(pool.total_borrowed.toString())

  if (d.isZero() || borrowed.isZero() || totalD.isZero()) {
    return 0
  }

  const SCALE = new Decimal(10).pow(decimals)

  const amount = d.mul(borrowed).div(totalD).div(SCALE)

  return amount.isFinite() ? Number(amount.toFixed(decimals)) : 0
}

export function calcFee(
  amount: number,
  fee: number,
) {
  const feeNum = bpsToNumber(fee)
  return feeNum > 0 ? Number(amount) * feeNum : 0
}
