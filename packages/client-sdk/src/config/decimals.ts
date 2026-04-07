import { Client } from '@alula/market-sdk'
import { cacheManager } from '../utils'

export const DEFAULT_ORACLE_DECIMALS = 14

export const DEFAULT_DECIMALS = {
  oracleDecimals: DEFAULT_ORACLE_DECIMALS,
}

export type DecimalsConfig = Readonly<{
  oracleDecimals: number
}>

export async function loadMarketDecimals(
  client: Client,
  contractId?: string,
): Promise<DecimalsConfig> {
  if (!contractId) {
    return DEFAULT_DECIMALS
  }

  const oracleDecimals = await fetchOracleDecimals({ fetchFn: async () => (await client.get_oracle_price_decimals()).result, contractId })

  return {
    oracleDecimals,
  }
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
