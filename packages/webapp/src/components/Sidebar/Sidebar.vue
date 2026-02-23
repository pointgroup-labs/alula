<script lang="ts" setup>
const {
  isSidebar,
  position = 'right',
} = defineProps<{
  isSidebar: boolean
  title?: string
  position?: 'left' | 'right'
  className?: string
}>()

const emit = defineEmits(['close'])

function close() {
  emit('close')
}

let body: HTMLElement | null

watch(() => isSidebar, (val) => {
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
        v-show="isSidebar"
        class="sidebar"
        :class="className"
        role="dialog"
      >
        <div
          class="sidebar-bg"
          @click="close"
        />
        <div
          v-show="isSidebar"
          id="sidebar-root"
          class="sidebar-wrapper"
          :style="{
            '--sidebar-translate': position === 'left' ? '-100%' : '100%',
            '--sidebar-x': position === 'left' ? 'initial' : '0',
          }"
        >
          <div class="sidebar-header">
            <div class="sidebar-header__title">
              {{ title }}
            </div>

            <div
              class="btn-close"
              @click="close"
            />
          </div>

          <div class="sidebar-body">
            <slot />
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
}

.sidebar-bg {
  width: 100%;
  height: 100%;
  background: rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(6.4px);
}

.sidebar-wrapper {
  width: 400px;
  padding: $spacing-32 $spacing-24;
  position: absolute;
  right: var(--sidebar-x);
  top: 0;
  bottom: 0;
  transition: 0.1s ease;
  color: $dark;
  overflow-y: auto;
  border-left: 1px solid rgba(255, 255, 255, 0.1);
  background: $surface-neutral-04;
  box-shadow: 0 8px 64px 0 rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(125px);

  &:has(.sidebar-sub-menu) {
    overflow: hidden;
  }

  @media (max-width: $breakpoint-xs) {
    padding: $spacing-24;
    width: 100vw;
  }
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  &__title {
    color: #fff;
    font-size: 32px;
    font-style: normal;
    font-weight: 700;
    line-height: 28px;
  }

  .btn-close {
    width: 24px;
    height: 24px;
    background: #fff;
    mask-size: 22px;
    opacity: 1;
  }
}
</style>
