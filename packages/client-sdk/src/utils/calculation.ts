import type { Obligation } from '@alula/market-sdk'
import Decimal from 'decimal.js'
import { bigintToNumber } from './format'

export function calcUserTotalStakeInUsd(obligation: Obligation, pools: any[], assetDecimals: number) {
  const deposits = obligation?.deposits
  if (!deposits) {
    return 0
  }

  let userDepositsInUsd = 0

  for (const deposit of deposits) {
    const [depositedPoolAddress, data] = deposit
    const depositedPool = pools?.find(p => p.pool_address === depositedPoolAddress)

    const collateral = data?.collateral || 0
    userDepositsInUsd += Number(bigintToNumber(BigInt(collateral), assetDecimals)) * Number(depositedPool?.pool_price)
    const j_tokens = data?.j_tokens
    if (!depositedPool || !j_tokens) {
      userDepositsInUsd += 0
      continue
    }

    const userAvailable = calculateTotalStake(
      j_tokens,
      {
        total_j_tokens: depositedPool.total_j_tokens,
        total_borrowed: depositedPool.total_borrowed,
        total_available: depositedPool.total_available,
      },
      depositedPool.asset_decimals,
    )
    const availableInUsd = Number(userAvailable) * Number(depositedPool.pool_price)
    userDepositsInUsd += availableInUsd || 0
  }
  return userDepositsInUsd
}

export function calcUserTotalBorrowedInUsd(obligation: Obligation, pools: any[], assetDecimals: number) {
  const borrows = obligation?.borrows
  if (!borrows) {
    return 0
  }

  let userBorrowedInUsd = 0

  for (const borrow of borrows) {
    const [borrowedPoolAddress, data] = borrow
    const borrowedPool = pools?.find(p => p.pool_address === borrowedPoolAddress)

    const userBorrow = bigintToNumber(data?.borrowed, assetDecimals)
    if (!borrowedPool || !userBorrow) {
      userBorrowedInUsd += 0
      continue
    }
    const borrowedInUsd = Number(userBorrow) * Number(borrowedPool.pool_price)
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
