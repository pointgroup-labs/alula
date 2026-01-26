<script setup lang="ts">
import { arrow, autoUpdate, flip, offset as Offset, shift, useFloating } from '@floating-ui/vue'
import { onClickOutside } from '@vueuse/core'
import { ref, watchEffect } from 'vue'

const {
  offset = 4,
  isArrow = true,
} = defineProps<{
  offset?: number
  tooltipClass?: string
  contentClass?: string
  isArrow?: boolean
}>()

const slots = defineSlots()

const { width } = useWindowSize()

const reference = ref(null)
const floating = ref(null)
const floatingArrow = ref(null)
const isVisible = ref(false)

const { floatingStyles, middlewareData, placement, update } = useFloating(reference, floating, {
  middleware: [
    arrow({ element: floatingArrow }),
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

const getArrowSide = () => {
  if (placement.value.startsWith('top')) { return 'bottom' }
  if (placement.value.startsWith('bottom')) { return 'top' }
  if (placement.value.startsWith('left')) { return 'right' }
  if (placement.value.startsWith('right')) { return 'left' }
  return 'top'
}
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
      <div
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
      />
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
$tooltip-bg-color: $neutral-3;
$tooltip-dark-bg-color: $dark;
$tooltip-padding-y: 12px;
$tooltip-padding-x: 12px;
$tooltip-border-radius: 12px;
$tooltip-border-color: $neutral-7;
$tooltip-dark-border-color: $neutral-18;

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
  word-break: break-word;

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

.theme-dark {
  .tooltip-content {
    background-color: $tooltip-dark-bg-color;
    border-color: $tooltip-dark-border-color;
    color: #fff;

    &__arrow {
      background-color: $tooltip-dark-bg-color;
      border-color: $tooltip-dark-border-color;
    }
  }
}
</style>
