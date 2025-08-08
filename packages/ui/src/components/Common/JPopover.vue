<script lang="ts" setup>
import type { BButtonProps } from 'bootstrap-vue-next'

const {
  position = 'top',
  closePopup,
  modelValue,
  className,
  teleportToBody = true,
  ...props
} = defineProps<
  {
    teleportToBody?: boolean
    position?: 'top' | 'bottom'
    menuClass?: string
    closePopup?: boolean
    label?: string
    modelValue?: boolean
    className?: string
    menuClassName?: string
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
        :class="className"
        class="popover-target"
        @click="show = true"
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
.popover-arrow {
  display: none !important;
}

.popover-target {
  width: fit-content;
  cursor: pointer;
}

body.body--dark {
  .popover {
    background-color: $dark;
    border: 1px solid $neutral-18;
  }
}
</style>
