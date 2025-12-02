import type { AnnualPercentageYields, MultiplyPair, Obligation, Pool } from '@alula/market-sdk'
// import type { CompoundRates, Obligation, Pool } from '@jlend/sdk'
import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { rpc as SorobanRpc } from '@stellar/stellar-sdk'
import { amountToBigInt, bigintToNumber, bindOwnMethods, cacheManager, getNetworkPassphrase, getRPC, hidePrivate, sendSorobanTx } from '../utils'

export class MarketClient {
  rpc: RPCcluster
  sorobanServer: any
  assetDecimals: number = 7
  oracleDecimals: number = 14
  private base: Client
  private market?: string

  constructor(rpc: RPCcluster, publicKey?: string, market?: string) {
    this.base = new Client({
      publicKey,
      rpcUrl: getRPC(rpc, 'soroban'),
      contractId: market || '',
      networkPassphrase: getNetworkPassphrase(rpc),
    })

    this.sorobanServer = new SorobanRpc.Server(getRPC(rpc, 'soroban'))
    this.rpc = rpc
    this.market = market

    if (market) {
      this.getDecimals().catch(() => {})
    }

    hidePrivate(this, 'base')
    bindOwnMethods(this)
  }

  /**
   * Get market data
   */
  async getMarketData() {
    const result = await this.base.get_market_data()
    return this.unwrapOk2(result.result)
  }

  /**
   * Get asset decimals
   */
  async getAssetDecimals() {
    if (!this.market) {
      return
    }
    const key = cacheManager.key(this.rpc, this.market, 'decimals:asset')
    this.assetDecimals = await cacheManager.getOrSet<number>(key, async () => {
      return (await this.base.get_asset_decimals()).result
    })
  }

  /**
   * Get oracle decimals
   */
  async getOracleDecimals() {
    if (!this.market) {
      return
    }
    const key = cacheManager.key(this.rpc, this.market, 'decimals:oracle')
    this.oracleDecimals = await cacheManager.getOrSet<number>(key, async () => {
      return (await this.base.get_oracle_price_decimals()).result
    })
  }

  /**
   * Get decimals
   */
  async getDecimals() {
    await Promise.all([
      this.getAssetDecimals(),
      this.getOracleDecimals(),
    ])
  }

  /**
   * Get pool asset oracle price
   */
  async getPoolAssetOraclePrice(pool_address: string) {
    const poolPrice = await this.base.get_pool_asset_oracle_price({ pool_address })
    const poolPriceResult: bigint = this.unwrapOk2(poolPrice.result)
    const normalizedPrice = bigintToNumber(poolPriceResult, this.oracleDecimals)
    return normalizedPrice || 0
  }

  /**
   * Get all pools -- DELETE
   */
  async getAllPools() {
    return (await this.base.get_all_pools()).result
  }

  /**
   * Get pool
   */
  async getPoolData(pool_address: string) {
    const poolResult = await this.base.get_pool_data({ pool_address })
    return this.unwrapOk2(poolResult.result)
  }

  /**
   * Get all leverage pools -- DELETE
   */
  async getAllLeveragePools() {
    return (await this.base.get_all_multiply_pairs()).result
  }

  /**
   * Get leverage pool -- DELETE
   */
  async getLeveragePool(deposit_pool_address: string, borrow_pool_address: string): Promise<MultiplyPair> {
    const result = await this.base.get_multiply_pair({ deposit_pool_address, borrow_pool_address })
    return this.unwrapOk2(result.result)
  }

  /**
   * Get pool info -- DELETE
   */
  async getPoolInfo(pool_address: string): Promise<Pool> {
    const poolResult = await this.base.get_pool({ pool_address })
    return this.unwrapOk2(poolResult.result)
  }

  /**
   * Get user obligation
   * @param {string} user
   */
  async getUserObligation(user: string): Promise<Obligation> {
    const obligation = await this.base.get_user_obligation({ user })
    return this.unwrapOk2(obligation.result)
  }

  async getUserMultiplyObligation(user: string, deposit_pool_address: string, borrow_pool_address: string): Promise<Obligation> {
    const obligation = await this.base.get_multiply_pair_obligation({ user, deposit_pool_address, borrow_pool_address })
    return this.unwrapOk2(obligation.result)
  }

