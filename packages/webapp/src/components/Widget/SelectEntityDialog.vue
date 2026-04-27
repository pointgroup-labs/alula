<script lang="ts" setup>
defineProps<{
  searchPlaceholder?: string
}>()
const dialog = defineModel({ default: false })
const search = defineModel('search', { default: '' })
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="select-entity-dialog"
  >
    <template #header>
      <j-input
        v-model="search"
        size="sm"
        :placeholder="searchPlaceholder"
        class="select-entity-input"
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

    <slot />
  </j-dialog>
</template>

<style lang="scss">
.select-entity-dialog {
  .modal-content {
    width: 500px;
  }

  .modal-header {
    padding: 8px 0;
    border-bottom: 1px solid $border-primary;

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
      color: $text-primary;

      &::placeholder {
        color: $text-tertiary;
      }
    }

    .search-icon {
      width: 18px;
      height: 18px;
      color: $text-tertiary;
    }

    .close-icon {
      display: none;
    }

    .esc {
      font-size: 12px;
      color: $text-tertiary;
      background-color: color-mix(in oklab, #1a2236 60%, transparent);
      display: flex;
      align-items: center;
      padding: 4px 8px;
      border-radius: $radius-md;
      transition: all $transition-base ease;
      cursor: pointer;

      &:hover {
        background-color: color-mix(in oklab, #1a2236 90%, transparent);
        color: $text-primary;
      }
    }
  }

  .no-data {
    padding: 24px;
    text-align: center;
    color: $text-tertiary;
    font-size: 12px;
  }
}
</style>
