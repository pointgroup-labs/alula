<script lang="ts" setup>
const {
  iconColor,
  labelColor,
  variant = 'deposit',
  isLighting = true,
} = defineProps<{
  isLighting?: boolean
  iconColor?: string
  labelColor?: string
  variant?: 'deposit' | 'borrow'
  label?: string
}>()

const slots = defineSlots()

const iconColorByVatiant = computed(() => iconColor ?? (variant === 'deposit' ? 'rgb(255, 165, 0)' : 'rgb(135, 132, 247)'))
const labelColorByVatiant = computed(() => labelColor ?? (variant === 'deposit' ? '#08b576' : 'rgb(255, 165, 0)'))
</script>

<template>
  <div :class="$style['lighting-apy']">
    <i-app-lighting-icon
      v-if="isLighting"
      :color="iconColorByVatiant"
    />
    <template v-if="slots?.tip">
      <j-tooltip>
        <span
          :style="{ color: labelColorByVatiant }"
          :class="$style['lighting-label']"
        >{{ label }}</span>
        <template #content>
          <slot name="tip" />
        </template>
      </j-tooltip>
    </template>
    <span
      v-else
      :style="{ color: labelColorByVatiant }"
      :class="$style['lighting-label']"
    >{{ label }}</span>
  </div>
</template>

<style module>
.lighting-apy {
  display: flex;
  align-items: center;
  gap: 4px;
}

.lighting-label {
  text-decoration: underline dashed;
  text-decoration-thickness: 1px;
  text-decoration-color: currentColor;
  text-underline-offset: 4px;
}
</style>
