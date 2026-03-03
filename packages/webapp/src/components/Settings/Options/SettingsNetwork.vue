<script lang="ts" setup>
const rpcStore = useRpcStore()

const activeNetwork = ref()

const networks = Object.values(Network)

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
    <div class="d-flex justify-content-between align-items-center">
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

    <div class="networks">
      <div class="network-rpc">
        Horizon URL

        <div class="network-rpc__url">
          {{ rpcStore.horizonRPCUrl }}
        </div>
      </div>

      <div class="network-rpc">
        RPC URL

        <div class="network-rpc__url">
          {{ rpcStore.sorobanRPCUrl }}
        </div>
      </div>
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
      padding: $spacing-8 $spacing-24;
      cursor: pointer;

      &:hover {
        background-color: color-mix(in oklab, $card 60%, transparent);
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
    gap: $spacing-24;
    padding-top: $spacing-24;
  }

  .network-rpc {
    font-size: 14px;
    font-style: normal;
    font-weight: 700;
    line-height: 16px;
    display: flex;
    flex-direction: column;
    gap: $spacing-4;
    color: $text-primary;

    &__url {
      font-size: 12px;
      font-style: normal;
      font-weight: 400;
      line-height: 14px;
      color: $muted-foreground;
    }
  }
}
</style>
