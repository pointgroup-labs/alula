import { generateExplorerLink } from '~/utils'

export function useMarket() {
    const userStore = useUserStore()
    const marketsStore = useMarketsStore()
    const connectionStore = useConnectionStore()
    const clientStore = useClientStore()
    const jLendClient = computed(() => clientStore.jLendClient)

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
                        href: generateExplorerLink(String(res.txHash)),
                    },
                ],
            })
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Deposit Error',
                body: String(message),
                variant: 'danger',
                modelValue: 10_000,
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

            if (limit < amount) {
                throw new Error('Borrow limit exceeded')
            }
            if (!amount || amount <= 0) {
                throw new Error('Amount should be greater than 0')
            }

            const [asset_code] = asset_data.split(':')

            marketsStore.poolDepositAddr = pool_address
            const asset = asset_code === 'native' ? 'XLM' : asset_code

            borrowToast = await Toast.create({
                title: 'Borrow',
                body: `Sending transaction to borrow ${amount} ${asset}`,
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
                        href: generateExplorerLink(String(res.txHash)),
                    },
                ],
            })
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Borrow Error',
                body: String(message),
                variant: 'danger',
                modelValue: 10_000,
            })
            throw error
        } finally {
            marketsStore.poolDepositAddr = undefined
            borrowToast?.dismiss()
        }
    }

    // Withdraw
    async function withdraw(pool_address: string, amount: number, limit: number, asset_code: string) {
        let withdrawToast
        try {
            if (!wallet.publicKey) {
                throw new Error('Wallet not connected')
            }

            if (!amount || amount <= 0) {
                throw new Error('Amount should be greater than 0')
            }

            if (amount > limit) {
                throw new Error('Withdraw limit exceeded')
            }

            marketsStore.poolDepositAddr = pool_address
            const asset = asset_code === 'native' ? 'XLM' : asset_code

            withdrawToast = await Toast.create({
                title: 'Withdraw',
                body: `Sending transaction to withdraw ${amount} ${asset}`,
                modelValue: 30_000,
                variant: 'info',
                noProgress: false,
            })

            const res = await jLendClient.value.sdk.withdraw(wallet.publicKey, pool_address, amount, connectionStore.kit)

            await reloadData(pool_address)

            Toast.create({
                title: 'Withdraw Success',
                body: `You withdrew ${amount} ${asset} successfully`,
                modelValue: 30_000,
                alertProps: {
                    variant: 'success',
                },
                actions: [
                    {
                        label: 'View Transaction',
                        href: generateExplorerLink(String(res.txHash)),
                    },
                ],
            })
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Withdraw Error',
                body: String(message),
                variant: 'danger',
                modelValue: 10_000,
            })
            throw error
        } finally {
            marketsStore.poolDepositAddr = undefined
            withdrawToast?.dismiss()
        }
    }

    // Repay
    async function repay(pool_address: string, amount: number, limit: number, asset_code: string) {
        let withdrawToast
        try {
            if (!wallet.publicKey) {
                throw new Error('Wallet not connected')
            }

            if (!amount || amount <= 0) {
                throw new Error('Amount should be greater than 0')
            }

            if (amount > limit) {
                throw new Error('You don`t have enough balance to repay')
            }

            marketsStore.poolDepositAddr = pool_address
            const asset = asset_code === 'native' ? 'XLM' : asset_code

            withdrawToast = await Toast.create({
                title: 'Repay',
                body: `Sending transaction to repay ${amount} ${asset}`,
                modelValue: 30_000,
                variant: 'info',
                noProgress: false,
            })

            const res = await jLendClient.value.sdk.repay(wallet.publicKey, pool_address, amount, connectionStore.kit)

            await reloadData(pool_address)

            Toast.create({
                title: 'Repay Success',
                body: `You repaid ${amount} ${asset} successfully`,
                modelValue: 30_000,
                alertProps: {
                    variant: 'success',
                },
                actions: [
                    {
                        label: 'View Transaction',
                        href: generateExplorerLink(String(res.txHash)),
                    },
                ],
            })
        } catch (error: any) {
            const message = error?.message || error
            Toast.create({
                title: 'Repay Error',
                body: String(message),
                variant: 'danger',
                modelValue: 10_000,
            })
            throw error
        } finally {
            marketsStore.poolDepositAddr = undefined
            withdrawToast?.dismiss()
        }
    }

    async function reloadData(pool_address: string) {
        await Promise.all([
            marketsStore.updatePools(pool_address),
            userStore.loadUserObligation(),
            wallet.loadBalances(),
        ])
    }

    function isDisabled(pool_address: string) {
        return marketsStore.poolDepositAddr && pool_address !== marketsStore.poolDepositAddr
    }

    function isLoading(pool_address: string) {
        return marketsStore.poolDepositAddr && pool_address === marketsStore.poolDepositAddr
    }

    return {
        borrow,
        deposit,

        repay,
        withdraw,

        addTrustLine,

        isDisabled,
        isLoading,
    }
}
