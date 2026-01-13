/* eslint-disable ts/prefer-literal-enum-member */
export enum PoolStatusFlag {
  DEPOSIT_ENABLED = Math.trunc(1),
  BORROW_ENABLED = 1 << 1,
  WITHDRAW_ENABLED = 1 << 2,
  REPAY_ENABLED = 1 << 3,
  ADD_COLLATERAL_ENABLED = 1 << 4,
  REMOVE_COLLATERAL_ENABLED = 1 << 5,
}

export const POOL_STATUS_RESTRICTED_MESSAGES: Record<keyof PoolStatus, string> = {
  canDeposit: 'Supplies',
  canBorrow: 'Borrows',
  canWithdraw: 'Withdrawals',
  canRepay: 'Repayments',
  canAddCollateral: 'Collateral deposits',
  canRemoveCollateral: 'Collateral withdrawals',
}

export function hasPoolFlag(flags: number, flag: PoolStatusFlag): boolean {
  return ((flags >>> 0) & flag) !== 0
}

export function decodePoolStatus(flags: number): PoolStatus {
  const f = flags >>> 0
  return {
    canDeposit: hasPoolFlag(f, PoolStatusFlag.DEPOSIT_ENABLED),
    canBorrow: hasPoolFlag(f, PoolStatusFlag.BORROW_ENABLED),
    canWithdraw: hasPoolFlag(f, PoolStatusFlag.WITHDRAW_ENABLED),
    canRepay: hasPoolFlag(f, PoolStatusFlag.REPAY_ENABLED),
    canAddCollateral: hasPoolFlag(f, PoolStatusFlag.ADD_COLLATERAL_ENABLED),
    canRemoveCollateral: hasPoolFlag(f, PoolStatusFlag.REMOVE_COLLATERAL_ENABLED),
  }
}

export type PoolStatus = {
  canDeposit: boolean
  canBorrow: boolean
  canWithdraw: boolean
  canRepay: boolean
  canAddCollateral: boolean
  canRemoveCollateral: boolean
}
