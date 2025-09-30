import type { AnnualPercentageYields, MultiplyPair, Obligation, Pool } from '@alula/market-sdk'
// import type { CompoundRates, Obligation, Pool } from '@jlend/sdk'
import type { RPCcluster } from '../types'
import { Client } from '@alula/market-sdk'
import { rpc as SorobanRpc } from '@stellar/stellar-sdk'
import { amountToBigInt, bigintToNumber, getNetworkPassphrase, getRPC, sendSorobanTx } from '../utils'

export class MarketClient extends Client {
  rpc: RPCcluster
  sorobanServer: any
  assetDecimals: number = 7
  oracleDecimals: number = 14

  constructor(rpc: RPCcluster, publicKey?: string, market?: string) {
    super({
      publicKey,
      rpcUrl: getRPC(rpc, 'soroban'),
      contractId: market || '',
      networkPassphrase: getNetworkPassphrase(rpc),
    })

    this.sorobanServer = new SorobanRpc.Server(getRPC(rpc, 'soroban'))
    // this.getDecimals()
    this.rpc = rpc
  }

  /**
   * Get market data
   */
  async getMarketData() {
    return (await this.get_global_state()).result
  }

  /**
   * Get asset decimals
   */
  async getAssetDecimals() {
    this.assetDecimals = (await this.get_asset_decimals()).result
  }

  /**
   * Get oracle decimals
   */
  async getOracleDecimals() {
    this.oracleDecimals = (await this.get_oracle_price_decimals()).result
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
    const poolPrice = await this.get_pool_asset_oracle_price({ pool_address })
    const poolPriceResult: bigint = this.unwrapOk2(poolPrice.result)
    const normalizedPrice = bigintToNumber(poolPriceResult, this.oracleDecimals)
    return normalizedPrice || 0
  }

  /**
   * Get all pools
   */
  async getAllPools() {
    return (await this.get_all_pools()).result
  }

  /**
   * Get all leverage pools
   */
  async getAllLeveragePools() {
    return (await this.get_all_multiply_pairs()).result
  }

  /**
   * Get leverage pool
   */
  async getLeveragePool(deposit_pool_address: string, borrow_pool_address: string): Promise<MultiplyPair> {
    const result = await this.get_multiply_pair({ deposit_pool_address, borrow_pool_address })
    return this.unwrapOk2(result.result)
  }

  /**
   * Get pool info
   */
  async getPoolInfo(pool_address: string): Promise<Pool> {
    const poolResult = await this.get_pool({ pool_address })
    return this.unwrapOk2(poolResult.result)
  }

  /**
   * Get pool APY
   */
  async getPoolApy(pool_address: string): Promise<AnnualPercentageYields> {
    const poolApy = await this.get_apy({ pool_address })
    return this.unwrapOk2(poolApy.result)
  }

  /**
   * Get user obligation
   * @param {string} user
   */
  async getUserObligation(user: string): Promise<Obligation> {
    const obligation = await this.get_user_obligation({ user })
    return this.unwrapOk2(obligation.result)
  }

  async getUserMultiplyObligation(user: string, deposit_pool_address: string, borrow_pool_address: string): Promise<Obligation> {
    const obligation = await this.get_multiply_pair_obligation({ user, deposit_pool_address, borrow_pool_address })
    return this.unwrapOk2(obligation.result)
  }

  /**
   * Deposit Tx
   */
  async depositTx(user: string, pool_address: string, amount: string | number): Promise<any> {
    return await this.deposit({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Borrow Tx
   */
  async borrowTx(user: string, pool_address: string, amount: string | number) {
    return await this.borrow({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Withdraw Tx
   */
  async withdrawTx(user: string, pool_address: string, amount: string | number) {
    return await this.withdraw({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Repay Tx
   */
  async repayTx(user: string, pool_address: string, amount: string | number) {
    return await this.repay({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Collateral Tx
   */
  async collateralTx(user: string, pool_address: string, amount: string | number) {
    return await this.add_collateral({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
  }

  /**
   * Remove collateral Tx
   */
  async removeCollateralTx(user: string, pool_address: string, amount: string | number) {
    return await this.remove_collateral({ user, pool_address, amount: amountToBigInt(String(amount), this.assetDecimals) })
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
    return await this.deposit_with_leverage(
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
    return await this.withdraw_from_leveraged(
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
  async wathdrawDeposit(
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
