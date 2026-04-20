import type { RPCcluster, StellarClient } from '@alula/client-sdk'
import type { ObligationKey } from '@alula/market-sdk'
import type { TableActionType } from '~/store/markets'
import { SOROBAN_RPC_URLS } from '@alula/client-sdk'
import { TRANSACTION_TIMEOUT } from '~/config'
import { destructurePoolAsset } from '~/utils'

function parseAsset(asset_data?: string, asset_code_from_param?: string) {
  let asset_code = asset_code_from_param
  let asset_issuer: string | undefined
  if (asset_data) {
    [asset_code, asset_issuer] = destructurePoolAsset(asset_data)
  }
  const symbol = asset_code === 'native' ? 'XLM' : asset_code
  return { asset_code, asset_issuer, symbol }
}

export function useMarketActions() {
  const userStore = useUserStore()
  const marketsStore = useMarketsStore()
  const connectionStore = useConnectionStore()
  const recentStore = useRecentActivityStore()
  const rpcStore = useRpcStore()
  const marketclient = computed(() => marketsStore.marketClient)

  const { generateExplorerLink } = useExplorerLink()

  const agreeToBorrow = useLocalStorage('agreeToBorrow', false)

  const collateralOnly = ref(false)

  const depositAmount = ref()
  const borrowAmount = ref()
  const withdrawAmount = ref()
  const repayAmount = ref()

  const toast = useToast()

  const {
    publicKey,
    nativeBalance,
    getAssetBalance,
    loadBalances,
  } = useWalletComposable()

  const assetDecimals = computed(() => marketsStore.assetDecimals)

  const kit = computed(() => connectionStore.kit)

  async function addTrustLine(asset: string, issuer: string) {
    try {
      if (!publicKey.value) {
        return
      }
      if (nativeBalance.value <= 0.5) {
        throw new Error('Insufficient balance')
      }
      const res = await marketclient.value!.wallet.addTrustline(publicKey.value, asset, issuer, connectionStore.kit)
      await loadBalances()
      return res
    } catch (error) {
      console.log(error)
      throw error
    }
  }

  // check if rpc is available
  async function ensureRpcAvailable() {
    try {
      const rpcNetwork = SOROBAN_RPC_URLS[rpcStore.network as RPCcluster]

      const res = await fetch(String(rpcNetwork), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          jsonrpc: '2.0',
          id: 1,
          method: 'getHealth',
        }),
      })

      const data = await res.json()

      return res.ok && !data.error
    } catch {
      throw new Error('RPC not available! Please try again later.')
    }
  }

  function requireWallet() {
    if (!publicKey.value) {
      throw new Error('Wallet not connected')
    }
    return publicKey.value
  }

  async function runAction(opts: {
    client: StellarClient
    market: string
    pool: string
    type: TableActionType
    title: string
    body: string
    withObligation?: boolean
    exec: () => Promise<{ txHash?: string }>
    action?: () => void | Promise<void>
    reset?: () => void
  }) {
    const { pool, type, title, body, exec, market, client } = opts
    marketsStore.activeActionPool = {
      market,
      poolAddress: pool,
      poolActionType: type,
    }
    const info = await toast.create({
      title,
      body,
      modelValue: 30_000,
      variant: 'info',
      noProgress: false,
    })
    try {
      // Ensure RPC is available
      await ensureRpcAvailable()
      const res = await withTimeoutAbort(exec(), TRANSACTION_TIMEOUT)
      opts?.reset?.()
      await reloadData({
        pool_address: pool,
        market,
        client,
        withObligation: opts?.withObligation,
        action: opts?.action,
      })
      toast.create({
        title: `${title} Success`,
        body: 'Transaction sent successfully',
        modelValue: 10_000,
        actions: res?.txHash
          ? [{ label: 'View Transaction', href: generateExplorerLink(String(res.txHash)), target: '_blank' }]
          : [],
      })
      await recentStore.fetchAndUpdateLastTx()
    } catch (error: any) {
      if (error?.message.includes('rejected')) {
        return
      }
      toast.create({
        title: `${title} Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    } finally {
      marketsStore.activeActionPool = {
        market: undefined,
        poolAddress: undefined,
        poolActionType: undefined,
      }
      info?.dismiss()
    }
  }

  // Deposit
  async function deposit(
    props: {
      market: string
      client: StellarClient
      pool_address: string
      amount: number
      asset_data: string
    },
  ) {
    const pk = requireWallet()
    const { market, client, pool_address, amount, asset_data } = props
    const { asset_code, asset_issuer, symbol } = parseAsset(asset_data)
    const balance = asset_code === 'native' ? nativeBalance.value : getAssetBalance(asset_issuer)

    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      if (balance < amount) {
        throw new Error('Insufficient balance')
      }
    } catch (error: any) {
      toast.create({
        title: `Deposit Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const oblKey = buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'deposit',
      title: 'Deposit',
      body: `Sending transaction to deposit ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.deposit(oblKey, pool_address, amount, assetDecimals.value, kit.value),
      reset: () => {
        depositAmount.value = undefined
        marketsStore.dialogSupply = false
      },
    })
  }

  // Borrow
  async function borrow(
    props: {
      client: StellarClient
      market: string
      pool_address: string
      amount: number
      asset_data: string
      poolBorrowLimit: number
      withBuffer: boolean
    },
  ) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, asset_data, poolBorrowLimit, withBuffer } = props
    try {
      if (poolBorrowLimit < amount) {
        throw new Error('Borrow limit exceeded')
      }
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
    } catch (error: any) {
      toast.create({
        title: `Borrow Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const { symbol } = parseAsset(asset_data)

    const oblKey = buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'borrow',
      title: 'Borrow',
      body: `Sending transaction to borrow ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.borrowing.borrow(oblKey, pool_address, amount, assetDecimals.value, kit.value, withBuffer),
      reset: () => {
        borrowAmount.value = undefined
        marketsStore.dialogBorrow = false
        agreeToBorrow.value = true
      },
    })
  }

  // Withdraw
  async function withdraw(
    props: {
      client: StellarClient
      market: string
      pool_address: string
      amount: number
      limit: number
      asset_data: string
      withBuffer: boolean
    }) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, limit, asset_data, withBuffer } = props
    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      if (amount > limit) {
        throw new Error('Withdraw limit exceeded')
      }
    } catch (error: any) {
      toast.create({
        title: `Withdraw Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const { symbol } = parseAsset(asset_data)

    const oblKey = buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw',
      body: `Sending transaction to withdraw ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.withdraw(oblKey, pool_address, amount, assetDecimals.value, kit.value, withBuffer),
      reset: () => {
        withdrawAmount.value = undefined
        marketsStore.dialogWithdraw = false
      },
    })
  }

  // Repay
  async function repay(
    props: {
      client: StellarClient
      market: string
      pool_address: string
      amount: number
      limit: number
      asset_data: string
      withBuffer: boolean
    },
  ) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, limit, asset_data, withBuffer } = props
    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      if (amount > limit) {
        throw new Error('Withdraw limit exceeded')
      }
    } catch (error: any) {
      toast.create({
        title: `Repay Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const { symbol } = parseAsset(asset_data)

    const increasedAmount = withBuffer ? amount * 1.05 : amount

    const oblKey = buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'repay',
      title: 'Repay',
      body: `Sending transaction to repay ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.borrowing.repay(oblKey, pool_address, increasedAmount, assetDecimals.value, kit.value),
      reset: () => {
        repayAmount.value = undefined
        marketsStore.dialogRepay = false
      },
    })
  }

  // Add collateral
  async function addCollateral(
    props: {
      client: StellarClient
      market: string
      pool_address: string
      amount: number
      asset_data: string
    },
  ) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, asset_data } = props
    const { asset_code, asset_issuer, symbol } = parseAsset(asset_data)
    const balance = asset_code === 'native' ? nativeBalance.value : getAssetBalance(asset_issuer)

    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      if (balance < amount) {
        throw new Error('Insufficient balance')
      }
    } catch (error: any) {
      toast.create({
        title: `Collateral Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const oblKey = buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'deposit',
      title: 'Add Collateral',
      body: `Sending transaction to add collateral ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.addCollateral(oblKey, pool_address, amount, assetDecimals.value, kit.value),
      reset: () => {
        depositAmount.value = undefined
        marketsStore.dialogSupply = false
      },
    })
  }

  // Remove collateral
  async function removeCollateral(
    props: {
      client: StellarClient
      market: string
      pool_address: string
      amount: number
      limit: number
      asset_data: string
      withBuffer: boolean
    }) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, limit, asset_data, withBuffer } = props
    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      if (amount > limit) {
        throw new Error('Withdraw limit exceeded')
      }
    } catch (error: any) {
      toast.create({
        title: `Collateral Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const { symbol } = parseAsset(asset_data)

    const oblKey = buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw Collateral',
      body: `Sending transaction to withdraw collateral ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.removeCollateral(oblKey, pool_address, amount, assetDecimals.value, kit.value, withBuffer),
      reset: () => {
        withdrawAmount.value = undefined
        marketsStore.dialogWithdraw = false
      },
    })
  }

  // Leverage
  async function openMultiply(
    props: {
      client: StellarClient
      market: string
      deposit_pool_address: string
      borrow_pool_address: string
      initial_amount: number
      leverage_multiplier: number
      margin_asset?: 'borrow' | 'deposit'
      slippage: number
      swap_provider: string
      obligation_key?: ObligationKey
      path?: string[]
      action?: () => void | Promise<void>
      reset?: () => void
    },
  ) {
    const pk = requireWallet()
    const {
      client,
      market,
      deposit_pool_address,
      borrow_pool_address,
      initial_amount,
      leverage_multiplier,
      margin_asset,
      slippage,
      swap_provider,
      path,
    } = props

    try {
      if (!initial_amount || initial_amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
      if (!Number.isFinite(leverage_multiplier) || leverage_multiplier <= 1) {
        throw new Error('Multiplier should be greater than 1')
      }
    } catch (error: any) {
      toast.create({
        title: 'Multiply Error',
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const oblKey = props.obligation_key ?? buildObligationKey({ pablicKey: pk })

    await runAction({
      client,
      market,
      pool: deposit_pool_address,
      type: 'multiplyOpen',
      title: 'Open Multiply',
      body: `Sending multiply transaction`,
      action: props.action,
      exec: () => client.multiply.openPosition({
        user: oblKey,
        depositPoolAddress: deposit_pool_address,
        borrowPoolAddress: borrow_pool_address,
        initialAmount: initial_amount,
        leverageMultiplier: leverage_multiplier,
        marginAsset: margin_asset,
        slippagePercent: slippage,
        swapProviderAddress: swap_provider,
        path,
      }, kit.value),
      reset: props.reset,
    })
  }

  async function withdrawMultiply(
    props: {
      client: StellarClient
      market: string
      deposit_pool_address: string
      borrow_pool_address: string
      margin_asset?: 'borrow' | 'deposit'
      repay_amount?: number
      min_receive_amount?: number
      swap_provider: string
      obligation_key: ObligationKey
      path?: string[]
      action?: () => void | Promise<void>
      reset?: () => void
    },
  ) {
    const pk = requireWallet()
    const {
      client,
      market,
      deposit_pool_address,
      borrow_pool_address,
      margin_asset,
      repay_amount,
      min_receive_amount,
      swap_provider,
      obligation_key,
      path,
    } = props

    if (obligation_key.user !== pk) {
      throw new Error('Invalid multiply obligation owner')
    }

    try {
      if (repay_amount != null && repay_amount <= 0) {
        throw new Error('Repay amount must be greater than 0')
      }
    } catch (error: any) {
      toast.create({
        title: 'Withdraw Multiply Error',
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    await runAction({
      client,
      market,
      pool: deposit_pool_address,
      type: 'withdrawLeverage',
      title: 'Withdraw Multiply',
      body: 'Sending transaction to close multiply position',
      withObligation: true,
      action: props.action,
      exec: () => client.multiply.closePosition({
        user: obligation_key,
        depositPoolAddress: deposit_pool_address,
        borrowPoolAddress: borrow_pool_address,
        marginAsset: margin_asset,
        repayAmount: repay_amount,
        minReceiveAmount: min_receive_amount,
        swapProviderAddress: swap_provider,
        path,
      }, kit.value),
      reset: props.reset,
    })
  }

  async function reloadData({
    pool_address,
    market,
    client,
    withObligation = true,
    action,
  }: { pool_address: string
    market: string
    client: StellarClient
    withObligation?: boolean
    action?: () => void | Promise<void> }) {
    const tasks = [
      () => marketsStore.updatePool(pool_address, market, client),
      () => loadBalances(),
      () => action?.(),
    ]
    if (withObligation) {
      tasks.push(() => userStore.updateUserObligation(market, client))
    }

    await Promise.allSettled(tasks.map(cb => cb()))
  }

  function isDisabled(pool_address: string, actionType: TableActionType, activeMarket: string) {
    const { market, poolAddress, poolActionType } = marketsStore.activeActionPool
    return poolAddress
      ? pool_address !== poolAddress || actionType !== poolActionType || market !== activeMarket
      : false
  }

  function isLoading(pool_address: string, actionType: TableActionType, activeMarket: string) {
    const { market, poolAddress, poolActionType } = marketsStore.activeActionPool
    return poolAddress
      ? pool_address === poolAddress && actionType === poolActionType && activeMarket === market
      : false
  }

  function amountToAssetDecimals(amount: number) {
    return formatPrice(amount, 0, assetDecimals.value)
  }

  return {
    assetDecimals,

    borrowAmount,
    depositAmount,
    withdrawAmount,
    repayAmount,

    collateralOnly,

    borrow,
    deposit,

    repay,
    withdraw,

    addCollateral,
    removeCollateral,

    openMultiply,
    withdrawMultiply,

    // leverage,
    // withdrawLeverage,

    addTrustLine,

    isDisabled,
    isLoading,
  }
}
