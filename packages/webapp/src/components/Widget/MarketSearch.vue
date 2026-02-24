<script lang="ts" setup>
const search = defineModel({ default: undefined })
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
  width: 326px;
  margin-left: auto;
  margin-bottom: 24px;

  @media (max-width: $breakpoint-xs) {
    width: 100%;
  }

  .j-input {
    .input-group {
      height: 42px;
      gap: $spacing-8;
      border: none;
      background-color: $surface-neutral-08;
      padding: $spacing-8 $spacing-12;
    }

    .j-input__prepend {
      min-width: 20px;
      width: 20px;
      display: flex;
      align-items: center;
    }

    input {
      color: $text-secondary;
      font-size: 16px;
      font-style: normal;
      font-weight: 500;
      line-height: 16px;
    }

    input::placeholder {
      color: rgba(255, 255, 255, 0.40);
    }

    .search-icon {
      min-width: 20px;
      width: 20px;
      min-height: 20px;
      height: 20px;
    }
  }
}
</style>
