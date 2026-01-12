<script lang="ts" setup>
const rpcStore = useRpcStore()

const activeNetwork = ref()

const networks = Object.values(Network)

watch(activeNetwork, (n) => {
  rpcStore.setNetwork(n)
})

watch(() => rpcStore.network, (val) => {
  activeNetwork.value = val
}, { once: true })
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
          <div class="select-network__btn">
            {{ activeNetwork }}
            <i-app-arrow-up
              class="arrow-icon"
              :class="{ 'arrow-icon--active': active }"
            />
          </div>
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
    &__btn {
      border: 1px solid $neutral-16;
      text-transform: capitalize;
      padding: $spacing-6 $spacing-12;
      border-radius: $spacing-6;
      cursor: pointer;
    }

    &__menu {
      .popover-body {
        padding: 0;
      }
    }

    &__item {
      text-transform: capitalize;
      padding: $spacing-8 $spacing-24;
      cursor: pointer;

      &:hover {
        background-color: $neutral-5;
      }
    }

    .arrow-icon {
      transform: rotate(180deg);

      &--active {
        transform: rotate(0deg);
      }
    }
  }

  .networks {
    display: flex;
    flex-direction: column;
    gap: $spacing-12;
    padding-top: $spacing-12;
  }

  .network-rpc {
    font-size: 16px;
    font-style: normal;
    font-weight: 700;
    line-height: 20px;
    display: flex;
    flex-direction: column;
    gap: $spacing-4;

    &__url {
      font-size: 15px;
      font-style: normal;
      font-weight: 400;
      line-height: 20px;
      color: $neutral-16;
    }
  }
}

body.body--dark {
  .setting-item.network {
    .select-network__item {
      color: #fff;

      &:hover {
        background-color: $neutral-18;
      }
    }

    .network-rpc__url {
      color: $neutral-12;
    }
  }
}
</style>
