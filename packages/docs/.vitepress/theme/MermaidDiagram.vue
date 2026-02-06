<template>
  <div v-html="svg"></div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import mermaid from 'mermaid'

const props = defineProps({
  graph: { type: String, required: true },
  id: { type: String, required: true },
  class: { type: String, required: false, default: 'mermaid' },
})

const svg = ref(null)
let mut = null

onMounted(async () => {
  mut = new MutationObserver(async () => await renderChart())
  mut.observe(document.documentElement, { attributes: true })
  await renderChart()
})

onUnmounted(() => mut?.disconnect())

const renderChart = async () => {
  const hasDarkClass = document.documentElement.classList.contains('dark')
  const config = {
    securityLevel: 'loose',
    startOnLoad: false,
    theme: hasDarkClass ? 'dark' : 'default',
  }
  mermaid.initialize(config)
  const { svg: svgCode } = await mermaid.render(props.id, decodeURIComponent(props.graph))
  const salt = Math.random().toString(36).substring(7)
  svg.value = `${svgCode} <span style="display: none">${salt}</span>`
}
</script>
