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
  } & BButtonProps>()

const emit = defineEmits(['update:modelValue'])

const slot = defineSlots()

const navRef = ref()

const show = ref(false)

const navHeight = computed(() => navRef.value?.clientHeight)
const boundary = computed(() => position === 'top' ? 'bottom' : 'top')

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
    :click="true"
    :close-on-hide="true"
    :delay="{ show: 0, hide: 0 }"
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
