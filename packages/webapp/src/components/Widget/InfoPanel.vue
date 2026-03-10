<script lang="ts" setup>
defineProps<{
  title?: string
  data: {
    label: string
    value: string | number
    slotName?: string
    className?: string
  }[]
  variant?: 'indigo' | 'success' | 'accent'
}>()
</script>

<template>
  <div
    class="dialog-info-card"
    :class="[`dialog-info-card--${variant}`]"
  >
    <div
      v-if="title"
      class="dialog-info-card__title"
    >
      {{ title }}
    </div>

    <div class="dialog-info-card__body">
      <div
        v-for="d in data"
        :key="d.label"
        class="dialog-info-card__item"
        :class="[d?.className]"
      >
        <span class="label">{{ d.label }}</span>

        <span class="value">
          <slot
            v-if="d.slotName"
            :name="d.slotName"
            :item="d"
          >
            {{ d.value }}
          </slot>

          <template v-else>
            {{ d.value }}
          </template>
        </span>
      </div>
    </div>
  </div>
</template>