  /**
   * Deposit Tx
   */
  async depositTx(user: string, pool_address: string, amount: string | number): Promise<any> {
    return await this.base.deposit({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Borrow Tx
   */
  async borrowTx(user: string, pool_address: string, amount: string | number) {
    return await this.base.borrow({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Withdraw Tx
   */
  async withdrawTx(user: string, pool_address: string, amount: string | number) {
    return await this.base.withdraw({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Repay Tx
   */
  async repayTx(user: string, pool_address: string, amount: string | number) {
    return await this.base.repay({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Collateral Tx
   */
  async collateralTx(user: string, pool_address: string, amount: string | number) {
    return await this.base.add_collateral({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Remove collateral Tx
   */
  async removeCollateralTx(user: string, pool_address: string, amount: string | number) {
    return await this.base.remove_collateral({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Leverage Tx
   */
  async leverageTx(
    user: string,
    deposit_pool_address: string,
    borrow_pool_address: string,
    deposit_as_margin: boolean,
    amount: string | number,
    leverage_multiplier: number) {
    const multiplier = Number(leverage_multiplier * 100).toFixed(0)
    const amountInBigInt = amountToBigInt(String(amount), this.assetDecimals)
    return await this.base.deposit_with_leverage(
      {
        user,
        deposit_pool_address,
        borrow_pool_address,
        deposit_as_margin,
        amount: amountInBigInt,
        leverage_multiplier: Number(multiplier),
      })
  }

  /**
   * Withdraw Leverage Tx
   */

  async withdrawLeverageTx(user: string, deposit_pool_address: string, borrow_pool_address: string, amount: string | number) {
    return await this.base.withdraw_from_leveraged(
      {
        user,
        deposit_pool_address,
        borrow_pool_address,
        amount: amountToBigInt(String(amount), this.assetDecimals),
      })
  }

  /**
   * Deposit
   */
  async depositToLending(
    user: string,
    pool_address: string,
    amount: number,
    kit: any) {
    const tx = await this.depositTx(user, pool_address, amount)

    console.log('%c[Deposit tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Borrow
   */
  async borrowLendingAsset(
    user: string,
    pool_address: string,
    amount: number,
    kit: any) {
    const tx = await this.borrowTx(user, pool_address, String(amount))

    console.log('%c[Borrow tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Withdraw
   */
  async withdrawDeposit(
    user: string,
    pool_address: string,
    amount: number,
    kit: any) {
    const tx = await this.withdrawTx(user, pool_address, String(amount))

    console.log('%c[Withdraw tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Repay
   */
  async repayBorrow(
    user: string,
    pool_address: string,
    amount: number,
    kit: any) {
    const tx = await this.repayTx(user, pool_address, String(amount))

    console.log('%c[Repay tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Add collateral
   */
  async addCollateral(
    user: string,
    pool_address: string,
    amount: number,
    kit: any) {
    const tx = await this.collateralTx(user, pool_address, amount)

    console.log('%c[Collateral tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Remove collacetal
   */
  async removeCollateral(
    user: string,
    pool_address: string,
    amount: number,
    kit: any) {
    const tx = await this.removeCollateralTx(user, pool_address, amount)

    console.log('%c[Remove Collateral tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Leverage
   */
  async leverage(
    user: string,
    deposit_pool_address: string,
    borrow_pool_address: string,
    deposit_as_margin: boolean,
    amount: number,
    leverage_multiplier: number,
    kit: any,
  ) {
    const tx = await this.leverageTx(user, deposit_pool_address, borrow_pool_address, deposit_as_margin, amount, leverage_multiplier)

    console.log('%c[Leverage tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Withdraw Leverage
   */
  async withdrawLeverage(
    user: string,
    deposit_pool_address: string,
    borrow_pool_address: string,
    amount: number,
    kit: any,
  ) {
    const tx = await this.withdrawLeverageTx(user, deposit_pool_address, borrow_pool_address, amount)

    console.log('%c[Withdraw Leverage tx]', 'color: #00ff00', tx)

    return await sendSorobanTx(tx, user, this.rpc, this.sorobanServer, kit)
  }

  /**
   * Get transaction fee
   */
  getTransactionFee(tx: any) {
    const stroops: bigint = tx.simulation.minResourceFee || 0
    return Number(bigintToNumber(stroops, this.assetDecimals))
  }

  /**
   * Unwrap ok2
   */
  private unwrapOk2<T>(ok2: any): T {
    return ok2.value
  }
}
