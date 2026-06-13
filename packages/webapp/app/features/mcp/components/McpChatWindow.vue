<script lang="ts" setup>
import type { McpContext } from '../composables/mcp'
import { formatTime } from '../composables/mcp'
import { useMcpSettings } from '../composables/settings'
import McpHelpers from './McpHelpers.vue'
import McpSettings from './McpSettings.vue'

const mcp = inject('mcp')

const { settings } = useMcpSettings()

const {
  messages,
  isOpen,
  analizyng,
  isTyping,
  draft,
  canSend,
  closeChat,
  sendMessage,
} = mcp as McpContext

function handleHelperSelect(question: string) {
  draft.value = question
  nextTick(() => {
    sendMessage()
  })
}

const panelRef = ref<HTMLElement | null>(null)
const headerRef = ref<HTMLElement | null>(null)
const messagesRef = ref<HTMLElement | null>(null)

const isGlass = computed(() => settings.value.isGlass)

function scrollToBottom() {
  const el = messagesRef.value
  if (!el) {
    return
  }
  el.scrollTop = el.scrollHeight
}

// Draggable functionality - reset on each page load
const isDragged = ref(false)
const { x, y, isDragging } = useDraggable(panelRef, {
  handle: headerRef,
  initialValue: { x: 0, y: 0 },
})

// Get window size for viewport constraints
const { width: windowWidth, height: windowHeight } = useWindowSize()

// Resizable functionality - reset on each page load
const panelSize = ref({ width: 400, height: 600 })
const isResizing = ref(false)
const resizeEdge = ref<'se' | 'sw' | 'ne' | 'nw' | ''>('')

// Constrain position to viewport
function constrainToViewport() {
  if (!isDragged.value) {
    return
  }

  const maxX = windowWidth.value - panelSize.value.width
  const maxY = windowHeight.value - panelSize.value.height

  x.value = Math.max(0, Math.min(x.value, maxX))
  y.value = Math.max(0, Math.min(y.value, maxY))
}

// Mark as dragged when user starts dragging
watch(isDragging, (dragging: boolean) => {
  if (dragging && !isDragged.value) {
    // Convert current position to fixed coordinates
    const panel = panelRef.value
    if (panel) {
      const rect = panel.getBoundingClientRect()
      x.value = rect.left
      y.value = rect.top
      isDragged.value = true
    }
  }
})

// Constrain position when dragging or coordinates change
watch([x, y], () => {
  if (isDragging.value) {
    constrainToViewport()
  }
}, { flush: 'sync' })

// Constrain position when window is resized
watch([windowWidth, windowHeight], () => {
  constrainToViewport()
})

// Constrain position when panel size changes
watch(() => panelSize.value, () => {
  constrainToViewport()
}, { deep: true })

