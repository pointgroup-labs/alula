<script lang="ts" setup>
defineProps<{
  modelValue: boolean
  className?: string
  title?: string
}>()

const { width } = useWindowSize()

const dialog = defineModel({
  default: false,
})
</script>

<template>
  <b-modal
    v-model="dialog"
    :title="title"
    centered
    scrollable
    lazy
    unmount-lazy
    no-footer
    class="j-dialog"
    :class="className"
    :content-class="{ 'modal-slide-up': width <= 650 }"
  >
    <template #header>
      <slot name="header" />
      <i-app-cross-icon
        class="close-icon"
        :color="isDark ? '#fff' : '#8A8B8D'"
        @click="dialog = false"
      />
    </template>

    <slot />
  </b-modal>
</template>

<style lang="scss">
.j-dialog {
  ::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }

  .modal-content {
    padding: 0;
    border-radius: $radius-2xl;
    border: 1px solid $border-primary;
    background: transparent;
    box-shadow: 0 8px 64px 0 rgba(0, 0, 0, 0.4);
    color: #fff;
    position: relative;
    isolation: auto;

    &::before {
      content: '';
      position: absolute;
      inset: 0;
      border-radius: inherit;
      background: rgba(10, 14, 23, 0.4);
      backdrop-filter: blur(30px);
      -webkit-backdrop-filter: blur(30px);
      z-index: -1;
    }
  }

  .modal-header {
    padding: $spacing-xl $spacing-lg $spacing-xl $spacing-3xl;
    border-bottom: 1px solid $border-primary;

    &:not(:has(.dialog-balance)) {
      .close-icon {
        margin-left: auto;
      }
    }

    .close-icon {
      width: 28px;
      height: 28px;
      padding: 4px;
      margin-left: 16px;
      cursor: pointer;
    }

    .dialog-balance {
      margin-left: auto;
      font-family: $font-family-base;

      &__label {
        color: #9ca3af;
        text-align: right;
        font-size: $text-xs;
        font-style: normal;
        font-weight: 400;
        line-height: 16.5px;
      }

      &__value {
        color: #fff;
        text-align: right;
        font-size: 12px;
        font-style: normal;
        font-weight: 500;
        line-height: 18px;
      }
    }
  }
  .modal-body {
    padding: 0;
  }
  .modal-dialog {
    width: 100%;
    max-width: 90vw;
    width: fit-content;

    @media (max-width: $breakpoint-xs) {
      height: 100%;
      max-width: 100%;
      width: 100%;
      margin: 0;
      align-items: flex-end;

      .modal-content {
        max-height: 90%;
        border-radius: $radius-4xl $radius-4xl 0 0;
      }
    }
  }

  .modal-slide-up {
    transform: translateY(100%);
    opacity: 0;
    transition:
      transform 0.1s ease-out,
      opacity 0.1s ease-out;
  }

  &.show .modal-slide-up {
    transform: translateY(2px);
    opacity: 1;
  }
}

.modal-backdrop {
  background: rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(6.4px);
  opacity: 1 !important;
}

// .theme-dark{
//   .j-dialog {
//     .modal-body {
//       color: #fff;
//     }
//   }
// }
</style>
