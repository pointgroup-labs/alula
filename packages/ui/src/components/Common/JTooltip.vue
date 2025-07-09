<script setup lang="ts">
import { /* arrow, */ autoUpdate, flip, offset as Offset, shift, useFloating } from '@floating-ui/vue'
import { onClickOutside } from '@vueuse/core'
import { ref, watchEffect } from 'vue'

const {
  offset = 4,
} = defineProps<{
  offset?: number
  tooltipClass?: string
  contentClass?: string
}>()

const slots = defineSlots()

const { width } = useWindowSize()

const reference = ref(null)
const floating = ref(null)
// const floatingArrow = ref(null)
const isVisible = ref(false)

const { floatingStyles, /* middlewareData,  placement, */ update } = useFloating(reference, floating, {
  middleware: [
    // arrow({ element: floatingArrow }),
    Offset(offset),
    flip(),
    shift(),
  ],
  placement: 'top',
})

const toggleVisible = () => {
  if (width.value > 650) {
    return
  }
  isVisible.value = true
}

watchEffect(() => {
  if (isVisible.value && reference.value && floating.value) {
    autoUpdate(reference.value, floating.value, update)
  }
  if (width.value <= 650) {
    onClickOutside(reference, () => {
      isVisible.value = false
    })
  }
})

// const getArrowSide = () => {
//   if (placement.value.startsWith('top')) { return 'bottom' }
//   if (placement.value.startsWith('bottom')) { return 'top' }
//   if (placement.value.startsWith('left')) { return 'right' }
//   if (placement.value.startsWith('right')) { return 'left' }
//   return 'top'
// }
</script>

<template>
  <div
    ref="reference"
    :class="[$style.tooltip, tooltipClass]"
    @mouseenter="isVisible = true"
    @mouseleave="isVisible = false"
    @click="toggleVisible"
  >
    <slot />
  </div>

  <teleport to="body">
    <div
      v-if="isVisible"
      ref="floating"
      class="tooltip-content"
      :class="contentClass"
      :style="{
        ...floatingStyles,
        position: 'absolute',
        transition: 'opacity 0.2s ease',
        opacity: isVisible ? 1 : 0,
        zIndex: 9999,
      }"
    >
      <slot
        v-if="slots?.content"
        name="content"
      />
      <!-- <div
        v-if="isArrow"
        ref="floatingArrow"
        class="tooltip-content__arrow"
        :style="{
          position: 'absolute',
          width: '6px',
          height: '6px',
          transform: 'rotate(45deg)',
          left: middlewareData.arrow?.x != null ? `${middlewareData.arrow.x}px` : '',
          top: middlewareData.arrow?.y != null ? `${middlewareData.arrow.y}px` : '',
          [getArrowSide()]: '-4px',
        }"
      /> -->
    </div>
  </teleport>
</template>

<style module>
.tooltip {
  width: fit-content;
  cursor: help;
}
</style>

<style lang="scss">
$tooltip-bg-color: white !default;
// $tooltip-dark-bg-color: $dark-bg !default;
$tooltip-padding-y: 12px !default;
$tooltip-padding-x: 12px !default;
$tooltip-border-radius: 12px !default;
$tooltip-border-color: #ebebeb !default;
// $tooltip-dark-border-color: $neutral-900 !default;

.tooltip-content {
  background-color: $tooltip-bg-color;
  padding: $tooltip-padding-y $tooltip-padding-x;
  border-radius: $tooltip-border-radius;
  border: 1px solid $tooltip-border-color;
  color: $dark;
  font-size: 12px;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  max-width: 300px;

  // &__arrow {
  //   background-color: $tooltip-bg-color;
  // }
}

body.body--dark {
  // .tooltip-content {
  //   background-color: $tooltip-dark-bg-color;
  //   border-color: $tooltip-dark-border-color;
  //   color: #fff;
  // }
}
</style>
