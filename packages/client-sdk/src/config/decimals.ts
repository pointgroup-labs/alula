import { Client } from '@alula/market-sdk'
import { cacheManager } from '../utils'

export const DEFAULT_ASSET_DECIMALS = 7
export const DEFAULT_ORACLE_DECIMALS = 14

export const DEFAULT_DECIMALS = {
  assetDecimals: DEFAULT_ASSET_DECIMALS,
  oracleDecimals: DEFAULT_ORACLE_DECIMALS,
}

export type DecimalsConfig = {
  assetDecimals: number
  oracleDecimals: number
}

export async function loadMarketDecimals(
  client: Client,
  contractId?: string,
): Promise<DecimalsConfig> {
  if (!contractId) {
    return DEFAULT_DECIMALS
  }

  const [assetDecimals, oracleDecimals] = await Promise.all([
    fetchAssetDecimals({ fetchFn: async () => (await client.get_asset_decimals()).result, contractId }),
    fetchOracleDecimals({ fetchFn: async () => (await client.get_oracle_price_decimals()).result, contractId }),
  ])

  return {
    assetDecimals,
    oracleDecimals,
  }
}

export async function fetchAssetDecimals(props: {
  fetchFn: () => Promise<number>
  contractId?: string
}): Promise<number> {
  if (!props.contractId) {
    return DEFAULT_ASSET_DECIMALS
  }
  const { fetchFn } = props
  const key = cacheManager.key(props.contractId, 'decimals:asset')
  return await cacheManager.getOrSet<number>(key, fetchFn)
}

/**
 * Fetch and cache oracle decimals from contract
 */
export async function fetchOracleDecimals(props: {
  fetchFn: () => Promise<number>
  contractId?: string
}): Promise<number> {
  if (!props.contractId) {
    return DEFAULT_ORACLE_DECIMALS
  }
  const { fetchFn } = props
  const key = cacheManager.key(props.contractId, 'decimals:oracle')
  return await cacheManager.getOrSet<number>(key, fetchFn)
}
