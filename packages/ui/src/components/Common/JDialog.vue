<script lang="ts" setup>
const {
  modelValue,
} = defineProps<{
  modelValue: boolean
  className?: string
  title?: string
}>()

const emits = defineEmits(['update:modelValue'])

const { width } = useWindowSize()

const dialog = computed({
  get() {
    return modelValue
  },
  set(val) {
    emits('update:modelValue', val)
  },
})
</script>

<template>
  <b-modal
    v-model="dialog" :title="title" centered scrollable no-footer
    class="j-dialog" :class="className" :content-class="{ 'modal-slide-up': width <= 650 }"
  >
    <template #header>
      <slot name="header" />
      <i-app-cross-icon class="close-icon" :color="isDark ? '#fff' : '#8A8B8D'" @click="dialog = false" />
    </template>

    <slot />
  </b-modal>
</template>

<style lang="scss">
.j-dialog {
  .modal-content {
    padding: $spacing-24;
    border-radius: $spacing-16;
  }

  .modal-header {
    padding: 0;
    border: none;

    .close-icon {
      margin-left: auto;
      cursor: pointer;
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
        max-height: 70%;
        border-radius: $spacing-24 $spacing-24 0 0;
        padding: $spacing-24;
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

// body.body--dark {
//   .j-dialog {
//     .modal-body {
//       color: #fff;
//     }
//   }
// }
</style>
