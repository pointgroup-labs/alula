<script lang="ts" setup>
const dialog = defineModel({ default: false })

const route = useRoute()
const router = useRouter()

const selectedOption = ref()

const search = ref()

const marketsStore = useMarketsStore()

const options = computed(() => {
  return marketsStore.selectedMarketPools?.map((data) => {
    const asset = getFullTokenData(data.pool.token_symbol)
    return {
      label: asset.symbol,
      value: data.pool.pool_address,
      name: asset.name,
      icon: asset.icon,
      apy: {
        borrow: data.apy.borrow_bps / 100,
        supply: data.apy.supply_bps / 100,
      },
    }
  }) ?? []
})

const filteredOptions = computed(() => {
  return search.value ? options.value.filter(option => option.label.toLowerCase().includes(search.value.toLowerCase())) : options.value
})

const escHandler = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    dialog.value = false
  }
}

function cleanUp() {
  globalThis.removeEventListener('keydown', escHandler)
  search.value = ''
}

function handleSelect(option: typeof options.value[number]) {
  selectedOption.value = option
  dialog.value = false
}

watch(selectedOption, (opt) => {
  if (!opt) {
    return
  }

  router.replace({
    name: route.name as string,
    params: {
      ...route.params,
      pool: opt.value,
    },
    query: route.query,
    hash: route.hash,
  })
})

watch(dialog, async (isOpen) => {
  globalThis.removeEventListener('keydown', escHandler)

  if (isOpen) {
    globalThis.addEventListener('keydown', escHandler)

    await sleep(300)
    focusInput('.change-pool-input')
  } else {
    cleanUp()
  }
})

onUnmounted(() => {
  cleanUp()
})
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="change-pool-dialog"
  >
    <template #header>
      <j-input
        v-model="search"
        size="sm"
        placeholder="Search token in current market"
        class="change-pool-input"
      >
        <template #prepend>
          <i-app-search-icon class="search-icon" />
        </template>
        <template #append>
          <div
            class="esc"
            @click="dialog = false"
          >ESC</div>
        </template>
      </j-input>
    </template>

    <div
      v-if="filteredOptions.length > 0"
      class="pool-list"
    >
      <div
        v-for="option in filteredOptions"
        :key="option.value"
        class="pool-list__item"
        @click="() => handleSelect(option)"
      >
        <img
          :src="option.icon"
          alt="asset icon"
          class="asset-icon"
        >
        <div class="asset-data">
          <div class="asset-data__symbol">{{ option.label }}</div>
          <div class="asset-data__name">{{ option.name }}</div>
        </div>
        <div class="apy-data">
          <div :style="{ '--color': '#22d3ee' }">
            Supply APY: <span>{{ option.apy.supply }}%</span>
          </div>
          <div :style="{ '--color': '#f59e0b' }">
            Borrow APY: <span>{{ option.apy.borrow }}%</span>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else
      class="no-data"
    >
      No pools
    </div>
  </j-dialog>
</template>

<style lang="scss">
.change-pool-dialog {
  .modal-content {
    width: 400px;
  }

  .modal-header {
    padding: 8px 0;
    border-bottom: 1px solid $border-color;

    .input-group {
      border: none;
    }

    .j-input__prepend {
      min-width: 24px;
      min-height: 24px;
      width: 24px;
      height: 24px;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    input {
      font-weight: 400;
      color: $foreground;

      &::placeholder {
        color: $muted-foreground;
      }
    }

    .search-icon {
      width: 18px;
      height: 18px;
      color: $muted-foreground;
    }

    .close-icon {
      display: none;
    }

    .esc {
      font-size: 12px;
      color: $muted-foreground;
      background-color: color-mix(in oklab, #1a2236 60%, transparent);
      display: flex;
      align-items: center;
      padding: 4px 8px;
      border-radius: 8px;
      transition: all 0.2s ease;
      cursor: pointer;

      &:hover {
        background-color: color-mix(in oklab, #1a2236 90%, transparent);
        color: $foreground;
      }
    }
  }

  .pool-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px 0;

    &__item {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 6px 16px;
      cursor: pointer;

      &:hover {
        background-color: color-mix(in oklab, $new-secondary 70%, transparent);
      }

      .asset-icon {
        width: 32px;
        height: 32px;
        object-fit: contain;
        border-radius: 50%;
      }

      .asset-data {
        min-width: 40px;
        &__symbol {
          font-size: 14px;
          color: $foreground;
        }

        &__name {
          font-size: 11px;
          color: $muted-foreground;
        }
      }

      .apy-data {
        display: flex;
        align-items: flex-start;
        flex-direction: column;
        font-size: 11px;
        color: $foreground;

        > div {
          display: flex;
          align-items: center;
          justify-content: flex-end;
          gap: 8px;

          &::before {
            content: '';
            display: block;
            width: 4px;
            height: 4px;
            border-radius: 50%;
            background-color: var(--color);
          }

          span {
            color: var(--color);
            font-family: $font-JetBrainsMono;
          }
        }
      }
    }
  }

  .no-data {
    padding: 24px;
    text-align: center;
    color: $muted-foreground;
    font-size: 12px;
  }
}
</style>
