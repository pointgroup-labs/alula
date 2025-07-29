<script lang="ts" setup>
const {
  filters,
  modelValue,
} = defineProps<{
  filters: { label: string, value: string | number }[]
  modelValue: { label: string, value: string | number }
}>()

const emit = defineEmits(['update:modelValue'])

const activeFilter = computed({
  get() {
    return modelValue
  },
  set(value) {
    emit('update:modelValue', value)
  },
})
</script>

<template>
  <j-btn-group
    v-model="activeFilter"
    :buttons="filters"
    class="chart-date-filters"
  >
    <template #default="{ label, isActive, select }">
      <j-btn
        :variant="isActive ? 'primary' : 'secondary'"
        @click="select"
      >
        {{ label }}
      </j-btn>
    </template>
  </j-btn-group>
</template>

<style lang="scss">
.chart-date-filters {
  width: fit-content;
  margin-left: auto;
  border-radius: $spacing-4;

  .btn {
    width: fit-content;
    padding: $spacing-4 $spacing-8;
    border-radius: $spacing-4;

    .btn-content {
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;
    }
  }
  .btn-primary {
    background-color: $neutral-3;
    color: $dark;
    border-color: transparent;

    &:hover,
    &.active {
      background-color: $neutral-3;
      border-color: $neutral-3;
      color: $dark;
    }
  }
}
</style>
