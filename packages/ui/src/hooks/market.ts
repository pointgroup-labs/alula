export function useMarket() {
    const userStore = useUserStore()
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

    // Deposit
    async function deposit(pool_address: string, amount: number, asset_data: string) {
        let depositToast
        try {
            if (!wallet.publicKey) {
                throw new Error('Wallet not connected')
            }

            const [asset_code, asset_issuer] = asset_data.split(':')
            const balance = asset_code === 'native' ? wallet.nativeBalance : wallet.getAssetBalance(asset_issuer)

            if (balance < amount) {
                throw new Error('Insufficient balance')
            }
            if (!amount || amount <= 0) {
                throw new Error('Amount should be greater than 0')
            }

            marketsStore.poolDepositAddr = pool_address
            const asset = asset_code === 'native' ? 'XLM' : asset_code

            depositToast = await Toast.create({
                title: 'Deposit',
                body: `Sending transaction to deposit ${amount} ${asset}`,
                modelValue: 30_000,
                variant: 'info',
                noProgress: false,
            })

            const res = await jLendClient.value.sdk.deposit(wallet.publicKey, pool_address, amount, connectionStore.kit)

            await reloadData(pool_address)
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
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Deposit Error',
                body: String(message),
                variant: 'danger',
                // alertProps: {
                //     variant: 'error',
                // },
            })
            throw error
        } finally {
            marketsStore.poolDepositAddr = undefined
            depositToast?.dismiss()
        }
    }

    // Borrow
    async function borrow(pool_address: string, amount: number, asset_data: string, limit: number) {
        let borrowToast
        try {
            if (!wallet.publicKey) {
                throw new Error('Wallet not connected')
            }

            const [asset_code] = asset_data.split(':')

            if (limit < amount) {
                throw new Error('Borrow limit exceeded')
            }
            if (!amount || amount <= 0) {
                throw new Error('Amount should be greater than 0')
            }

            marketsStore.poolDepositAddr = pool_address
            const asset = asset_code === 'native' ? 'XLM' : asset_code

            borrowToast = await Toast.create({
                title: 'Deposit',
                body: `Sending transaction to deposit ${amount} ${asset}`,
                modelValue: 30_000,
                variant: 'info',
                noProgress: false,
            })

            const res = await jLendClient.value.sdk.borrow(wallet.publicKey, pool_address, amount, connectionStore.kit)

            await reloadData(pool_address)

            Toast.create({
                title: 'Borrow Success',
                body: `You borrowed ${amount} ${asset} successfully`,
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
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Borrow Error',
                body: String(message),
                variant: 'danger',

            })
            throw error
        } finally {
            marketsStore.poolDepositAddr = undefined
            borrowToast?.dismiss()
        }
    }

    async function reloadData(pool_address: string) {
        await Promise.all([
            marketsStore.updatePools(pool_address),
            userStore.loadUserObligation(),
            wallet.loadBalances(),
        ])
    }

    return {
        borrow,
        deposit,

        addTrustLine,
    }
}
