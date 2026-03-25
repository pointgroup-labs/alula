<script lang="ts" setup>
const search = ref()

const router = useRouter()
const route = useRoute()

watchDebounced(search, (val) => {
  const query = val ? { search: val } : undefined
  router.replace({
    ...route,
    query,
  })
}, { debounce: 300 })

onMounted(() => {
  if (route.query?.search) {
    search.value = route.query.search
  }
})
</script>

<template>
  <div class="market-search">
    <j-input
      v-model="search"
      placeholder="Search"
      size="md"
    >
      <template #prepend>
        <i-app-search-icon class="search-icon" />
      </template>
    </j-input>
  </div>
</template>

<style lang="scss">
.market-search {
  width: 280px;

  @media (max-width: $breakpoint-xs) {
    width: 100%;
  }

  .j-input {
    .input-group {
      height: 40px;
      gap: 12px;
      background-color: $bg-card;
      padding: $spacing-sm $spacing-lg;
      border: 1px solid $border-primary;
    }

    .j-input__prepend {
      min-width: 20px;
      width: 20px;
      display: flex;
      align-items: center;
    }

    input {
      color: $text-primary;
      font-size: $text-sm;
      font-style: normal;
      font-weight: 400;
      line-height: 16px;
    }

    input::placeholder {
      color: $text-tertiary;
      opacity: 0.7;
    }

    .search-icon {
      min-width: 17px;
      width: 17px;
      min-height: 17px;
      height: 17px;
      color: #8a9bb8;
    }
  }
}
</style>
