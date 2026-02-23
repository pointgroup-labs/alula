<script setup lang="ts">
import { arrow, autoUpdate, flip, offset as Offset, shift, useFloating } from '@floating-ui/vue'
import { onClickOutside, useWindowSize } from '@vueuse/core'
import { ref, watchEffect } from 'vue'

const {
  offset = 4,
  // isArrow = true,
  closeDelay = 200,
} = defineProps<{
  offset?: number
  tooltipClass?: string
  contentClass?: string
  isArrow?: boolean
  closeDelay?: number
}>()

const slots = defineSlots()

const { width } = useWindowSize()

const reference = ref<HTMLElement | null>(null)
const floating = ref<HTMLElement | null>(null)
const floatingArrow = ref<HTMLElement | null>(null)

const isVisible = ref(false)
const closeTimer = ref<ReturnType<typeof setTimeout> | null>(null)

const { floatingStyles, /*  middlewareData, *//*  placement, */ update } = useFloating(reference, floating, {
  middleware: [
    arrow({ element: floatingArrow }),
    Offset(offset),
    flip(),
    shift({ padding: 8 }),
  ],
  placement: 'top',
})

const clearCloseTimer = () => {
  if (closeTimer.value) {
    clearTimeout(closeTimer.value)
    closeTimer.value = null
  }
}

const scheduleClose = () => {
  clearCloseTimer()
  closeTimer.value = setTimeout(() => {
    isVisible.value = false
  }, closeDelay)
}

const handleMouseEnter = () => {
  clearCloseTimer()
  isVisible.value = true
}

const handleMouseLeave = () => {
  if (width.value > 650) {
    scheduleClose()
  }
}

const toggleVisible = () => {
  if (width.value > 650) {
    return
  }
  isVisible.value = true
  clearCloseTimer()
}

watchEffect((onCleanup) => {
  if (isVisible.value && reference.value && floating.value) {
    const cleanup = autoUpdate(reference.value, floating.value, update)
    onCleanup(cleanup)
  }

  if (width.value <= 650 && reference.value) {
    const stop = onClickOutside(reference, () => {
      isVisible.value = false
    })
    onCleanup(stop)
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
    @mouseenter="handleMouseEnter"
    @mouseleave="handleMouseLeave"
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
      @mouseenter="clearCloseTimer"
      @mouseleave="handleMouseLeave"
    >
      <slot
        v-if="slots?.content"
        name="content"
      />

      <!-- <div
        v-if="isArrow"
        ref="floatingArrow"
        class="tooltip-content__arrow"
        :data-side="getArrowSide()"
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
$tooltip-bg-color: $surface-neutral-12;
$tooltip-dark-bg-color: $dark;
$tooltip-padding-y: 12px;
$tooltip-padding-x: 12px;
$tooltip-border-radius: 12px;
$tooltip-border-color: $surface-neutral-10;

.tooltip-content {
  background-color: $tooltip-bg-color;
  padding: $tooltip-padding-y $tooltip-padding-x;
  border-radius: $tooltip-border-radius;
  border: 1px solid $tooltip-border-color;
  color: $text-primary;
  font-size: 12px;
  font-style: normal;
  font-weight: 400;
  line-height: 16px;
  max-width: 300px;
  word-break: break-word;
  box-shadow: 0 8px 64px 0 rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(25px);

  &__arrow {
    background: $tooltip-bg-color;
    border-style: solid;
    border-color: $tooltip-border-color;

    &[data-side='top'] {
      border-width: 1px 0 0 1px;
    }

    &[data-side='bottom'] {
      border-width: 0 1px 1px 0;
    }

    &[data-side='left'] {
      border-width: 0 0 1px 1px;
    }

    &[data-side='right'] {
      border-width: 1px 1px 0 0;
    }
  }
}
</style>
