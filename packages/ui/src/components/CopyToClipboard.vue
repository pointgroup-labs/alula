<script lang="ts" setup>
import copyIcon from '~/assets/img/icons/copy.svg?raw'

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

const Toast = useToast()

function copy() {
  navigator.clipboard.writeText(text)
  Toast.create({
    body: entity ? `Copied ${entity}` : 'Copied to clipboard',
    variant: 'info',
  })
}
</script>

<template>
  <j-tooltip>
    <i
      :style="{ color }"
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
