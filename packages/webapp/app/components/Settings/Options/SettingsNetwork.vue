<script lang="ts" setup>
import { capitalize } from 'vue'

const selected = ref()

const rpcStore = useRpcStore()

const activeNetwork = ref()

const horizonRPC = ref()
const sorobanRPC = ref()

const networks = Object.values(Network)

const isHorizenEdit = ref(false)
const isSorobanEdit = ref(false)

function handleShowEdit(network: 'horizon' | 'soroban') {
  if (network === 'horizon') {
    isHorizenEdit.value = !isHorizenEdit.value
  } else {
    isSorobanEdit.value = !isSorobanEdit.value
  }
  requestAnimationFrame(() => {
    if (network === 'horizon') {
      horizonRPC.value = rpcStore.customHorizonRpc
    } else {
      sorobanRPC.value = rpcStore.customSorobanRpc
    }
    focusInput(`.${network}-input`)
  })
}

function isValidUrl(url?: string) {
  return url && url?.startsWith('https://')
}

function handleEdit(network: 'horizon' | 'soroban', isSave: boolean) {
  if (network === 'horizon') {
    if (!isSave) {
      rpcStore.customHorizonRpc = ''
      handleShowEdit(network)
      return
    }
    if (!isValidUrl(horizonRPC.value)) {
      return
    }
    rpcStore.customHorizonRpc = horizonRPC.value
    handleShowEdit(network)
    return
  }

  if (network === 'soroban') {
    if (!isSave) {
      rpcStore.customSorobanRpc = ''
      handleShowEdit(network)
      return
    }
    if (!isValidUrl(sorobanRPC.value)) {
      return
    }
    rpcStore.customSorobanRpc = sorobanRPC.value
    handleShowEdit(network)
  }
}

const stellarRPCList = computed(() => activeNetwork.value === 'testnet' ? SOROBAN_TESTNET_RPC_URLS : SOROBAN_PUBLIC_RPC_URLS)

watch(activeNetwork, (n) => {
  rpcStore.setNetwork(n)
})

watch(() => rpcStore.network, (val) => {
  activeNetwork.value = val
}, { once: true, immediate: true })

watch(() => rpcStore.sorobanRPCUrl, (val) => {
  if (!import.meta.client) {
    return
  }
  selected.value = val && stellarRPCList.value.has(val) ? val : ''
}, { immediate: true })

watch(selected, (s) => {
  if (!s || !import.meta.client) {
    return
  }
  if (s) {
    sorobanRPC.value = s
  }
}, { immediate: true })
</script>

<template>
  <div
    class="setting-item network"
  >
    <features-toggle feature="network">
      <div
        class="d-flex justify-content-between align-items-center mb-4"
      >
        <div class="setting-item__title">
          Network RPC
        </div>
        <j-popover
          :teleport-to-body="false"
          position="bottom"
          class-name="select-network"
          menu-class="select-network__menu"
          close-popup
        >
          <div>
            <div
              v-for="network in networks"
              :key="network"
              class="select-network__item"
              :class="{ 'select-network__item--active': activeNetwork === network }"
              @click="activeNetwork = network"
            >
              {{ network }}
            </div>
          </div>
          <template #target="{ active }">
            <j-btn
              size="xs"
              variant="ghost"
            >
              {{ activeNetwork }}

              <i-app-chevron-down
                class="arrow-icon"
                :class="{ 'arrow-icon--active': active }"
              />
            </j-btn>
          </template>
        </j-popover>
      </div>
    </features-toggle>

    <div class="networks">
      <div class="network-rpc">
        Horizon URL

        <div class="network-rpc__url">
          {{ rpcStore.horizonRPCUrl }}

        </div>

        <i-app-edit-icon
          v-if="!isHorizenEdit"
          class="edit-icon"
          @click="handleShowEdit('horizon')"
        />
        <i-app-cross-icon
          v-else
          class="edit-icon"
          style="transform: scale(1.3)"
          @click="handleShowEdit('horizon')"
        />
      </div>

      <j-input
        v-show="isHorizenEdit"
        v-model="horizonRPC"
        class="rpc-input horizon-input"
        :rules="[
          val => {
            return (!val || isValidUrl(String(val))) || 'Invalid RPC url! Should start with https://'
          },
        ]"
        @keypress.enter="handleEdit('horizon', true)"
      >
        <template
          v-if="horizonRPC"
          #append
        >
          <template
            v-if="horizonRPC !== rpcStore.customHorizonRpc"
          >
            <span
              :class="{ disabled: !isValidUrl(horizonRPC) }"
              @click="handleEdit('horizon', true)"
            >Save</span>
          </template>
          <template
            v-else
          >
            <span
              @click="handleEdit('horizon', false)"
            >Reset</span>
          </template>
        </template>
      </j-input>

      <div class="network-rpc">
        Soroban RPC URL

        <div class="network-rpc__url">
          {{ rpcStore.sorobanRPCUrl }}
        </div>

        <i-app-edit-icon
          v-if="!isSorobanEdit"
          class="edit-icon"
          @click="handleShowEdit('soroban')"
        />
        <i-app-cross-icon
          v-else
          class="edit-icon"
          style="transform: scale(1.3)"
          @click="handleShowEdit('soroban')"
        />
      </div>

      <div
        v-if="isSorobanEdit"
        class="default-networks"
      >
        {{ capitalize(activeNetwork) }} Networks:
        <div
          v-for="network in stellarRPCList"
          :key="network"
          class="network-item"
        >

          <BFormRadio
            v-model="selected"
            name="some-radios"
            :value="network"
          >{{ network }}
          </BFormRadio>
        </div>
      </div>

      <j-input
        v-show="isSorobanEdit"
        v-model="sorobanRPC"
        class="rpc-input soroban-input"
        :rules="[
          val => {
            return (!val || isValidUrl(String(val))) || 'Invalid RPC url! Should start with https://'
          },
        ]"
        @keypress.enter="handleEdit('soroban', true)"
      >
        <template #label>
          Custom RPC
        </template>
        <template
          v-if="sorobanRPC"
          #append
        >
          <template
            v-if="sorobanRPC !== rpcStore.customSorobanRpc"
          >
            <span
              :class="{ disabled: !isValidUrl(sorobanRPC) }"
              @click="handleEdit('soroban', true)"
            >Save</span>
          </template>
          <template
            v-else
          >
            <span
              @click="handleEdit('soroban', false)"
            >Reset</span>
          </template>
        </template>
      </j-input>

    </div>
  </div>
