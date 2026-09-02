/* eslint-disable ts/prefer-literal-enum-member */
/** Mirrors `POOL_STATUS_*` in `contracts/market/src/constants.rs`. */
export enum PoolStatusFlag {
  DEPOSIT_ENABLED = Math.trunc(1),
  BORROW_ENABLED = 1 << 1,
  ADD_COLLATERAL_ENABLED = 1 << 2,
  FLASH_LOAN_ENABLED = 1 << 3,
}

export const POOL_STATUS_RESTRICTED_MESSAGES: Record<keyof PoolStatus, string> = {
  canDeposit: 'Supplies',
  canBorrow: 'Borrows',
  canAddCollateral: 'Collateral deposits',
  canFlashLoan: 'Flash loans',
}

export function hasPoolFlag(flags: number, flag: PoolStatusFlag): boolean {
  return ((flags >>> 0) & flag) !== 0
}

export function decodePoolStatus(flags: number): PoolStatus {
  const f = flags >>> 0
  return {
    canDeposit: hasPoolFlag(f, PoolStatusFlag.DEPOSIT_ENABLED),
    canBorrow: hasPoolFlag(f, PoolStatusFlag.BORROW_ENABLED),
    canAddCollateral: hasPoolFlag(f, PoolStatusFlag.ADD_COLLATERAL_ENABLED),
    canFlashLoan: hasPoolFlag(f, PoolStatusFlag.FLASH_LOAN_ENABLED),
  }
}

export type PoolStatus = {
  canDeposit: boolean
  canBorrow: boolean
  canAddCollateral: boolean
  canFlashLoan: boolean
}
