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
  width: 280px;
  margin-left: auto;
  margin-bottom: 24px;

  @media (max-width: $breakpoint-xs) {
    width: 100%;
  }

  .j-input {
    .input-group {
      height: 40px;
      gap: $spacing-12;
      background-color: $card-bg;
      padding: $spacing-6 $spacing-12;
      border: 1px solid $border-color;
    }

    .j-input__prepend {
      min-width: 20px;
      width: 20px;
      display: flex;
      align-items: center;
    }

    input {
      color: $foreground;
      font-size: $text-sm;
      font-style: normal;
      font-weight: 400;
      line-height: 16px;
    }

    input::placeholder {
      color: $muted-foreground;
      opacity: .7;
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
