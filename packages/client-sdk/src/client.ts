import type { RPCcluster } from './types'
import { ObligationService } from './services'
import { BorrowingService } from './services/borrowing-service'
import { LendingService } from './services/lending-service'
import { LeverageService } from './services/leverage-service'
import { MarketManagerService } from './services/market-manager-service'
import { MarketService } from './services/market-service'
import { WalletService } from './services/wallet-service'

/**
 * Main client configuration
 */
export interface StellarClientConfig {
  publicKey: string
  rpc: RPCcluster
  marketContractId?: string
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
  public readonly publicKey: string
  public readonly market: MarketService
  public readonly obligation: ObligationService
  public readonly lending: LendingService
  public readonly borrowing: BorrowingService
  public readonly leverage: LeverageService
  public readonly wallet: WalletService
  public readonly marketManager: MarketManagerService

  private constructor(
    config: StellarClientConfig,
    market: MarketService,
  ) {
    this.rpc = config.rpc
    this.publicKey = config.publicKey
    this.market = market

    const context = {
      rpc: config.rpc,
      publicKey: config.publicKey,
      contractId: config.marketContractId,
    }

    const decimals = market.getDecimalsConfig()

    this.lending = new LendingService({ ...context, decimals })
    this.borrowing = new BorrowingService({ ...context, decimals })
    this.leverage = new LeverageService({ ...context, decimals })
    this.wallet = new WalletService(context)
    this.obligation = new ObligationService(context)
    this.marketManager = new MarketManagerService(context)
  }

  static async create(config: StellarClientConfig): Promise<StellarClient> {
    const market = await MarketService.create({
      rpc: config.rpc,
      publicKey: config.publicKey,
      contractId: config.marketContractId,
    })

    return new StellarClient(config, market)
  }

  /**
   * Create client from address (factory method)
   */
  static async fromAddress(
    address: string,
    rpc: RPCcluster,
    marketContractId?: string,
  ) {
    return StellarClient.create({
      publicKey: address,
      rpc,
      marketContractId,
    })
  }
}
