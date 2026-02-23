<script lang="ts" setup>
import arrowLeft from '~/assets/img/icons/arrow-left.svg?raw'

const {
  isSubMenu = false,
  title = 'back',
  position = 'right',
} = defineProps<{
  isSubMenu?: boolean
  title?: string
  position?: 'left' | 'right'
}>()

const emit = defineEmits(['close'])

function close() {
  emit('close')
}
</script>

<template>
  <Teleport to="body">
    <transition name="slide">
      <div
        v-if="isSubMenu"
        class="sidebar-sub-menu"
        :style="{
          '--sidebar-translate': position === 'left' ? '-100%' : '100%',
          '--sidebar-x': position === 'left' ? 'initial' : '0',
        }"
      >
        <div
          class="sidebar-sub-menu__title"
          @click="close"
        >
          <i v-html="arrowLeft" />
          {{ title }}
        </div>

        <div class="sidebar-sub-menu__body">
          <slot />
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<style lang="scss">
.sidebar-sub-menu {
  position: fixed;
  height: 100dvh;
  width: 400px;
  right: 0;
  bottom: 0;
  transition: transform 0.1s ease-in-out;
  padding: 32px 24px;
  z-index: 101;
  overflow: hidden;
  border-left: 1px solid $surface-neutral-10;
  background: $surface-neutral-04;
  box-shadow: 0 8px 64px 0 rgba(0, 0, 0, 0.4);
  backdrop-filter: blur(125px);

  @media (max-width: $breakpoint-xs) {
    padding: $spacing-24;
    width: 100%;
  }

  &__title {
    height: 36px;
    display: flex;
    align-items: center;
    gap: $spacing-12;
    font-size: 32px;
    font-style: normal;
    font-weight: 700;
    line-height: 28px;
    cursor: pointer;

    i {
      display: flex;
      align-items: center;

      svg {
        width: 22px;
        height: 22px;

        path {
          stroke: $text-primary;
        }
      }
    }
  }

  &__body {
    height: 100%;
    padding-top: 24px;
    overflow: auto;
  }
}

.slide-enter-from {
  transform: translateX(var(--sidebar-translate));
}

.slide-enter-to {
  transform: translateX(0);
}

.slide-leave-from {
  transform: translateX(0);
}

.slide-leave-to {
  transform: translateX(var(--sidebar-translate));
}
</style>
