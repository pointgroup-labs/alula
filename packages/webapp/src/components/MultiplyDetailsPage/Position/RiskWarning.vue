<script lang="ts" setup>
const {
  buffer,
} = defineProps<{
  buffer: number
}>()

const isWarning = computed(() => buffer < 50)
const isDanger = computed(() => buffer < 20)

const title = computed(() => {
  switch (true) {
    case buffer < 10: return 'Critical'
    case buffer < 20: return 'High Risk'
    default: return 'Elevated Risk'
  }
})
const warnintText = computed(() => {
  if (!isWarning.value) {
    return ''
  }
  return isDanger.value
    ? 'Near liquidation. Add collateral immediately.'
    : 'Position at risk. Consider adding collateral.'
})
</script>

<template>
  <warning-block
    v-if="isWarning"
    :title="title"
    :text="warnintText"
    is-warning
    class="risk-warning"
    :class="`risk-warning--${isDanger ? 'danger' : 'warning'}`"
  />
</template>

<style lang="scss">
.warning-block.risk-warning {
  padding: $spacing-4 $spacing-12 $spacing-4 $spacing-8;
  align-items: center;

  .warning-text--warning {
    color: $neutral-12;
  }

  &--warning {
    .warning-text--warning {
      span {
        color: $warning;
      }
    }
  }

  &--danger {
    .warning-text--warning {
      span {
        color: $danger;
      }
    }

    .warning-icon {
      path {
        fill: $danger;
      }
    }
  }
}
</style>
