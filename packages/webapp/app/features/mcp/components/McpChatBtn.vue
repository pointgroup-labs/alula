<script lang="ts" setup>
import type { McpContext } from '../composables'
import { useFeatureToggle } from '~/features/features-toggle'
import mcpLogo from '/favicon.svg'

const mcp = inject('mcp') as McpContext

const { isEnabled } = useFeatureToggle()

const hasUnreadMessage = computed(() => mcp.hasUnreadMessage.value)
const analizyng = computed(() => mcp.analizyng.value)

const showBtnText = ref(false)

// Launch animation on mount
onMounted(() => {
  setTimeout(() => {
    showBtnText.value = true
  }, 800)
})
</script>

<template>
  <div
    v-if="isEnabled('mcp')"
    class="mcp-chat__launcher"
  >
    <j-btn
      class="mcp-btn"
      variant="primary"
      size="sm"
      pill
      :class="{ 'mcp-btn--expanded': showBtnText }"
      @click="mcp.openChat"
    >
      <span class="mcp-btn-logo-wrap">
        <img
          :src="mcpLogo"
          alt="mcp logo"
          class="mcp-btn-logo"
        >
        <span
          v-if="hasUnreadMessage || analizyng"
          class="mcp-btn-unread-dot"
          :class="`mcp-btn-unread-dot--${hasUnreadMessage ? 'unread' : 'typing'}`"
        />
      </span>
      <span class="mcp-btn-text">
        Ask me
      </span>
    </j-btn>
  </div>
</template>

<style lang="scss">
.mcp-btn-logo-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.mcp-btn-unread-dot {
  --indicator-color: #22c55e;
  --indicator-shadow-color: rgba(34, 197, 94, 0.7);
  --indicator-glow-color: rgba(34, 197, 94, 0.45);

  position: absolute;
  top: -6px;
  right: 22px;

  width: 6px;
  height: 6px;
  border-radius: 50%;

  background: var(--indicator-color);

  box-shadow:
    0 0 0 0 var(--indicator-shadow-color),
    0 0 10px var(--indicator-glow-color);

  animation: mcp-dot-pulse 1.8s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  will-change: transform, box-shadow, opacity;

  &--unread {
    --indicator-color: #22c55e;
    --indicator-shadow-color: rgba(34, 197, 94, 0.7);
    --indicator-glow-color: rgba(34, 197, 94, 0.45);
  }

  &--typing {
    --indicator-color: #60a5fa;
    --indicator-shadow-color: rgba(96, 165, 250, 0.7);
    --indicator-glow-color: rgba(96, 165, 250, 0.45);

    animation-duration: 1.1s;
  }
}

@keyframes mcp-dot-pulse {
  0% {
    transform: scale(1);
    box-shadow:
      0 0 0 0 var(--indicator-shadow-color),
      0 0 8px var(--indicator-glow-color);
  }

  70% {
    transform: scale(1.12);
    box-shadow:
      0 0 0 8px rgba(0, 0, 0, 0),
      0 0 14px var(--indicator-glow-color);
  }

  100% {
    transform: scale(1);
    box-shadow:
      0 0 0 0 rgba(0, 0, 0, 0),
      0 0 8px var(--indicator-glow-color);
  }
}
</style>
