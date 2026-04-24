<script lang="ts" setup>
import { SWAP_PROVIDERS } from '@alula/client-sdk'
import aquaLogo from '~/assets/img/providers/aqua-logo.png'
import soroswapLogo from '~/assets/img/providers/soroswap-logo.jpg'

const providerIcons = {
  aquarius: aquaLogo,
  soroswap: soroswapLogo,
}

const swapProviderAddress = defineModel({ default: '' })

const providers = computed(() => {
  return Object.entries(SWAP_PROVIDERS).map(([name, provider]) => {
    return { label: name, value: provider, icon: providerIcons[name as keyof typeof providerIcons] }
  })
})
const selectedProvider = ref(providers.value[0])

watch(selectedProvider, (provider) => {
  if (!provider?.value) {
    return
  }
  swapProviderAddress.value = provider.value
})

let stop: (() => void) | undefined

// eslint-disable-next-line prefer-const
stop = watch(swapProviderAddress, (address) => {
  if (!address) { return }

  selectedProvider.value
    = providers.value.find(p => p.value === address) ?? providers.value[0]

  stop?.()
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
      <template #label>
        <span class="select-item">
          <img
            v-if="selectedProvider?.icon"
            :src="selectedProvider.icon"
            alt="provider icon"
          >
          {{ selectedProvider?.label }}
        </span>
      </template>

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
