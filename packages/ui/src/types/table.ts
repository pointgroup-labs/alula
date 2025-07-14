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
  pool_address: string
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
  pool_address: string
  asset_issuer: string
} & TableAsset

export type SuppliedCardTableItem = {
  available: string | number
  balance: string | number
  supply_apy: string | number
  action: string
  pool_address: string
  collateral: string | number
} & TableAsset
