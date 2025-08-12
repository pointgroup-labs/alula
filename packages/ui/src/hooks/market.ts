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
  const clientStore = useClientStore()
  const jLendClient = computed(() => clientStore.jLendClient)

  const { generateExplorerLink } = useExplorerLink()

  const collateralOnly = ref(false)

  const depositAmount = ref()
  const borrowAmount = ref()
  const withdrawAmount = ref()
  const repayAmount = ref()

  const Toast = useToast()

  const wallet = useWallet()

  const kit = computed(() => connectionStore.kit)

  async function addTrustLine(asset: string, issuer: string) {
    try {
      if (!wallet.publicKey) {
        return
      }
      const res = await jLendClient.value.addTrustlineTx(wallet.publicKey, asset, issuer, connectionStore.kit)
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
    pool: string
    type: TableActionType
    title: string
    body: string
    exec: () => Promise<{ txHash?: string }>
  }) {
    marketsStore.poolDepositAddr = opts.pool
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
      await reloadData(opts.pool)
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
      marketsStore.poolDepositAddr = undefined
      marketsStore.poolActionType = undefined
      info?.dismiss()
    }
  }

  // Deposit
  async function deposit(
    pool_address: string,
    amount: number,
    asset_data: string,
  ) {
    const pk = requireWallet()
    const { asset_code, asset_issuer, symbol } = parseAsset(asset_data)
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }
    if (balance < amount) {
      throw new Error('Insufficient balance')
    }

    await runAction({
      pool: pool_address,
      type: 'deposit',
      title: 'Deposit',
      body: `Sending transaction to deposit ${amount} ${symbol}`,
      exec: () => jLendClient.value.sdk.deposit(pk, pool_address, amount, kit.value),
    })

    depositAmount.value = undefined
  }

  // Borrow
  async function borrow(
    pool_address:
    string,
    amount: number,
    asset_data: string,
    limit: number,
  ) {
    const pk = requireWallet()

    if (limit < amount) {
      throw new Error('Borrow limit exceeded')
    }
    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    const { symbol } = parseAsset(asset_data)

    await runAction({
      pool: pool_address,
      type: 'borrow',
      title: 'Borrow',
      body: `Sending transaction to borrow ${amount} ${symbol}`,
      exec: () => jLendClient.value.sdk.borrow(pk, pool_address, amount, kit.value),
    })

    borrowAmount.value = undefined
  }

  // Withdraw
  async function withdraw(
    pool_address: string,
    amount: number,
    limit: number,
    asset_data: string) {
    const pk = requireWallet()

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (amount > limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(asset_data)

    await runAction({
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw',
      body: `Sending transaction to withdraw ${amount} ${symbol}`,
      exec: () => jLendClient.value.sdk.withdraw(pk, pool_address, amount, kit.value),
    })

    withdrawAmount.value = undefined
  }

  // Repay
  async function repay(
    pool_address: string,
    amount: number,
    limit: number,
    asset_data: string,
  ) {
    const pk = requireWallet()

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (amount > limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(asset_data)

    const increasedAmount = amount * 1.01

    await runAction({
      pool: pool_address,
      type: 'repay',
      title: 'Repay',
      body: `Sending transaction to repay ${amount} ${symbol}`,
      exec: () => jLendClient.value.sdk.repay(pk, pool_address, increasedAmount, kit.value),
    })

    repayAmount.value = undefined
  }

  // Add collateral
  async function addCollateral(
    pool_address: string,
    amount: number,
    asset_data: string,
  ) {
    const pk = requireWallet()
    const { asset_code, asset_issuer, symbol } = parseAsset(asset_data)
    console.log(asset_data)
    const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (balance < amount) {
      throw new Error('Insufficient balance')
    }

    await runAction({
      pool: pool_address,
      type: 'deposit',
      title: 'Add Collateral',
      body: `Sending transaction to add collateral ${amount} ${symbol}`,
      exec: () => jLendClient.value.sdk.addCollateral(pk, pool_address, amount, kit.value),
    })

    depositAmount.value = undefined
  }

  // Remove collateral
  async function removeCollateral(
    pool_address: string,
    amount: number,
    limit: number,
    asset_data: string) {
    const pk = requireWallet()

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    if (amount > limit) {
      throw new Error('Withdraw limit exceeded')
    }

    const { symbol } = parseAsset(asset_data)

    await runAction({
      pool: pool_address,
      type: 'withdraw',
      title: 'Withdraw Collateral',
      body: `Sending transaction to withdraw collateral ${amount} ${symbol}`,
      exec: () => jLendClient.value.sdk.removeCollateral(pk, pool_address, amount, kit.value),
    })

    withdrawAmount.value = undefined
  }

  // Leverage
  async function leverage(
    deposit_pool_address: string,
    borrow_pool_address: string,
    deposit_as_margin: boolean,
    amount: number,
    leverage_multiplier: number,
    asset_code: string,
  ) {
    const pk = requireWallet()

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    await runAction({
      pool: deposit_pool_address,
      type: 'leverage',
      title: 'Leverage',
      body: `Sending transaction to leverage ${amount} ${asset_code}`,
      exec: () => jLendClient.value.sdk.leverage(
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
    deposit_pool_address: string,
    borrow_pool_address: string,
    amount: number,
    asset_code: string,
  ) {
    const pk = requireWallet()

    if (!amount || amount <= 0) {
      throw new Error('Amount should be greater than 0')
    }

    await runAction({
      pool: deposit_pool_address,
      type: 'withdrawLeverage',
      title: 'Leverage',
      body: `Sending transaction to Withdraw leverage ${amount} ${asset_code}`,
      exec: () => jLendClient.value.sdk.withdrawLeverage(
        pk,
        deposit_pool_address,
        borrow_pool_address,
        amount,
        connectionStore.kit),
    })

    withdrawAmount.value = undefined
  }

  async function reloadData(pool_address: string) {
    await Promise.all([
      marketsStore.updatePools(pool_address),
      userStore.loadUserObligation(),
      wallet.loadBalances(),
    ])
  }

  function isDisabled(pool_address: string, actionType: TableActionType) {
    return marketsStore.poolDepositAddr
      ? pool_address !== marketsStore.poolDepositAddr || marketsStore.poolActionType !== actionType
      : false
  }

  function isLoading(pool_address: string, actionType: TableActionType) {
    return marketsStore.poolDepositAddr
      ? pool_address === marketsStore.poolDepositAddr && marketsStore.poolActionType === actionType
      : false
  }

  return {
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
