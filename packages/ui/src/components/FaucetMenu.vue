<script lang="ts" setup>
import { getTokenIcon } from '~/utils'

const toast = useToast()

const rpcStore = useRpcStore()

const isTestNet = computed(() => rpcStore.network === 'testnet')

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const clientStore = useClientStore()
const alulaClient = computed(() => clientStore.alulaClient)

const loading = ref(false)

async function faucet() {
  try {
    if (alulaClient.value?.marketSdk?.rpc !== 'testnet') {
      return
    }
    loading.value = true
    const faucetToast = await toast.create({
      title: 'Requesting Faucet',
      variant: 'info',
      noProgress: false,
      modelValue: 20_000,
    })

    const res = await fetch(`https://friendbot.stellar.org/?addr=${publicKey.value}`)
    const data = await res.json()

    faucetToast?.dismiss()

    toast.create({
      title: data?.title || 'Faucet',
      body: data?.detail || 'Funds have been successfully added to your balance.',
      variant: 'info',
    })
    if (res?.ok) {
      await wallet.loadBalances()
    }
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <j-btn
    v-if="publicKey && isTestNet"
    pill
    size="lg"
    class="faucet-btn"
    variant="accent"
    :loading="loading"
    @click="faucet"
  >
    <img
      :src="getTokenIcon('native')"
      alt="XLM"
    > Faucet XLM - TestNet
  </j-btn>
</template>

<style lang="scss">
.faucet-btn {
  width: 100%;

  .btn-content {
    img {
      width: 20px;
      height: 20px;
    }
  }
}
</style>
