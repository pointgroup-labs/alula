import type { PoolData } from '@alula/market-sdk'

export type TableAsset = {
  asset: {
    name: string
    symbol: string
    icon: string
  }
}

export type MarketTableItem = {
  raw: PoolData
  total_supply: number
  total_borrowed: number
  deposit_apy: string
  borrow_apy: string
  utilization_rate: string
  utilization_rate_percent: number
  open_ltv: string
  action: string
  price: number
  available: number
  supply_limit: number
  utilization_rate_limit: number
  pool_address: string
  market?: string
  assetDecimals: number
  position: {
    supplied: number | string
    borrowed: number | string
  }
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
  pairKey: string
  depositPoolData: PoolData
  borrowPoolData: PoolData
  borrowAsset: TableAsset['asset']
  liquidity: number
  multiplier: number
  apyAtMaxMultiplier: number
  price: number
  borrowPoolPrice: number
  supplied: number
  pool_address: string
  market?: string
  assetDecimals: number
} & TableAsset

export type MultiplyVaultItem = {
  pairKey: string
  market: string
  marketAddress: string
  depositPoolData: PoolData
  borrowPoolData: PoolData
  asset: TableAsset['asset']
  borrowAsset: TableAsset['asset']
  maxMultiplier: number
  apyAtMaxMultiplier: number
  price: number
  borrowPoolPrice: number
  supplied: number
  liquidity: number
  pool_address: string
  netEquityUsd?: number
}

export type MultiplyPositionItem = {
  market: string
  pairKey: string
  deposited: number
  borrowed: number
  depositedUsd: number
  borrowedUsd: number
  netEquityUsd: number
  positionValueUsd: number
  currentMultiplier: number
  supplyApy: number
  borrowApy: number
  currentApy: number
  healthFactor: number
  currentLtv: number
  openLtv: number
  closeLtv: number
  liabilityFactor: number
  yearlyResultUsd: number
  liquidationBufferUsd: number
  liquidationPrice: number | null
  distanceToLiquidationPercent: number | null
}

export type MultiplyPortfolioTableItem = {
  pairKey: string
  depositPoolData: PoolData
  borrowPoolData: PoolData
  borrowAsset: TableAsset['asset']
  deposited: number
  borrowed: number
  healthFactor: number
  multiplier: number
  maxAPY: number
  price: number
  borrowPoolPrice: number
  pool_address: string
  market?: string
  assetDecimals: number
} & TableAsset

export type BorrowCardTableItem = {
  raw: PoolData
  debt: string | number
  debtUsd: number | string
  borrow_apy: string | number
  action: string | number
  price: string | number
  pool_address: string
  asset_issuer: string
  market?: string
  healthFactor: number
} & TableAsset

export type SuppliedCardTableItem = {
  raw: PoolData
  available: string | number
  assetDecimals: number
  balance: string | number
  balanceUsd: string | number
  supply_apy: string | number
  action: string
  price: number
  pool_address: string
  collateral: string | number
  collateralPercent: string | number
  deposited: string | number
  depositedPercent: string | number
  market?: string
} & TableAsset
