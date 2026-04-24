import type { RPCcluster } from './types'
import { ObligationService } from './services'
import { BorrowingService } from './services/borrowing'
import { LendingService } from './services/lending'
import { MarketService } from './services/market'
import { MarketManagerService } from './services/market-manager'
import { MultiplyService } from './services/multiply'
import { SwapService } from './services/swap'
import { WalletService } from './services/wallet'

/**
 * Main client configuration
 */
export interface StellarClientConfig {
  publicKey?: string
  marketContractId?: string
  opts: StellarClientOptions
}

export type StellarClientOptions = {
  rpc: RPCcluster
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
}

/**
 * Main Stellar client that provides access to all services
 *
 * @example
 * ```typescript
 * const client = new StellarClient({
 *   publicKey: 'GABC...',
 *   rpc: 'testnet',
 *   marketContractId: 'CABC...'
 * })
 *
 * // Market operations
 * const marketData = await client.market.getMarketData()
 * const poolData = await client.market.getPoolData(poolAddress)
 *
 * // Lending operations
 * await client.lending.deposit(user, poolAddress, amount, kit)
 *
 * // Borrowing operations
 * await client.borrowing.borrow(user, poolAddress, amount, kit)
 *
 * // Leverage operations
 * await client.leverage.openPosition({ ... }, kit)
 *
 * // Wallet operations
 * const balances = await client.wallet.getBalances()
 * ```
 */
export class StellarClient {
  public readonly rpc: RPCcluster
  public readonly market: MarketService
  public readonly obligation: ObligationService
  public readonly lending: LendingService
  public readonly borrowing: BorrowingService
  public readonly multiply: MultiplyService
  public readonly swap: SwapService
  public readonly wallet: WalletService
  public readonly marketManager: MarketManagerService
  public readonly horizonRpcUrl?: string
  public readonly sorobanRpcUrl?: string

  private constructor(
    config: StellarClientConfig,
    market: MarketService,
  ) {
    this.rpc = config.opts.rpc
    this.market = market
    this.horizonRpcUrl = config.opts.horizonRpcUrl
    this.sorobanRpcUrl = config.opts.sorobanRpcUrl

    const context = {
      rpc: config.opts.rpc,
      publicKey: config.publicKey,
      contractId: config.marketContractId,
      horizonRpcUrl: config.opts.horizonRpcUrl,
      sorobanRpcUrl: config.opts.sorobanRpcUrl,
    }

    const decimals = market.decimals

    this.lending = new LendingService({ ...context, decimals })
    this.borrowing = new BorrowingService({ ...context, decimals })
    this.multiply = new MultiplyService({ ...context, decimals })
    this.swap = new SwapService({ ...context, decimals })
    this.wallet = new WalletService(context)
    this.obligation = new ObligationService(context)
    this.marketManager = new MarketManagerService(context)
  }

  static async create(config: StellarClientConfig): Promise<StellarClient> {
    const market = await MarketService.create({
      rpc: config.opts.rpc,
      publicKey: config.publicKey,
      contractId: config.marketContractId,
      horizonRpcUrl: config.opts.horizonRpcUrl,
      sorobanRpcUrl: config.opts.sorobanRpcUrl,
    })

    return new StellarClient(config, market)
  }

  /**
   * Create client from address (factory method)
   */
  static async fromAddress(
    address: string | undefined,
    marketContractId?: string,
    opts: StellarClientOptions = { rpc: 'testnet' },
  ) {
    return StellarClient.create({
      publicKey: address,
      marketContractId,
      opts,
    })
  }
}
