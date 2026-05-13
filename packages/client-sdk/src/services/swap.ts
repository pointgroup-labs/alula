import type { ObligationKey, Request } from '@alula/market-sdk'
import type { DecimalsConfig } from '../config/decimals'
import type { RPCcluster } from '../types'
import { Client as MarketClient } from '@alula/market-sdk'
import { Client as AquaSwapProviderClient } from 'aqua_swap_provider'
import Decimal from 'decimal.js'
import { Client as SoroswapSwapProviderClient } from 'soroswap_swap_provider'
import { AQUA_PROVIDER_ADDRESS, SOROSWAP_PROVIDER_ADDRESS } from '../constants'
import { BaseClient } from '../core/base-client'
import { TransactionHelper } from '../core/transaction-builder'

/**
 * Minimal interface that `executeSwap` needs from the wallet kit. We don't
 * pull in `@creit.tech/stellar-wallets-kit` here to keep the SDK's dependency
 * surface narrow — `TransactionHelper.signAndSend` is the only consumer and
 * it just needs a `signTransaction` callable. Any kit conforming to this
 * shape works (real kit, mocks, e2e doubles).
 */
export interface SigningKit {
  signTransaction: (
    xdr: string,
    opts?: Record<string, unknown>,
  ) => Promise<{ signedTxXdr: string }>
}

export interface SwapServiceConfig {
  rpc: RPCcluster
  publicKey?: string
  contractId?: string
  horizonRpcUrl?: string
  sorobanRpcUrl?: string
  decimals: DecimalsConfig
}

export interface SwapPreviewParams {
  fromTokenAddress: string
  toTokenAddress: string
  fromTokenDecimals: number
  toTokenDecimals: number
  amountIn: string | number
  slippagePercent?: number
  swapProviderAddress: string
  /** Optional explicit path; defaults to direct [from, to]. */
  path?: string[]
}

export interface BuildSwapTxParams extends SwapPreviewParams {
  user: ObligationKey
}

export interface SwapPreview {
  fromTokenAddress: string
  toTokenAddress: string
  swapPath: string[]
  swapProviderAddress: string
  amountIn: bigint
  expectedAmountOut: bigint
  minAmountOut: bigint
  /** expectedOut / amountIn, scaled to human units using token decimals. */
  spotPrice: number
  slippagePercent: number
}

/**
 * One candidate route returned by `getSwapRoutes`. The webapp ranks these and
 * lets the user pin one; `previewFromRoute` then turns the chosen route into a
 * full `SwapPreview` (adds slippage / `minAmountOut`) without re-querying the
 * provider.
 */
export interface SwapRoute {
  /** Stable, UI-friendly key — `"<providerName>:<path joined by '>'>"`. */
  key: string
  /** Display label for the route picker. */
  providerName: string
  providerAddress: string
  path: string[]
  /** Hop count: 1 = direct, 2 = single intermediate, etc. */
  hops: number
  amountIn: bigint
  expectedAmountOut: bigint
  spotPrice: number
  fromTokenDecimals: number
  toTokenDecimals: number
}

export interface GetSwapRoutesParams {
  fromTokenAddress: string
  toTokenAddress: string
  fromTokenDecimals: number
  toTokenDecimals: number
  amountIn: string | number
  /**
   * Optional override of providers to query. Defaults to all known providers
   * (`Aquarius`, `Soroswap`).
   */
  providers?: SwapProviderDescriptor[]
}

export interface SwapProviderDescriptor {
  name: string
  address: string
}

const DEFAULT_SLIPPAGE_PERCENT = 0.5
const MAX_SLIPPAGE_PERCENT = 50

/**
 * Standalone swap executed through the Market contract's `submit_requests_batch`
 * with a single `SwapExactTokens` request. The Market contract's `process_swap_exact`
 * does not touch obligations/collateral when invoked outside a multiply batch —
 * it just forwards to the configured provider. See
 * `contracts/market/src/processors.rs::process_swap_exact`.
 */
export class SwapService extends BaseClient {
  private marketClient: MarketClient
  private txHelper: TransactionHelper

  constructor(config: SwapServiceConfig) {
    super(config)

    this.marketClient = new MarketClient({
      publicKey: config.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: config.contractId || '',
      networkPassphrase: this.networkPassphrase,
    })

    this.txHelper = new TransactionHelper(config.rpc, this.sorobanServer)
  }

