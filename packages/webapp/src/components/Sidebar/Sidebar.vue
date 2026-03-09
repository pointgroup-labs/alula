<script lang="ts" setup>
import arrowLeft from '~/assets/img/icons/arrow-left.svg?raw'

type PanelView = {
  title: string
  render: () => any
}

const {
  isSidebar,
  title,
  position = 'right',
} = defineProps<{
  isSidebar: boolean
  title?: string
  position?: 'left' | 'right'
  className?: string
}>()

const emit = defineEmits(['close'])

// ─── Navigation stack ────────────────────────────────────────────────────────
const views = ref<PanelView[]>([])
const slideDir = ref<1 | -1>(1)

provide('sidebarPush', (view: PanelView) => {
  slideDir.value = 1
  views.value.push(view)
})

provide('sidebarPop', () => {
  slideDir.value = -1
  views.value.pop()
})

const activeView = computed(() => views.value.at(-1) ?? null)
const headerTitle = computed(() => activeView.value?.title ?? title)

function back() {
  slideDir.value = -1
  views.value.pop()
}

function close() {
  views.value = []
  emit('close')
}

let body: HTMLElement | null

watch(() => isSidebar, (val) => {
  if (!val) { views.value = [] }
  body?.classList.toggle('body--no-scroll', val)
})

onMounted(() => {
  nextTick(() => {
    body = document.querySelector('body')
  })
})
</script>

<template>
  <teleport to="body">
    <transition name="fade">
      <aside
        v-if="isSidebar"
        class="sidebar"
        :class="className"
        role="dialog"
      >
        <div
          class="sidebar-bg"
          @click="close"
        />
        <div
          v-if="isSidebar"
          id="sidebar-root"
          class="sidebar-wrapper"
          :style="{
            '--sidebar-translate': position === 'left' ? '-100%' : '100%',
            '--sidebar-x': position === 'left' ? 'initial' : '0',
          }"
        >
          <div class="sidebar-header">
            <div
              class="sidebar-header__title"
              :class="{ 'sidebar-header__title--back': activeView }"
              @click="activeView ? back() : undefined"
            >
              <i
                v-if="activeView"
                class="sidebar-header__back-icon"
                v-html="arrowLeft"
              />
              {{ headerTitle }}
            </div>

            <div
              class="btn-close"
              @click="close"
            />
          </div>

          <div
            class="sidebar-body"
            :style="{ '--slide-dir': slideDir }"
          >
            <transition
              name="slide-panel"
              mode="out-in"
            >
              <div
                v-if="activeView"
                :key="views.length"
                class="sidebar-panel-view"
              >
                <component :is="activeView.render" />
              </div>
              <div
                v-else
                key="sidebar-root-view"
                class="sidebar-panel-view"
              >
                <slot />
              </div>
            </transition>
          </div>
        </div>
      </aside>
    </transition>
  </teleport>
</template>

<style lang="scss">
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.1s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;

  .sidebar-wrapper {
    transform: translateX(var(--sidebar-translate));
  }
}

.sidebar {
  position: fixed;
  top: 0;
  right: 0;
  left: 0;
  bottom: 0;
  z-index: 100;
  contain: layout paint style;
}

.sidebar-bg {
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(6px);
}

.sidebar-wrapper {
  width: 400px;
  padding: $spacing-4xl $spacing-3xl;
  position: absolute;
  right: var(--sidebar-x);
  top: 0;
  bottom: 0;
  transition: 0.1s ease;
  color: $dark;
  overflow-y: auto;
  border-left: 1px solid $border-primary;
  background: rgba(10, 14, 23, 0.3);
  backdrop-filter: blur(20px);
  transform: translateZ(0);
  will-change: transform;

  @media (max-width: $breakpoint-xs) {
    padding: $spacing-3xl;
    width: 100vw;
    box-shadow: none;
  }
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  &__title {
    color: #fff;
    font-size: 22px;
    font-style: normal;
    font-weight: 700;
    line-height: 28px;

    &--back {
      display: flex;
      align-items: center;
      gap: 12px;
      cursor: pointer;
      user-select: none;
    }
  }

  &__back-icon {
    display: flex;
    align-items: center;

    svg {
      width: 16px;
      height: 16px;

      path {
        stroke: #fff;
      }
    }
  }

  .btn-close {
    width: 16px;
    height: 16px;
    background: #fff;
    mask-size: 16px;
    opacity: 1;
    flex-shrink: 0;
  }
}

.sidebar-panel-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.slide-panel-enter-active,
.slide-panel-leave-active {
  transition:
    transform 0.18s ease,
    opacity 0.18s ease;
}

.slide-panel-enter-from {
  transform: translateX(calc(32px * var(--slide-dir)));
  opacity: 0;
}

.slide-panel-leave-to {
  transform: translateX(calc(-32px * var(--slide-dir)));
  opacity: 0;
}
</style>
