import type { RPCcluster } from '../types'
import { cacheManager } from '../utils'

/**
 * Decimals configuration manager
 */
export class DecimalsConfig {
  private assetDecimals: number = 7
  private oracleDecimals: number = 14
  private rpc: RPCcluster
  private contractId?: string

  constructor(rpc: RPCcluster, contractId?: string) {
    this.rpc = rpc
    this.contractId = contractId
  }

  /**
   * Get asset decimals
   */
  getAssetDecimals(): number {
    return this.assetDecimals
  }

  /**
   * Get oracle decimals
   */
  getOracleDecimals(): number {
    return this.oracleDecimals
  }

  /**
   * Set asset decimals
   */
  setAssetDecimals(decimals: number): void {
    this.assetDecimals = decimals
  }

  /**
   * Set oracle decimals
   */
  setOracleDecimals(decimals: number): void {
    this.oracleDecimals = decimals
  }

  /**
   * Fetch and cache asset decimals from contract
   */
  async fetchAssetDecimals(fetchFn: () => Promise<number>): Promise<void> {
    if (!this.contractId) {
      return
    }
    const key = cacheManager.key(this.rpc, this.contractId, 'decimals:asset')
    this.assetDecimals = await cacheManager.getOrSet<number>(key, fetchFn)
  }

  /**
   * Fetch and cache oracle decimals from contract
   */
  async fetchOracleDecimals(fetchFn: () => Promise<number>): Promise<void> {
    if (!this.contractId) {
      return
    }
    const key = cacheManager.key(this.rpc, this.contractId, 'decimals:oracle')
    this.oracleDecimals = await cacheManager.getOrSet<number>(key, fetchFn)
  }

  /**
   * Fetch both decimals in parallel
   */
  async fetchAll(
    assetFetchFn: () => Promise<number>,
    oracleFetchFn: () => Promise<number>,
  ): Promise<void> {
    await Promise.all([
      this.fetchAssetDecimals(assetFetchFn),
      this.fetchOracleDecimals(oracleFetchFn),
    ])
  }
}
