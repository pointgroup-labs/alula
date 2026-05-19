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
        :variant="isActive ? 'ghost' : 'outline-ghost'"
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
  border-radius: $radius-xs;
  background-color: $navi-800;

  .btn {
    height: 20px;
    width: fit-content;
    padding: $spacing-xs $spacing-md;
    border-radius: $spacing-xs;
    color: $text-tertiary;
    outline: none !important;

    &-ghost {
      color: $text-primary;
      background-color: $navi-500;
      border-radius: $radius-xs;
    }

    &:hover {
      color: $text-primary;
    }

    .btn-content {
      font-size: 11px;
      font-style: normal;
      font-weight: 500;
      line-height: 12px;
    }
  }
}
</style>
