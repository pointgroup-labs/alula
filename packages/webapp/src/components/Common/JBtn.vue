<script lang="ts" setup>
import type { BButtonProps } from 'bootstrap-vue-next'

const {
  loading = false,
  variant = 'primary',
  rounded = false,
  ...props } = defineProps<{
    rounded?: boolean
    loading?: boolean
    iconLeft?: boolean
    iconRight?: boolean
  } & BButtonProps>()

const slot = useSlots()
</script>

<template>
  <b-button
    v-bind="props"
    :variant="variant"
    :loading="false"
    :class="{ 'btn-rounded': rounded }"
    :pressed="undefined"
  >
    <div class="btn-content">
      <slot
        v-if="slot?.prepend"
        name="prepend"
      />
      <i-app-arrow-left
        v-if="iconLeft"
        class="btn-icon-left"
      />
      <slot />
      <slot
        v-if="slot?.append"
        name="append"
      />
      <i-app-arrow-right
        v-if="iconRight"
        class="btn-icon-right"
      />
    </div>
    <transition name="fade">
      <div
        v-if="loading"
        class="loading-btn-spinner"
      >
        <BSpinner />
      </div>
    </transition>
  </b-button>
</template>
