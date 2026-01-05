<script lang="ts" setup>
const search = defineModel({ default: undefined })
</script>

<template>
  <div class="market-search">
    <j-input
      v-model="search"
      placeholder="Search assets..."
      size="sm"
    >
      <template #prepend>
        <i-app-search-icon class="search-icon" />
      </template>
    </j-input>
  </div>
</template>

<style lang="scss">
.market-search {
  width: 300px;
  margin-left: auto;
  margin-bottom: 12px;

  @media (max-width: $breakpoint-xs) {
    width: 100%;
  }

  .input-group {
    gap: $spacing-8;
    border-color: $neutral-7;
  }

  .j-input__prepend {
    min-width: 16px;
    width: 16px;
    display: flex;
    align-items: center;
  }

  input::placeholder {
    opacity: 0.7;
  }

  .search-icon {
    width: 16px;
    height: 16px;
  }
}

body.body--dark .market-search {
  .input-group {
    border-color: $neutral-16;
    &:has(input:focus) {
      border-color: $neutral-7;
    }
  }
}
</style>