  async getSwapPreview(params: SwapPreviewParams): Promise<SwapPreview> {
    const slippagePercent = this.normalizeSlippage(params.slippagePercent)
    const amountIn = this.amountToBigInt(params.amountIn, params.fromTokenDecimals)

    if (amountIn <= 0n) {
      throw new Error('Swap amount must be greater than 0')
    }

    const swapPath = params.path?.length
      ? params.path
      : [params.fromTokenAddress, params.toTokenAddress]

    if (swapPath.length < 2 || swapPath[0] === swapPath[swapPath.length - 1]) {
      throw new Error('Invalid swap path')
    }

    const expectedAmountsOut = await this.getExpectedAmountsOut(
      params.swapProviderAddress,
      amountIn,
      swapPath,
    )
    const expectedAmountOut = expectedAmountsOut[expectedAmountsOut.length - 1]
    if (expectedAmountOut == null || expectedAmountOut <= 0n) {
      throw new Error('Router did not return an output quote for this swap path')
    }

    const minAmountOut = this.applySlippageDown(expectedAmountOut, slippagePercent)
    const spotPrice = this.calculateSpotPrice(
      amountIn,
      expectedAmountOut,
      params.fromTokenDecimals,
      params.toTokenDecimals,
    )

    return {
      fromTokenAddress: params.fromTokenAddress,
      toTokenAddress: params.toTokenAddress,
      swapPath,
      swapProviderAddress: params.swapProviderAddress,
      amountIn,
      expectedAmountOut,
      minAmountOut,
      spotPrice,
      slippagePercent,
    }
  }

  /**
   * Quote the same direct path against every known provider in parallel and
   * return a ranked list (best `expectedAmountOut` first). Failed candidates
   * are silently dropped — the UI can show "no routes found" if the result is
   * empty.
   *
   * Multi-hop discovery is intentionally out of scope for now; if the protocol
   * later wants `[from, USDC, to]` style bridging, extend `candidatePaths`.
   */
  async getSwapRoutes(params: GetSwapRoutesParams): Promise<SwapRoute[]> {
    const amountIn = this.amountToBigInt(params.amountIn, params.fromTokenDecimals)
    if (amountIn <= 0n) {
      throw new Error('Swap amount must be greater than 0')
    }
    if (params.fromTokenAddress === params.toTokenAddress) {
      return []
    }

    const providers = params.providers ?? [
      { name: 'Aquarius', address: AQUA_PROVIDER_ADDRESS[this.rpc] },
      { name: 'Soroswap', address: SOROSWAP_PROVIDER_ADDRESS[this.rpc] },
    ].filter(p => p.address)
    // TODO(multi-hop): when `getExpectedAmountsOut` learns to quote a path of
    // length > 2 (e.g. via per-hop provider calls or a router-side multi-hop
    // RPC), extend `candidatePaths` with `[from, USDC, to]`-style bridges.
    // The `Promise.allSettled` shape below already supports a wider fan-out.
    const candidatePaths: string[][] = [
      [params.fromTokenAddress, params.toTokenAddress],
    ]

    const settled = await Promise.allSettled(
      providers.flatMap(provider =>
        candidatePaths.map(async (path) => {
          const expectedAmountsOut = await this.getExpectedAmountsOut(
            provider.address,
            amountIn,
            path,
          )
          const expectedAmountOut = expectedAmountsOut[expectedAmountsOut.length - 1]
          if (expectedAmountOut == null || expectedAmountOut <= 0n) {
            throw new Error(`No quote from ${provider.name}`)
          }
          const route: SwapRoute = {
            key: `${provider.name}:${path.join('>')}`,
            providerName: provider.name,
            providerAddress: provider.address,
            path,
            hops: path.length - 1,
            amountIn,
            expectedAmountOut,
            spotPrice: this.calculateSpotPrice(
              amountIn,
              expectedAmountOut,
              params.fromTokenDecimals,
              params.toTokenDecimals,
            ),
            fromTokenDecimals: params.fromTokenDecimals,
            toTokenDecimals: params.toTokenDecimals,
          }
          return route
        }),
      ),
    )

    return settled
      .filter((r): r is PromiseFulfilledResult<SwapRoute> => r.status === 'fulfilled')
      .map(r => r.value)
      .sort((a, b) => (a.expectedAmountOut > b.expectedAmountOut ? -1 : a.expectedAmountOut < b.expectedAmountOut ? 1 : 0))
  }

