<script lang="ts" setup>
import { parseMarkdownToHtml } from '~/composables/markdown'
import { TERMS_OF_SERVICE_DOCS_LINK } from '~/config'

const data = await $fetch<string>(TERMS_OF_SERVICE_DOCS_LINK, {
  responseType: 'text',
})

const termsHtml = computed(() => parseMarkdownToHtml(data))
</script>

<template>
  <div
    class="terms-content"
    v-html="termsHtml"
  />
</template>

<style lang="scss">
@import '../assets/legal.scss';
</style>
