<script lang="ts" setup>
import type { ToastAction, ToastProps } from '../toast'
import attentionIcon from '../assets/img/attention-circle.svg?raw'
import crossIcon from '../assets/img/cross-icon.svg?raw'
import dangerIcon from '../assets/img/danger-icon.svg?raw'
import checkIcon from '../assets/img/toast-check-icon.svg?raw'

const {
  noProgress,
  modelValue,
} = defineProps<{
  title?: string
  body?: string
  actions?: ToastProps['actions']
  noProgress?: boolean
  toastClass: string
  variant: string
  modelValue: number
}>()

const emit = defineEmits(['update:modelValue'])

const iconMap: Record<string, string> = {
  success: checkIcon,
  info: attentionIcon,
  warning: attentionIcon,
  danger: dangerIcon,
}

let timer: ReturnType<typeof setInterval> | undefined
let timeout: ReturnType<typeof setTimeout> | undefined

function resetTimers() {
  clearTimeout(timeout)
  clearInterval(timer)
}

function close() {
  resetTimers()
  emit('update:modelValue', false)
}

function onActionClick(action: ToastAction, event: MouseEvent) {
  if (action?.onClick) {
    event.preventDefault()
    action.onClick()
    close()
  }
}

// progress
const interval = 50
const progress = ref(0)

watch(() => noProgress, (value) => {
  if (value || modelValue === 0) {
    return
  }
  const step = (100 / modelValue) * interval
  progress.value = 100
  timer = setInterval(() => {
    progress.value = Math.max(0, progress.value - step)
    if (progress.value <= 0) {
      clearInterval(timer)
    }
  }, interval)
}, { immediate: true })

// close timer
watch(() => modelValue, (value) => {
  if (value > 0 && Number.isInteger(value)) {
    timeout = setTimeout(() => {
      close()
    }, value)
  }
}, { immediate: true })
</script>

<template>
  <div class="toast-wrapper" :class="toastClass">
    <div class="toast-content">
      <div class="toast-icon" v-html="iconMap[variant]" />
      <div class="toast-message">
        <div v-if="title" class="toast-title">
          {{ title }}
          <div class="alert-close" @click="close" v-html="crossIcon" />
        </div>

        {{ body }}
      </div>
    </div>

    <div v-if="actions?.length" class="toast-actions">
      <a
        v-for="(action, i) in actions"
        :key="i"
        class="toast-action"
        :class="action?.class"
        :href="action?.href || '#'"
        :target="action?.target || '_self'"
        :style="{ color: action?.color || 'inherit' }"
        @click="onActionClick(action, $event)"
      >
        {{ action.label }}
      </a>
    </div>
    <BProgress v-if="!noProgress" :value="progress" />
  </div>
</template>
