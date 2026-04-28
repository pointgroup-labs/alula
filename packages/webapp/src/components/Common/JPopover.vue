<script lang="ts" setup>
import type { BButtonProps } from 'bootstrap-vue-next'

const {
  position = 'top',
  placement,
  closePopup,
  modelValue,
  className,
  teleportToBody = true,
  disabled = false,
  hover = false,
  noFade = true,
  hideDelay = 150,
  ...props
} = defineProps<
  {
    teleportToBody?: boolean
    position?: 'top' | 'bottom'
    placement?:
      | 'top' | 'top-start' | 'top-end'
      | 'bottom' | 'bottom-start' | 'bottom-end'
      | 'left' | 'left-start' | 'left-end'
      | 'right' | 'right-start' | 'right-end'
    menuClass?: string
    closePopup?: boolean
    label?: string
    modelValue?: boolean
    className?: string
    menuClassName?: string
    disabled?: boolean
    // Open on pointerenter / focus instead of click. BPopover binds the
    // pointer events itself when `click` is false; we only need to flip the
    // mode and provide a small `hide` buffer so the cursor can cross the gap
    // between trigger and menu without the popover snapping shut.
    hover?: boolean
    // Skip the opacity fade entirely. Used together with `hover` to make
    // navigation popovers feel instantaneous. Leaves click popovers (modals,
    // confirm prompts) on their default 150ms fade.
    noFade?: boolean
    // Override the hover-mode hide buffer in ms. Show is always 0 — we never
    // want a delay before opening on hover.
    hideDelay?: number
  } & BButtonProps>()

const emit = defineEmits(['update:modelValue'])

const slot = defineSlots()

const navRef = ref()

const show = ref(false)

const navHeight = computed(() => navRef.value?.clientHeight)
const boundary = computed(() => position === 'top' ? 'bottom' : 'top')

// Hover delays: open instantly, leave a small window before closing so the
// cursor has time to cross the offset gap between trigger and floating menu.
// Click delays stay at zero (toggle is intentional).
const effectiveDelay = computed(() => hover
  ? { show: 0, hide: hideDelay }
  : { show: 0, hide: 0 })

function handleClickInside() {
  if (closePopup) {
    show.value = false
  }
}

watch(() => modelValue, (val) => {
  show.value = val
})

watch(show, () => {
  emit('update:modelValue', show.value)
})

onMounted(() => {
  navRef.value = document.body.querySelector('#app > nav')
})
</script>

<template>
  <b-popover
    v-model="show"
    :click="!hover"
    :no-fade="noFade"
    :hover="hover"
    :close-on-hide="true"
    :delay="effectiveDelay"
    :boundary-padding="{ [boundary]: navHeight }"
    :class="menuClass"
    :placement="placement"
    :teleport-to="teleportToBody ? 'body' : undefined"
    lazy
    unmount-lazy
  >
    <div
      class="popover-wrapper"
      :class="menuClassName"
      @click="handleClickInside"
    >
      <slot />
    </div>
    <template #target>
      <div
        v-if="slot?.target"
        :class="[className, { 'popover-target--disabled': disabled }]"
        class="popover-target"
      >
        <slot
          name="target"
          :active="show"
        />
      </div>

      <j-btn
        v-else
        v-bind="props"
      >
        {{ label }}
      </j-btn>
    </template>
  </b-popover>
</template>

<style lang="scss">
.popover {
  --bs-popover-bg: transparent;
  --bs-popover-border-color: transparent;
  --bs-popover-border-width: 0;
  border: none;
}

.popover-body {
  border-radius: $radius-xl;
  border: 1px solid $border-primary;
  background: $popover-bg;
  backdrop-filter: blur(6px);
}

.popover-arrow {
  display: none !important;
}

.popover-target {
  width: fit-content;
  user-select: none;
  cursor: pointer;

  &--disabled {
    pointer-events: none;
    cursor: not-allowed;
  }
}

.popover-body {
  padding: 12px;
}
</style>