  /**
   * Pure helper: turn a previously-quoted `SwapRoute` into a full `SwapPreview`
   * by applying slippage. No RPC — used after the user picks a route from the
   * ranked list, so we don't re-quote the provider just to render a min-amount.
   */
  previewFromRoute(route: SwapRoute, slippagePercent?: number): SwapPreview {
    const slippage = this.normalizeSlippage(slippagePercent)
    const minAmountOut = this.applySlippageDown(route.expectedAmountOut, slippage)
    return {
      fromTokenAddress: route.path[0]!,
      toTokenAddress: route.path[route.path.length - 1]!,
      swapPath: route.path,
      swapProviderAddress: route.providerAddress,
      amountIn: route.amountIn,
      expectedAmountOut: route.expectedAmountOut,
      minAmountOut,
      spotPrice: route.spotPrice,
      slippagePercent: slippage,
    }
  }

  async buildSwapTx(params: BuildSwapTxParams) {
    const preview = await this.getSwapPreview(params)

    const requests: Request[] = [
      {
        tag: 'SwapExactTokens',
        values: [{
          amount_in: preview.amountIn,
          min_amount_out: preview.minAmountOut,
          path: preview.swapPath,
          swap_provider: params.swapProviderAddress,
        }],
      },
    ]

    const tx = await this.marketClient.submit_requests_batch({
      user: params.user,
      requests,
      referrer: undefined,
    })

    return { preview, tx }
  }

  async executeSwap(params: BuildSwapTxParams, kit: SigningKit, options = { debug: false }) {
    const { tx } = await this.buildSwapTx(params)

    if (options?.debug) {
      console.log('%c[Swap Tx]', 'color: #00ff00', tx)
    }

    return await this.txHelper.signAndSend(tx, params.user, kit, options)
  }

  private async getExpectedAmountsOut(
    swapProviderAddress: string,
    amountIn: bigint,
    path: string[],
  ): Promise<bigint[]> {
    // The provider's `get_amount_out` returns a single scalar for the whole
    // path, not a per-hop array. We therefore only support direct paths today —
    // attempting to fan out across an intermediate asset would require either a
    // provider call per hop or a different provider RPC, neither of which is
    // wired up. Fail loudly rather than silently mis-quote.
    if (path.length !== 2) {
      throw new Error(
        `Multi-hop swap paths are not yet supported (got ${path.length} tokens)`,
      )
    }
    const providerClient = this.getSwapProviderClient(swapProviderAddress)
    const response = await providerClient.get_amount_out({
      amount_in: amountIn,
      path,
    })
    return [amountIn, response.result]
  }

  private getSwapProviderClient(swapProviderAddress: string) {
    const options = {
      publicKey: this.publicKey,
      rpcUrl: this.getSorobanRpcUrl(),
      contractId: swapProviderAddress,
      networkPassphrase: this.networkPassphrase,
    }

    if (swapProviderAddress === AQUA_PROVIDER_ADDRESS[this.rpc]) {
      return new AquaSwapProviderClient(options)
    }

    return new SoroswapSwapProviderClient(options)
  }

  private normalizeSlippage(slippagePercent?: number): number {
    if (slippagePercent == null || !Number.isFinite(slippagePercent)) {
      return DEFAULT_SLIPPAGE_PERCENT
    }
    if (slippagePercent < 0) {
      throw new Error(`Slippage must be non-negative, got ${slippagePercent}`)
    }
    if (slippagePercent > MAX_SLIPPAGE_PERCENT) {
      throw new Error(`Slippage must be at most ${MAX_SLIPPAGE_PERCENT}%, got ${slippagePercent}`)
    }
    return slippagePercent
  }

  private amountToBigInt(amount: string | number, decimals: number): bigint {
    return this.decimalToBigInt(
      new Decimal(amount).mul(new Decimal(10).pow(decimals)),
      Decimal.ROUND_DOWN,
    )
  }

  private applySlippageDown(amount: bigint, slippagePercent: number): bigint {
    return this.decimalToBigInt(
      new Decimal(amount.toString()).mul(new Decimal(1).minus(new Decimal(slippagePercent).div(100))),
      Decimal.ROUND_DOWN,
    )
  }

  private calculateSpotPrice(
    amountIn: bigint,
    amountOut: bigint,
    fromDecimals: number,
    toDecimals: number,
  ): number {
    if (amountIn <= 0n) {
      return 0
    }
    const inHuman = new Decimal(amountIn.toString()).div(new Decimal(10).pow(fromDecimals))
    const outHuman = new Decimal(amountOut.toString()).div(new Decimal(10).pow(toDecimals))
    return outHuman.div(inHuman).toNumber()
  }

  private decimalToBigInt(value: Decimal, rounding: Decimal.Rounding): bigint {
    return BigInt(value.toFixed(0, rounding))
  }
}
