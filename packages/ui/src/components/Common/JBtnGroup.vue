<script setup lang="ts">
import type { BButtonProps } from 'bootstrap-vue-next'

type ButtonOpt = string | { label: string, value: any }

const {
  modelValue,
  buttons,
  ...props } = defineProps<{
    buttons: Array<ButtonOpt>
    modelValue: ButtonOpt | string
  } & BButtonProps>()

const emit = defineEmits(['update:modelValue'])
const slots = useSlots() as { default?: () => VNode[] }

const hasDefaultSlot = computed(() => !!slots?.default)

function select(label: ButtonOpt) {
  emit('update:modelValue', label)
}
</script>

<template>
  <div class="j-btn-group">
    <template v-if="hasDefaultSlot">
      <slot
        v-for="(btn) in buttons" :key="typeof btn === 'string' ? btn : btn.value"
        :label="typeof btn === 'string' ? btn : btn.label"
        :is-active="(typeof btn === 'string' ? btn : btn.label) === (typeof modelValue === 'string' ? modelValue : modelValue.label)"
        :select="() => select(btn)"
      />
    </template>

    <template v-else>
      <j-btn
        v-for="btn in buttons" v-bind="props" :key="typeof btn === 'string' ? btn : btn.value"
        :variant="(typeof btn === 'string' ? btn : btn.label) === (typeof modelValue === 'string' ? modelValue : modelValue.label) ? 'primary' : 'secondary'"
        @click="select(btn)"
      >
        {{ btn }}
      </j-btn>
    </template>
  </div>
</template>

<style lang="scss">
.j-btn-group {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
  background-color: $neutral-2;
  padding: 4px;
  border-radius: $spacing-12;

  .btn {
    width: 100%;

    &-secondary {
      background-color: $neutral-2;
      border-color: transparent;

      &.active {
        background-color: $neutral-2;
        border-color: transparent;
      }

      &:hover {
        background-color: $neutral-2;
        border-color: $neutral-3;
      }
    }

    &-content {
      justify-content: center;
    }
  }
}

// body.body--dark {
//   .j-btn-group {
//     background-color: $dark;

//     .btn-secondary {
//       background-color: $dark;
//       color: #fff;

//       &:hover {
//         border-color: $neutral-600;
//       }
//     }
//   }
// }
</style>
