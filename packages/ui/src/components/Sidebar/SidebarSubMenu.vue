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
</template>

<style lang="scss">
.sidebar-sub-menu {
  position: fixed;
  height: 100dvh;
  width: 400px;
  right: 0;
  bottom: 0;
  background: #fff;
  transition: transform 0.1s ease-in-out;
  padding: $spacing-40;
  z-index: 1;
  overflow: hidden;

  @media (max-width: $breakpoint-xs) {
    padding: $spacing-24;
    width: 100%;
  }

  &__title {
    display: flex;
    align-items: center;
    gap: $spacing-12;
    padding-bottom: $spacing-24;
    border-bottom: 1px solid $secondary;
    font-size: 24px;
    font-style: normal;
    font-weight: 500;
    line-height: 24px;
    cursor: pointer;

    i {
      display: flex;
      align-items: center;

      svg {
        width: 24px;
        height: 24px;
      }
    }
  }

  &__body {
    height: 100%;
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

// body.body--dark {
//   .sidebar-sub-menu {
//     background-color: $dark-bg;

//     &__title {
//       border-color: $neutral-900;

//       i svg {
//         filter: invert(1);
//       }
//     }

//     .languages-list__item {
//       color: $neutral-400;

//       &.active {
//         color: #fff;
//       }
//     }
//   }
// }
</style>
