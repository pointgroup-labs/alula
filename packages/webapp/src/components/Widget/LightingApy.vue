<script lang="ts" setup>
const {
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

const labelColorByVatiant = computed(() => labelColor ?? (variant === 'deposit' ? '#22d3ee' : '#f59e0b'))
</script>

<template>
  <div :class="$style['lighting-apy']">
    <i-app-lighting-icon
      v-if="isLighting"
      :color="labelColorByVatiant"
      :class="$style['lighting-icon']"
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

<style module lang="scss">
.lighting-apy {
  display: flex;
  align-items: center;
  gap: 4px;
}

.lighting-icon {
  width: 10px;
  margin-bottom: -2px;
}

.lighting-label {
  font-family: $font-JetBrainsMono;
  font-size: 12px;
  font-style: normal;
  font-weight: 700;
  line-height: 100%;
  letter-spacing: -0.312px;
  text-decoration-line: underline;
  text-decoration-style: dotted;
  text-decoration-skip-ink: auto;
  text-decoration-thickness: 10.5%;
  text-underline-offset: 21.5%;
  text-underline-position: from-font;
}
</style>
