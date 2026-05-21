import { API_URL } from '~/config'
import { createApiClient } from '.'

const API_CLIENT_URL = `${API_URL}/v1`

const apiClient = createApiClient(API_CLIENT_URL)

export async function fetchMarkets(): Promise<ApiMarkets[]> {
  const response = await apiClient.get('/markets')
  return response.data
}

export async function fetchMarketData(address: string): Promise<ApiMarketData> {
  const response = await apiClient.get(`/markets/${address}`)
  return response.data
}

export async function fetchPoolData(marketAddress: string, poolAddress: string): Promise<ApiPoolData> {
  const response = await apiClient.get(`/assets/${marketAddress}/${poolAddress}`)
  return response.data
}

export async function fetchPoolHistory(marketAddress: string, poolAddress: string, bucket: PoolHistoryBucket = '1d'): Promise<ApiHistoryData[]> {
  const startDate = getPoolHistoryStartDate(bucket)
  const response = await apiClient.get(`/assets/${marketAddress}/${poolAddress}/history?start_date=${startDate}${bucket ? `&bucket=${bucket}` : ''}`)
  return response.data.data
}

export function getPoolHistoryStartDate(bucket: PoolHistoryBucket): string {
  const date = new Date()

  switch (bucket) {
    case '5m':
    case '15m':
    case '1h':
      date.setUTCDate(date.getUTCDate() - 1)
      break

    case '6h':
      date.setUTCDate(date.getUTCDate() - 7)
      break

    case '1d':
      date.setUTCDate(date.getUTCDate() - 31)
      break

    case '1w':
      date.setUTCFullYear(date.getUTCFullYear() - 1)
      break
  }

  return `${date.toISOString().split('.')[0]}Z`
}

export type PoolHistoryBucket = '5m' | '15m' | '1h' | '6h' | '1d' | '1w'

export type ApiMarkets = {
  market: string
  name: string
  observed_ledger: number
  pool_count: number
  tvl_usd_cents: string
}

export type ApiMarketData = {
  config_hash: string
  market: string
  observed_ledger: number
  pool_count: number
  pools: ApiPoolData[]
  truncated: true
  tvl_usd_cents: string
}

export type ApiPoolData = {
  borrow_apy_bps: number
  decimals: number
  pool: string
  supply_apy_bps: number
  symbol: string
  token_address: string
  total_borrowed: string
  total_supplied: string
  tvl_usd_cents: string
  utilization_bps: number
}

export type ApiHistoryData = {
  start_time: Date
  end_time: Date
  total_supplied: string
  total_borrowed: string
  supply_apy_bps: number
  borrow_apy_bps: number
  utilization_bps: number
  tvl_usd_cents: string
}
