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
  border-radius: $spacing-4;
  background-color: color-mix(in oklab, $new-secondary 40%, transparent);

  .btn {
    width: fit-content;
    padding: $spacing-4 $spacing-8;
    border-radius: $spacing-4;
    color: $muted-foreground;

    &-ghost {
      color: $foreground;
    }

    &:hover {
      color: $foreground;
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
