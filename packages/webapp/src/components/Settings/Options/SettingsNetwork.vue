<script lang="ts" setup>
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

watch(activeNetwork, (n) => {
  rpcStore.setNetwork(n)
})

watch(() => rpcStore.network, (val) => {
  activeNetwork.value = val
}, { once: true, immediate: true })
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
        RPC URL

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
      color: $text-primary;
      text-transform: capitalize;
      padding: $spacing-md $spacing-3xl;
      cursor: pointer;

      &:hover {
        background-color: color-mix(in oklab, $navi-900 60%, transparent);
      }
    }

    .arrow-icon {
      width: 12px;
      color: $surface-neutral-60;
      transform: rotate(0deg);
      margin-bottom: -2px;

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

  .rpc-input {
    height: 34px;
    margin-top: -12px;

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
