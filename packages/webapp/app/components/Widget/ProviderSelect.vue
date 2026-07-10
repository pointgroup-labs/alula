<script lang="ts" setup>
import type { RPCcluster } from '@alula/client-sdk'
import { SWAP_PROVIDERS } from '@alula/client-sdk'
import aquaLogo from '~/assets/img/providers/aqua-logo.png'
import soroswapLogo from '~/assets/img/providers/soroswap-logo.jpg'

const providerIcons = {
  aquarius: aquaLogo,
  soroswap: soroswapLogo,
}

const rpcStore = useRpcStore()
const swapProviderAddress = defineModel({ default: '' })

const providers = computed(() => {
  const network = rpcStore.network as RPCcluster | null
  if (!network) { return [] }
  return Object.entries(SWAP_PROVIDERS)
    .map(([name, addresses]) => ({ label: name, value: addresses[network] ?? '', icon: providerIcons[name as keyof typeof providerIcons] }))
    .filter(p => p.value)
})

const selectedProvider = ref(providers.value[0])

watch(selectedProvider, (provider) => {
  if (!provider?.value) { return }
  swapProviderAddress.value = provider.value
})

watch([providers, swapProviderAddress], ([newProviders, address]) => {
  selectedProvider.value = newProviders.find(p => p.value === address) ?? newProviders[0]
}, { immediate: true })
</script>

<template>
  <div class="provider-select">
    <span class="provider-select-label">
      Swap Provider
    </span>

    <j-select
      v-model="selectedProvider"
      :options="providers"
      :unselected="false"
    >
      <template #option="{ option }">
        <img
          :src="option.icon"
          alt="provider icon"
        >
        {{ option.label }}
      </template>
    </j-select>
  </div>
</template>

<style lang="scss">
.provider-select {
  display: flex;
  flex-direction: column;
  gap: 4px;
  &-label {
    font-size: 11px;
    font-weight: 500;
    color: #6b7994;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .j-select {
    .btn {
      height: 28px;
      font-size: 12px;
      padding: 0 $spacing-lg;
      background-color: transparent;
      border-radius: 6px;
      outline: 1px solid $navi-400;
      text-transform: capitalize;

      &[aria-expanded='true'] {
        outline-color: $navi-200;
      }
    }

    .select-item {
      text-transform: capitalize;
      display: flex;
      align-items: center;
      gap: 6px;

      img {
        width: 16px;
        height: 16px;
        object-fit: contain;
        border-radius: 50%;
      }
    }
  }
}
</style>
