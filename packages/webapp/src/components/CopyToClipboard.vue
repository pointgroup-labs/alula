<script lang="ts" setup>
import copyIcon from '~/assets/img/icons/copy.svg?raw'
import { isDark } from '~/hooks/theme'

const {
  color = '#878787',
  text = '',
  entity = '',
}
  = defineProps<{
    color?: string
    text?: string
    entity?: string
  }>()

const toast = useToast()

const iconColor = computed(() => isDark.value ? '#8a8b8d' : color)

function copy() {
  navigator.clipboard.writeText(text)
  toast.create({
    body: entity ? `Copied ${entity}` : 'Copied to clipboard',
    variant: 'info',
  })
}
</script>

<template>
  <j-tooltip>
    <i
      :style="{ color: iconColor }"
      class="copy-icon"
      @click="copy"
      v-html="copyIcon"
    />
    <template #content>
      Copy {{ entity || 'to clipboard' }}
    </template>
  </j-tooltip>
</template>

<style scoped>
.copy-icon {
  cursor: pointer;
  display: flex;
  align-items: center;
}
</style>
