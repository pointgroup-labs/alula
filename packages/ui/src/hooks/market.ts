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

export function useMarket() {
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

  const Toast = useToast()

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
    marketsStore.poolActiveAddress = opts.pool
    marketsStore.poolActionType = opts.type
    const info = await Toast.create({
      title: opts.title,
      body: opts.body,
      modelValue: 30_000,
      variant: 'info',
      noProgress: false,
    })
    try {
      const res = await opts.exec()
      await reloadData(opts.pool, opts.market, opts.client, opts?.action)
      Toast.create({
        title: `${opts.title} Success`,
        body: 'Transaction sent successfully',
        modelValue: 30_000,
        alertProps: { variant: 'success' },
        actions: res?.txHash
          ? [{ label: 'View Transaction', href: generateExplorerLink(String(res.txHash)) }]
          : [],
      })
    } catch (error: any) {
      Toast.create({
        title: `${opts.title} Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    } finally {
      marketsStore.poolActiveAddress = undefined
      marketsStore.poolActionType = undefined
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
    const { asset_code, asset_issuer, symbol } = parseAsset(props.asset_data)
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }
    if (balance < props.amount) {
      throw new Error('Insufficient balance')
    }

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.pool_address,
      type: 'deposit',
      title: 'Deposit',
      body: `Sending transaction to deposit ${props.amount} ${symbol}`,
      exec: () => props.client!.marketSdk.depositToLending(pk, props.pool_address, props.amount, kit.value),
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

    if (props.poolBorrowLimit < props.amount) {
      throw new Error('Borrow limit exceeded')
    }
    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    const { symbol } = parseAsset(props.asset_data)

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.pool_address,
      type: 'borrow',
      title: 'Borrow',
      body: `Sending transaction to borrow ${props.amount} ${symbol}`,
      exec: () => props.client!.marketSdk.borrowLendingAsset(pk, props.pool_address, props.amount, kit.value),
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

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (props.amount > props.limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(props.asset_data)

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.pool_address,
      type: 'withdraw',
      title: 'Withdraw',
      body: `Sending transaction to withdraw ${props.amount} ${symbol}`,
      exec: () => props.client!.marketSdk.wathdrawDeposit(pk, props.pool_address, props.amount, kit.value),
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

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (props.amount > props.limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(props.asset_data)

    const increasedAmount = props.amount * 1.01

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.pool_address,
      type: 'repay',
      title: 'Repay',
      body: `Sending transaction to repay ${props.amount} ${symbol}`,
      exec: () => props.client!.marketSdk.repayBorrow(pk, props.pool_address, increasedAmount, kit.value),
    })

    repayAmount.value = undefined
  }

  // Add collateral
  async function addCollateral(
    props: {
      market: string
      client: StellarClient
      pool_address: string
      amount: number
      asset_data: string
    },
  ) {
    const pk = requireWallet()
    const { asset_code, asset_issuer, symbol } = parseAsset(props.asset_data)
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (balance < props.amount) {
      throw new Error('Insufficient balance')
    }

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.pool_address,
      type: 'deposit',
      title: 'Add Collateral',
      body: `Sending transaction to add collateral ${props.amount} ${symbol}`,
      exec: () => props.client!.marketSdk.addCollateral(pk, props.pool_address, props.amount, kit.value),
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

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (props.amount > props.limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(props.asset_data)

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.pool_address,
      type: 'withdraw',
      title: 'Withdraw Collateral',
      body: `Sending transaction to withdraw collateral ${props.amount} ${symbol}`,
      exec: () => props.client!.marketSdk.removeCollateral(pk, props.pool_address, props.amount, kit.value),
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

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.deposit_pool_address,
      type: 'leverage',
      title: 'Leverage',
      body: `Sending transaction to leverage ${props.amount} ${props.asset_code}`,
      action: props.action,
      exec: () => props.client!.marketSdk.leverage(
        pk,
        props.deposit_pool_address,
        props.borrow_pool_address,
        props.deposit_as_margin,
        props.amount,
        props.leverage_multiplier,
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

    if (!props.amount || props.amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    await runAction({
      client: props.client,
      market: props.market,
      pool: props.deposit_pool_address,
      type: 'withdrawLeverage',
      title: 'Leverage',
      body: `Sending transaction to Withdraw leverage ${props.amount} ${props.asset_code}`,
      action: props.action,
      exec: () => props.client!.marketSdk.withdrawLeverage(
        pk,
        props.deposit_pool_address,
        props.borrow_pool_address,
        props.amount,
        connectionStore.kit),
    })

    withdrawAmount.value = undefined
  }

  async function reloadData(pool_address: string, market: string, client: StellarClient, action?: () => void | Promise<void>) {
    await Promise.all([
      marketsStore.updatePools(pool_address, market, client),
      wallet.loadBalances(),
      userStore.updateUserObligation(market, client),
      action?.(),
    ])
  }

  function isDisabled(pool_address: string, actionType: TableActionType) {
    return marketsStore.poolActiveAddress
      ? pool_address !== marketsStore.poolActiveAddress || marketsStore.poolActionType !== actionType
      : false
  }

  function isLoading(pool_address: string, actionType: TableActionType) {
    if (actionType === 'leverage') {
      console.log('poolActiveAddress', marketsStore.poolActiveAddress)
      console.log('POOL_ADDRESS', pool_address)
      console.log('actionType', actionType)
    }
    return marketsStore.poolActiveAddress
      ? pool_address === marketsStore.poolActiveAddress && marketsStore.poolActionType === actionType
      : false
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
