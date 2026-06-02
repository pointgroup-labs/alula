<script lang="ts" setup>
import type { ToastProps } from '../toast'
import crossIcon from '../assets/img/cross-icon.svg?raw'
import errorAlertIcon from '../assets/img/error-circle.svg?raw'
import exportIcon from '../assets/img/export-icon.svg?raw'
import successAlertIcon from '../assets/img/success-circle.svg?raw'

const {
  modelValue,
} = defineProps<{
  title?: string
  body?: string
  alertProps?: ToastProps['alertProps']
  actions?: ToastProps['actions']
  toastClass?: string
  modelValue: number
}>()

const emit = defineEmits(['update:modelValue'])

const iconAlertMap: Record<string, string> = {
  success: successAlertIcon,
  error: errorAlertIcon,
}

let timeout: ReturnType<typeof setTimeout> | undefined

function close() {
  clearTimeout(timeout)
  emit('update:modelValue', false)
}

watch(() => modelValue, (value) => {
  if (value > 0 && Number.isInteger(value)) {
    timeout = setTimeout(() => {
      close()
    }, value)
  }
}, { immediate: true })
</script>

<template>
  <div :class="`alert-${alertProps?.variant} ${toastClass}`">
    <div class="alert-header">
      <div
        class="toast-icon"
        v-html="iconAlertMap[alertProps?.variant!]"
      />
      <span :data-name="`alert-title--${alertProps?.variant}`">{{ title }}</span>
      <div
        class="alert-close"
        @click="close"
        v-html="crossIcon"
      />
    </div>

    <div class="alert-content">
      {{ body }}

      <div class="alert-content__actions">
        <div
          class="btn"
          variant="dark"
          pill
          @click="close"
        >
          OK
        </div>

        <a
          v-if="actions?.length"
          :href="actions[0]?.href"
          target="_blank"
          rel="noopener noreferrer nofollow"
          @click="close"
        >
          {{ actions[0]?.label }}
          <div
            class="export-icon"
            v-html="exportIcon"
          />
        </a>
      </div>
    </div>
  </div>
</template>
