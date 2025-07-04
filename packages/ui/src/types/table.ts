export type TableAsset = {
  asset: {
    name: string
    symbol: string
    icon: string
  }
}

export type SupplyTableItem = {
  pool_size: string
  price: string
  deposit_apy: string
  trust_ratio: string
  risk_floor: string
  position: string
  action: string
} & TableAsset

export type BorrowTableItem = {
  available: string
  price: string
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
