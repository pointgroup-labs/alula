<script lang="ts" setup>
import arrowDown from '~/assets/img/icons/accordion-arrow-down.svg?raw'
import arrowUp from '~/assets/img/icons/accordion-arrow-up.svg?raw'

const {
  flush = true,
  icon = arrowDown,
  activeIcon = arrowUp,
} = defineProps<{
  flush?: boolean // true - disabled default border
  visible?: boolean // true - default open accordion item
  title?: string
  icon?: string
  activeIcon?: string
}>()

const slot = useSlots()
const show = ref(false)

const accordionIcon = computed(() => show.value ? activeIcon : icon)
</script>

<template>
  <b-accordion
    :flush="flush"
    class="j-accordion"
  >
    <b-accordion-item
      v-model="show"
      :visible="visible"
    >
      <template #title>
        <slot
          v-if="slot?.title"
          name="title"
        />
        <template v-else>
          {{ title }}
        </template>
        <i v-html="accordionIcon" />
      </template>
      <slot />
    </b-accordion-item>
  </b-accordion>
</template>

<style lang="scss">
.j-accordion {
  .accordion-item {
    border-radius: $radius-8;
  }

  .accordion-header {
    margin: 0;
    background: linear-gradient(101deg, rgb(0, 165, 255) 3.44%, rgb(0, 66, 102) 95.59%);
    border-radius: $radius-8;
  }
  .accordion-body {
    padding: $spacing-16 $spacing-16 0;
    overflow: auto;
  }

  .accordion-button {
    cursor: pointer;
    color: #fff;
    font-size: 18px;
    font-style: normal;
    font-weight: 500;
    line-height: normal;
    padding: $spacing-16;
    background-color: transparent;

    font-family: $font-family-base;

    &:focus {
      box-shadow: none;
    }

    &::after {
      display: none;
    }

    &:not(.collapsed) {
      background-color: transparent;
      box-shadow: none;
    }
    i {
      display: flex;
      align-items: center;
      margin-left: auto;

      svg {
        width: 16px;
        height: 16px;
        filter: invert(1);

        @media (max-width: $breakpoint-xs) {
          width: 20px;
          height: 20px;
        }
      }
    }
  }

  .collapsing {
    transition: 0.1s ease;
  }
}

body.body--dark {
  .j-accordion {
    border-color: #fff;

    .accordion-item {
      background-color: $neutral-18;
    }

    .accordion-button {
      color: #fff;
    }
    .accordion-body {
      color: #fff;
    }
  }
}
</style>
