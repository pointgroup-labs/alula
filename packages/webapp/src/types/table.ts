import type { PoolWithPrice } from '~/store/markets'

export type TableAsset = {
  asset: {
    name: string
    symbol: string
    icon: string
  }
}

export type MarketTableItem = {
  raw: PoolWithPrice
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
  market?: string
} & TableAsset

export type BorrowTableItem = {
  available: string
  price: number
  borrow_apy: string
  utilization_rate: string
  position: string
  action: string
}

export type MultiplyTableItem = {
  depositPool: PoolWithPrice
  borrowPool: PoolWithPrice
  borrowAsset: {
    name: string
    symbol: string
    icon: string
  }
  liquidity: number
  multiplier: number
  maxAPY: number
  price: number
  borrowPoolPrice: number
  supplied: number
  pool_address: string
  market?: string
} & TableAsset

export type BorrowCardTableItem = {
  raw: PoolWithPrice
  debt: string | number
  debtUsd: number | string
  borrow_apy: string | number
  action: string | number
  pool_address: string
  asset_issuer: string
  market?: string
} & TableAsset

export type SuppliedCardTableItem = {
  raw: PoolWithPrice
  available: string | number
  balance: string | number
  balanceUsd: string | number
  supply_apy: string | number
  action: string
  price: number
  pool_address: string
  collateral: string | number
  market?: string
} & TableAsset