</template>

<style lang="scss">
.setting-item.network {
  .setting-item__title {
    display: flex;
    justify-content: space-between;
  }

  .select-network {
    .btn {
      text-transform: capitalize;
    }

    &__menu {
      .popover-body {
        padding-left: 0;
        padding-right: 0;
      }
    }

    &__item {
      color: $text-tertiary;
      text-transform: capitalize;
      padding: $spacing-md $spacing-3xl;
      cursor: pointer;

      &:hover,
      &--active {
        color: #fff;
      }
    }

    .arrow-icon {
      width: 8px;
      color: $surface-neutral-60;
      transform: rotate(0deg);
      margin-bottom: -2px;
      transition: $transition-base;

      &--active {
        transform: rotate(180deg);
      }
    }
  }

  .networks {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .network-rpc {
    position: relative;
    font-size: 14px;
    font-style: normal;
    font-weight: 700;
    line-height: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: $text-primary;

    &__url {
      font-size: 12px;
      font-style: normal;
      font-weight: 400;
      line-height: 14px;
      color: $text-tertiary;
    }

    .edit-icon {
      position: absolute;
      top: 0;
      right: 0;
      width: 20px;
      height: 20px;
      padding: 4px;
      cursor: pointer;
      opacity: 0.6;

      &:hover {
        opacity: 1;
      }
    }
  }

  .default-networks {
    position: relative;
    font-size: 14px;
    font-style: normal;
    font-weight: 700;
    line-height: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: $text-primary;

    .network-item {
      .form-check {
        display: flex;
        align-items: center;
        flex-direction: row-reverse;
        justify-content: space-between;
        padding: 0;

        label {
          font-size: 12px;
          font-style: normal;
          font-weight: 400;
          line-height: 14px;
          color: $text-tertiary;
          cursor: pointer;
        }

        .form-check-input {
          box-shadow: none;
          cursor: pointer;

          &:checked {
            background-color: $cyan;
            border-color: $cyan;
          }
        }
      }
    }
  }

  .rpc-input {
    margin-top: -12px;

    .j-input__label {
      color: $text-primary;
    }

    .input-group {
      height: 34px;
      border-color: $navi-300;
      border-radius: 8px;
    }

    input {
      font-size: 12px;
      margin-bottom: -2px;
    }

    .j-input__append {
      color: $primary;
      background-color: $navi-400;
      font-size: 12px;
      text-transform: capitalize;
      padding: 2px 6px;
      border-radius: 4px !important;

      &:hover:not(:has(.disabled)) {
        background-color: $navi-300;
      }

      span {
        cursor: pointer;
      }

      .disabled {
        opacity: 0.6;
        pointer-events: none;
        cursor: not-allowed;
      }
    }

    .validate-label {
      bottom: -20px;
    }
  }
}
</style>
