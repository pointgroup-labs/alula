import type { Pool } from 'sdk'

export type TableAsset = {
  asset: {
    name: string
    symbol: string
    icon: string
  }
}

export type MarketTableItem = {
  raw: Pool
  total_supply: number
  total_borrowed: number
  deposit_apy: string
  borrow_apy: string
  utilization_rate: string
  max_ltv: string
  action: string
  price: number
  available: number
  supply_limit: number
} & TableAsset

export type BorrowTableItem = {
  available: string
  price: number
  borrow_apy: string
  utilization_rate: string
  position: string
  action: string
}

export type BorrowCardTableItem = {
  debt: string | number
  borrow_apy: string | number
  action: string | number
} & TableAsset

export type SuppliedCardTableItem = {
  balance: string | number
  supply_apy: string | number
  action: string
} & TableAsset
