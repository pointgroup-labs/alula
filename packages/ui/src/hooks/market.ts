export function useMarket() {
    const marketsStore = useMarketsStore()
    const connectionStore = useConnectionStore()
    const jLendClient = computed(() => connectionStore.jLendClient)

    const Toast = useToast()

    const wallet = useWallet()

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

    async function deposit(publicKey: string, pool_address: string, amount: number, asset_code: string) {
        let toast
        try {
            marketsStore.poolDepositAddr = pool_address

            if (!amount || amount <= 0) {
                throw new Error('Amount should be greater than 0')
            }
            const asset = asset_code === 'native' ? 'XLM' : asset_code
            toast = await Toast.create({
                title: 'Deposit',
                body: `Sending transaction to deposit ${amount} ${asset}`,
                modelValue: 30_000,
                variant: 'info',
                noProgress: false,
            })

            const res = await jLendClient.value.sdk.deposit(publicKey, pool_address, amount, connectionStore.kit)

            const poolInfo = await jLendClient.value.sdk.getPoolInfo(pool_address)
            await marketsStore.updatePools(pool_address)
            Toast.create({
                title: 'Deposit Success',
                body: `You deposited ${amount} ${asset} successfully`,
                modelValue: 30_000,
                alertProps: {
                    variant: 'success',
                },
                actions: [
                    {
                        label: 'View Transaction',
                        href: `https://stellar.expert/explorer/testnet/tx/${res.txHash}`,
                    },
                ],
            })
            console.log('POOL_INFO', poolInfo)
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Deposit Error',
                body: String(message),
                alertProps: {
                    variant: 'error',
                },
            })
            throw error
        } finally {
            marketsStore.poolDepositAddr = undefined
            toast?.dismiss()
        }
    }

    return {
        deposit,

        addTrustLine,
    }
}
