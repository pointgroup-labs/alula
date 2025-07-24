<script lang="ts" setup>
const Toast = useToast()

const wallet = useWallet()
const publicKey = computed(() => wallet.publicKey)

const clientStore = useClientStore()
const jlendClient = computed(() => clientStore.jLendClient)

async function faucet() {
  if (jlendClient.value?.sdk?.rpc !== 'testnet') {
    return
  }
  let faucetToast
  // eslint-disable-next-line prefer-const
  faucetToast = await Toast.create({
    title: 'Requesting Faucet',
    variant: 'info',
    noProgress: false,
    modelValue: 20_000,
  })

  const res = await fetch(`https://friendbot.stellar.org/?addr=${publicKey.value}`)
  const data = await res.json()

  faucetToast?.dismiss()

  Toast.create({
    title: data?.title || 'Faucet',
    body: data?.detail || 'Funds have been successfully added to your balance.',
    variant: 'info',
  })
  if (res?.ok) {
    await wallet.loadBalances()
  }
}

onMounted(() => {
  nextTick(() => {
    const target = document.querySelector('.app-logo')
    const menu = document.querySelector('#custom-menu') as HTMLElement

    target?.addEventListener('contextmenu', (e) => {
      const mouseEvent = e as MouseEvent
      if (!publicKey.value || jlendClient.value?.sdk?.rpc !== 'testnet') {
        return
      }
      mouseEvent.preventDefault()
      if (!menu) {
        return
      }
      menu.style.top = `${mouseEvent.clientY}px`
      menu.style.left = `${mouseEvent.clientX}px`
      menu.style.display = 'block'
    })

    document.addEventListener('click', () => {
      menu.style.display = 'none'
    })
  })
})
</script>

<template>
  <div
    id="custom-menu"
    style="
        position: absolute;
        display: none;
        background: white;
        border: 1px solid #ccc;
        box-shadow: 0 2px 10px rgba(0,0,0,0.2);
        z-index: 1000;
        padding: 10px;"
  >
    <div @click="faucet">
      Fauset XLM
    </div>
  </div>
</template>

<style lang="scss">
#custom-menu {
  div {
    cursor: pointer;
  }
}
</style>
