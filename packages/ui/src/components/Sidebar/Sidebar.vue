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
  if (val) {
    body?.classList.add('body--no-scroll')
  } else {
    body?.classList.remove('body--no-scroll')
  }
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
      <div
        v-show="isSidebar"
        class="sidebar"
        :class="className"
      >
        <div
          class="sidebar-bg"
          @click="close"
        />
        <transition name="fade">
          <div
            v-show="isSidebar"
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
        </transition>
      </div>
    </transition>
  </teleport>
</template>

<style lang="scss">
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
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
  background-color: rgba(0, 0, 0, 0.44);
}

.sidebar-wrapper {
  width: 400px;
  padding: $spacing-40;
  background-color: #fff;
  position: absolute;
  right: var(--sidebar-x);
  top: 0;
  bottom: 0;
  transition: 0.2s ease;
  color: $dark;
  overflow-y: auto;

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
    font-size: 24px;
    font-style: normal;
    font-weight: 500;
  }

  .btn-close {
    width: 24px;
    height: 24px;
    background-color: $dark;
    mask-size: 22px;
  }
}

// body.body--dark {
//   .sidebar-wrapper {
//     background-color: $dark-bg;
//     color: #fff;

//     .btn-close {
//       background-color: #fff;
//     }
//   }
// }
</style>
