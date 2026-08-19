import { describe, expect, it } from 'vitest'
import { decodePoolStatus, PoolStatusFlag } from './pool-status'

describe('decodePoolStatus', () => {
  it('maps the four contract flags to their bit positions', () => {
    expect(PoolStatusFlag.DEPOSIT_ENABLED).toBe(1)
    expect(PoolStatusFlag.BORROW_ENABLED).toBe(2)
    expect(PoolStatusFlag.ADD_COLLATERAL_ENABLED).toBe(4)
    expect(PoolStatusFlag.FLASH_LOAN_ENABLED).toBe(8)
  })

  it('decodes each flag in isolation', () => {
    expect(decodePoolStatus(1)).toEqual({ canDeposit: true, canBorrow: false, canAddCollateral: false, canFlashLoan: false })
    expect(decodePoolStatus(2)).toEqual({ canDeposit: false, canBorrow: true, canAddCollateral: false, canFlashLoan: false })
    expect(decodePoolStatus(4)).toEqual({ canDeposit: false, canBorrow: false, canAddCollateral: true, canFlashLoan: false })
    expect(decodePoolStatus(8)).toEqual({ canDeposit: false, canBorrow: false, canAddCollateral: false, canFlashLoan: true })
  })

  it('decodes a mixed mask', () => {
    expect(decodePoolStatus(11)).toEqual({ canDeposit: true, canBorrow: true, canAddCollateral: false, canFlashLoan: true })
  })

  it('treats POOL_STATUS_ALL_ENABLED (u32::MAX) as everything enabled', () => {
    expect(decodePoolStatus(4294967295)).toEqual({ canDeposit: true, canBorrow: true, canAddCollateral: true, canFlashLoan: true })
  })

  it('treats 0 as everything disabled', () => {
    expect(decodePoolStatus(0)).toEqual({ canDeposit: false, canBorrow: false, canAddCollateral: false, canFlashLoan: false })
  })
})
