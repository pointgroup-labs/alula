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
      height: 52px;
      gap: $spacing-8;
      border: none;
      background-color: $surface-neutral-08;
      padding: $spacing-8 $spacing-12;
    }

    .j-input__prepend {
      min-width: 24px;
      width: 24px;
      display: flex;
      align-items: center;
    }

    input {
      color: $text-secondary;
      font-size: 20px;
      font-style: normal;
      font-weight: 500;
      line-height: 20px;
    }

    input::placeholder {
      color: rgba(255, 255, 255, 0.40);
    }

    .search-icon {
      min-width: 24px;
      width: 24px;
      min-height: 24px;
      height: 24px;
    }
  }
}
</style>
