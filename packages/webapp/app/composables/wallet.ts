const _useWalletComposable = () => {
  const wallet = useWallet()
  const {
    publicKey,
    balances,
    nativeBalance,
  } = storeToRefs(wallet)

  return {
    publicKey,
    balances,
    nativeBalance,
    initWallet: wallet.initWallet,
    loadBalances: wallet.loadBalances,
    getAssetBalance: wallet.getAssetBalance,
  }
}

export const useWalletComposable = createSharedComposable(_useWalletComposable)
