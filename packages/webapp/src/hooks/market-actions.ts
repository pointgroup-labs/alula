import type { StellarClient } from '@alula/client-sdk'
import type { TableActionType } from '~/store/markets'
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
      const res = await marketclient.value!.wallet.addTrustline(wallet.publicKey, asset, issuer, connectionStore.kit)
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
        alertProps: { variant: 'success' },
        actions: res?.txHash
          ? [{ label: 'View Transaction', href: generateExplorerLink(String(res.txHash)) }]
          : [],
      })
      await recentStore.fetchAndUpdateLastTx()
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

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'deposit',
      title: 'Deposit',
      body: `Sending transaction to deposit ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.deposit(pk, pool_address, amount, kit.value),
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
    },
  ) {
    const pk = requireWallet()
    const { client, market, pool_address, amount, asset_data, poolBorrowLimit } = props
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

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'borrow',
      title: 'Borrow',
      body: `Sending transaction to borrow ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.borrowing.borrow(pk, pool_address, amount, kit.value),
      reset: () => {
        borrowAmount.value = undefined
        marketsStore.dialogBorrow = false
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

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw',
      body: `Sending transaction to withdraw ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.withdraw(pk, pool_address, amount, kit.value, withBuffer),
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

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'repay',
      title: 'Repay',
      body: `Sending transaction to repay ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.borrowing.repay(pk, pool_address, increasedAmount, kit.value),
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
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

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

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'deposit',
      title: 'Add Collateral',
      body: `Sending transaction to add collateral ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.addCollateral(pk, pool_address, amount, kit.value),
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

    await runAction({
      client,
      market,
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw Collateral',
      body: `Sending transaction to withdraw collateral ${amountToAssetDecimals(amount)} ${symbol}`,
      exec: () => client!.lending.removeCollateral(pk, pool_address, amount, kit.value, withBuffer),
      reset: () => {
        withdrawAmount.value = undefined
        marketsStore.dialogWithdraw = false
      },
    })
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
    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
    } catch (error: any) {
      toast.create({
        title: `Leverage Error`,
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
      type: 'leverage',
      title: 'Leverage',
      body: `Sending transaction to leverage ${amountToAssetDecimals(amount)} ${asset_code}`,
      withObligation: false,
      action: props.action,
      exec: () => client!.leverage.depositWithLeverage(
        {
          user: pk,
          depositPoolAddress: deposit_pool_address,
          borrowPoolAddress: borrow_pool_address,
          depositAsMargin: deposit_as_margin,
          amount,
          leverageMultiplier: leverage_multiplier,
        },
        kit.value),
      reset: () => {
        depositAmount.value = undefined
        marketsStore.dialogLeverage = false
      },
    })
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
    try {
      if (!amount || amount <= 0) {
        throw new Error('Amount should be greater than 0')
      }
    } catch (error: any) {
      toast.create({
        title: `Withdraw Leverage Error`,
        body: String(error?.message || error),
        variant: 'danger',
        modelValue: 10_000,
      })
      throw error
    }

    const increasedAmount = amount * 1.05

    await runAction({
      client,
      market,
      pool: deposit_pool_address,
      type: 'withdrawLeverage',
      title: 'Leverage',
      body: `Sending transaction to Withdraw leverage ${amountToAssetDecimals(amount)} ${asset_code}`,
      withObligation: false,
      action: props.action,
      exec: () => client!.leverage.withdrawFromLeveraged(
        {
          user: pk,
          depositPoolAddress: deposit_pool_address,
          borrowPoolAddress: borrow_pool_address,
          amount: increasedAmount,
        },
        connectionStore.kit),
      reset: () => {
        withdrawAmount.value = undefined
        marketsStore.dialogLeverageWithdraw = false
      },
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
      () => wallet.loadBalances(),
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

    leverage,
    withdrawLeverage,

    addTrustLine,

    isDisabled,
    isLoading,
  }
}
