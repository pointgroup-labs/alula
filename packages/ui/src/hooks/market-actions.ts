import type { StellarClient } from '@alula/client-sdk'
import type { TableActionType } from '~/store/markets'
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
  const marketclient = computed(() => marketsStore.marketClient)

  const { generateExplorerLink } = useExplorerLink()

  const collateralOnly = ref(false)

  const depositAmount = ref()
  const borrowAmount = ref()
  const withdrawAmount = ref()
  const repayAmount = ref()

  const toast = useToast()

  const wallet = useWallet()

  const assetDecimals = computed(() => marketsStore.assetDecimals)

  const kit = computed(() => connectionStore.kit)

  async function addTrustLine(asset: string, issuer: string) {
    try {
      if (!wallet.publicKey) {
        return
      }
      const res = await marketclient.value!.addTrustlineTx(wallet.publicKey, asset, issuer, connectionStore.kit)
      await wallet.loadBalances()
      return res
    } catch (error) {
      console.log(error)
      throw error
    }
  }

  function requireWallet() {
    if (!wallet.publicKey) {
      throw new Error('Wallet not connected')
    }
    return wallet.publicKey
  }

  async function runAction(opts: {
    client: StellarClient
    market: string
    pool: string
    type: TableActionType
    title: string
    body: string
    exec: () => Promise<{ txHash?: string }>
    action?: () => void | Promise<void>
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
      const res = await exec()
      await reloadData(pool, market, client, opts?.action)
      toast.create({
        title: `${title} Success`,
        body: 'Transaction sent successfully',
        modelValue: 30_000,
        alertProps: { variant: 'success' },
        actions: res?.txHash
          ? [{ label: 'View Transaction', href: generateExplorerLink(String(res.txHash)) }]
          : [],
      })
    } catch (error: any) {
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
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }
    if (balance < amount) {
      throw new Error('Insufficient balance')
    }

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'deposit',
      title: 'Deposit',
      body: `Sending transaction to deposit ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.marketSdk.depositToLending(pk, pool_address, amount, kit.value),
    })

    depositAmount.value = undefined
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
    },
  ) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, asset_data, poolBorrowLimit } = props
    if (poolBorrowLimit < amount) {
      throw new Error('Borrow limit exceeded')
    }
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    const { symbol } = parseAsset(asset_data)

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'borrow',
      title: 'Borrow',
      body: `Sending transaction to borrow ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.marketSdk.borrowLendingAsset(pk, pool_address, amount, kit.value),
    })

    borrowAmount.value = undefined
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
    }) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, limit, asset_data } = props
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (amount > limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(asset_data)

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw',
      body: `Sending transaction to withdraw ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.marketSdk.wathdrawDeposit(pk, pool_address, amount, kit.value),
    })

    withdrawAmount.value = undefined
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
    },
  ) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, limit, asset_data } = props
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (amount > limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(asset_data)

    const increasedAmount = amount

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'repay',
      title: 'Repay',
      body: `Sending transaction to repay ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.marketSdk.repayBorrow(pk, pool_address, increasedAmount, kit.value),
    })

    repayAmount.value = undefined
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
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (balance < amount) {
      throw new Error('Insufficient balance')
    }

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'deposit',
      title: 'Add Collateral',
      body: `Sending transaction to add collateral ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.marketSdk.addCollateral(pk, pool_address, amount, kit.value),
    })

    depositAmount.value = undefined
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
    }) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, limit, asset_data } = props
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (amount > limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(asset_data)

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw Collateral',
      body: `Sending transaction to withdraw collateral ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.marketSdk.removeCollateral(pk, pool_address, amount, kit.value),
    })

    withdrawAmount.value = undefined
  }

  // Leverage
  async function leverage(
    props: {
      client: StellarClient
      market: string
      deposit_pool_address: string
      borrow_pool_address: string
      deposit_as_margin: boolean
      amount: number
      leverage_multiplier: number
      asset_code: string
      action?: () => void | Promise<void>
    },
  ) {
    const pk = requireWallet()
    const { client, market, deposit_pool_address, borrow_pool_address, deposit_as_margin, amount, leverage_multiplier, asset_code } = props
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    await runAction({
      client,
      market,
      pool: deposit_pool_address,
      type: 'leverage',
      title: 'Leverage',
      body: `Sending transaction to leverage ${amountToAssetDecimals(amount)} ${asset_code}`,
      action: props.action,
      exec: () => client!.marketSdk.leverage(
        pk,
        deposit_pool_address,
        borrow_pool_address,
        deposit_as_margin,
        amount,
        leverage_multiplier,
        kit.value),
    })

    withdrawAmount.value = undefined
  }

  // Withdraw Leverage
  async function withdrawLeverage(
    props: {
      client: StellarClient
      market: string
      deposit_pool_address: string
      borrow_pool_address: string
      amount: number
      asset_code: string
      action?: () => void | Promise<void>
    },
  ) {
    const pk = requireWallet()
    const { client, market, deposit_pool_address, borrow_pool_address, amount, asset_code } = props
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    await runAction({
      client,
      market,
      pool: deposit_pool_address,
      type: 'withdrawLeverage',
      title: 'Leverage',
      body: `Sending transaction to Withdraw leverage ${amountToAssetDecimals(amount)} ${asset_code}`,
      action: props.action,
      exec: () => client!.marketSdk.withdrawLeverage(
        pk,
        deposit_pool_address,
        borrow_pool_address,
        amount,
        connectionStore.kit),
    })

    withdrawAmount.value = undefined
  }

  async function reloadData(pool_address: string, market: string, client: StellarClient, action?: () => void | Promise<void>) {
    await Promise.all([
      marketsStore.updatePool(pool_address, market, client),
      wallet.loadBalances(),
      userStore.updateUserObligation(market, client),
      action?.(),
    ])
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

    leverage,
    withdrawLeverage,

    addTrustLine,

    isDisabled,
    isLoading,
  }
}