function startResize(edge: 'se' | 'sw' | 'ne' | 'nw', e: MouseEvent) {
  e.preventDefault()
  e.stopPropagation()

  // Convert to fixed positioning if not already dragged
  if (!isDragged.value) {
    const panel = panelRef.value
    if (panel) {
      const rect = panel.getBoundingClientRect()
      x.value = rect.left
      y.value = rect.top
      isDragged.value = true
    }
  }

  isResizing.value = true
  resizeEdge.value = edge

  const startX = e.clientX
  const startY = e.clientY
  const startWidth = panelSize.value.width
  const startHeight = panelSize.value.height
  const startPosX = x.value
  const startPosY = y.value

  const onMouseMove = (e: MouseEvent) => {
    if (!isResizing.value) {
      return
    }

    const deltaX = e.clientX - startX
    const deltaY = e.clientY - startY

    if (edge.includes('e')) {
      const maxWidth = windowWidth.value - startPosX
      panelSize.value.width = Math.max(300, Math.min(maxWidth, startWidth + deltaX))
    } else if (edge.includes('w')) {
      const newWidth = Math.max(300, startWidth - deltaX)
      const widthDiff = startWidth - newWidth
      const newX = startPosX + widthDiff

      // Ensure left edge doesn't go beyond viewport
      if (newX >= 0) {
        panelSize.value.width = newWidth
        x.value = newX
      }
    }

    if (edge.includes('s')) {
      const maxHeight = windowHeight.value - startPosY
      panelSize.value.height = Math.max(400, Math.min(maxHeight, startHeight + deltaY))
    } else if (edge.includes('n')) {
      const newHeight = Math.max(400, startHeight - deltaY)
      const heightDiff = startHeight - newHeight
      const newY = startPosY + heightDiff

      // Ensure top edge doesn't go beyond viewport
      if (newY >= 0) {
        panelSize.value.height = newHeight
        y.value = newY
      }
    }
  }

  const onMouseUp = () => {
    isResizing.value = false
    resizeEdge.value = ''
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}

watch(isOpen, (v: boolean) => {
  if (v) {
    // Reset dragged state when opening chat
    nextTick(() => {
      scrollToBottom()
      document.querySelector<HTMLTextAreaElement>('.mcp-chat__field')?.focus()
    })
  }
})

// auto scroll when new message typing
watch(
  () => messages.value.at(-1)?.text,
  () => nextTick(scrollToBottom),
)
</script>

<template>
  <div
    v-show="isOpen"
    ref="panelRef"
    class="mcp-chat__panel"
    :class="{
      'mcp-chat__panel--dragged': isDragged,
      'mcp-chat__panel--glass': isGlass,
    }"
    :style="isDragged ? {
      left: `${x}px`,
      top: `${y}px`,
      width: `${panelSize.width}px`,
      height: `${panelSize.height}px`,
    } : {
      width: `${panelSize.width}px`,
      height: `${panelSize.height}px`,
    }"
  >
    <!-- Resize handles -->
    <div
      class="resize-handle resize-handle-se"
      @mousedown="startResize('se', $event)"
    />
    <div
      class="resize-handle resize-handle-sw"
      @mousedown="startResize('sw', $event)"
    />
    <div
      class="resize-handle resize-handle-ne"
      @mousedown="startResize('ne', $event)"
    />
    <div
      class="resize-handle resize-handle-nw"
      @mousedown="startResize('nw', $event)"
    />

    <div
      ref="headerRef"
      class="mcp-chat__header"
    >
      <div>
        <div class="mcp-chat__title">
          Alula AI
        </div>
        <div class="mcp-chat__subtitle">
          Understanding your needs
        </div>
      </div>
      <mcp-settings />
      <i-app-cross-icon
        class="close-icon"
        @click="closeChat"
      />
    </div>

    <div
      ref="messagesRef"
      class="mcp-chat__messages"
    >
      <!-- Show helpers when no messages or at the start -->
      <mcp-helpers
        :can-send="!analizyng && !isTyping"
        @select="handleHelperSelect"
      />
      <div class="mcp-chat__message is-support">
        <div class="mcp-chat__bubble">
          Hi! I can answer any questions about Alula, markets, pools, multiplying or anything else.
        </div>
        <div class="mcp-chat__time">
          {{ formatTime(new Date()) }}
        </div>
      </div>

      <div
        v-for="m in messages"
        :key="m.id"
        class="mcp-chat__message"
        :class="`is-${m.from}`"
      >
        <div class="mcp-chat__bubble">
          <div
            v-if="m.streaming"
            class="streaming"
          >
            <div v-html="m.html" />
          </div>
          <div
            v-else
            v-html="m.html"
          />
        </div>
        <div class="mcp-chat__message__actions">
          <i-app-reload-icon
            v-if="m.from === 'user' && !analizyng"
            class="reload-icon"
            @click="handleHelperSelect(m.text)"
          />
          <div class="mcp-chat__time">
            {{ m.time }}
          </div>
        </div>
      </div>

      <div
        v-if="analizyng"
        class="mcp-chat__typing shimmer-text"
      >
        Analyzing request…
      </div>
    </div>

    <div class="mcp-chat__input">
      <BFormTextarea
        v-model="draft"
        class="mcp-chat__field"
        placeholder="Enter your question"
        rows="2"
        max-rows="6"
        no-resize
        @keydown.enter.exact.prevent="sendMessage"
      />
      <j-btn
        variant="brand"
        size="sm"
        :disabled="!canSend"
        @click="sendMessage"
      >
        Ask
      </j-btn>
    </div>
  </div>
</template>

<style scoped lang="scss">
.mcp-chat__panel--dragged {
  position: fixed !important;
  right: auto !important;
  bottom: auto !important;
}

.resize-handle {
  position: absolute;
  width: 12px;
  height: 12px;
  z-index: 10;

  &:hover {
    background: rgba(13, 110, 253, 0.3);
  }
}

.resize-handle-se {
  bottom: 0;
  right: 0;
  cursor: se-resize;
}

.resize-handle-sw {
  bottom: 0;
  left: 0;
  cursor: sw-resize;
}

.resize-handle-ne {
  top: 0;
  right: 0;
  cursor: ne-resize;
}

.resize-handle-nw {
  top: 0;
  left: 0;
  cursor: nw-resize;
}

.mcp-chat__header {
  cursor: move;
  user-select: none;
}

.streaming {
  white-space: normal;
}

.streaming :deep(.streaming-tail) {
  white-space: pre-wrap;
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
}

.shimmer-text {
  background: linear-gradient(
    90deg,
    rgba(123, 128, 138, 0.4) 0%,
    rgba(123, 128, 138, 1) 25%,
    rgba(123, 128, 138, 1) 50%,
    rgba(123, 128, 138, 0.4) 75%,
    rgba(123, 128, 138, 0.4) 100%
  );
  background-size: 200% 100%;
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: shimmer-wave 2.5s linear infinite;
}

@keyframes shimmer-wave {
  0% {
    background-position: 200% 0;
  }

  100% {
    background-position: -200% 0;
  }
}

body.body--dark .shimmer-text {
  background: linear-gradient(
    90deg,
    rgba(163, 168, 178, 0.4) 0%,
    rgba(163, 168, 178, 1) 25%,
    rgba(163, 168, 178, 1) 50%,
    rgba(163, 168, 178, 0.4) 75%,
    rgba(163, 168, 178, 0.4) 100%
  );
  background-size: 200% 100%;
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: shimmer-wave 2.5s linear infinite;
}
</style>
