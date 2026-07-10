<script lang="ts" setup>
import { useMcp } from '../composables/mcp'
import McpChatBtn from './McpChatBtn.vue'
import McpChatWindow from './McpChatWindow.vue'

const mcp = useMcp()

provide('mcp', mcp)
</script>

<template>
  <div class="mcp-chat">
    <mcp-chat-btn />

    <mcp-chat-window />
  </div>
</template>

<style lang="scss">
@import url(../assets/styles.scss);
</style>
