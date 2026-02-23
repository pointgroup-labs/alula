<script lang="ts" setup>
import chevronDown from '~/assets/img/icons/chevron-down.svg?raw'

const {
  flush = true,
} = defineProps<{
  flush?: boolean // true - disabled default border
  visible?: boolean // true - default open accordion item
  title?: string
}>()

const emit = defineEmits(['toggle'])

const slot = useSlots()
const show = ref(false)
</script>

<template>
  <b-accordion
    :flush="flush"
    class="j-accordion"
  >
    <b-accordion-item
      v-model="show"
      :visible="visible"
      @toggle="emit('toggle', $event)"
    >
      <template #title>
        <i v-html="chevronDown" />
        <slot
          v-if="slot?.title"
          name="title"
        />
        <template v-else>
          {{ title }}
        </template>
      </template>
      <slot />
    </b-accordion-item>
  </b-accordion>
</template>

<style lang="scss">
.j-accordion {
  --bs-accordion-bg: rgba(255, 255, 255, 0.04);
  color: #fff;

  .accordion-item {
    border-radius: 12px;
  }

  .accordion-header {
    margin: 0;
    border-radius: 12px;

    &:has(button[aria-expanded='true']) {
      border-radius: 12px 12px 0 0;
    }
  }

  .accordion-body {
    padding: $spacing-16;
    overflow: auto;
  }

  .accordion-button {
    cursor: pointer;
    color: $text-primary;
    font-size: 18px;
    font-style: normal;
    font-weight: 500;
    line-height: normal;
    padding: $spacing-12 $spacing-16;
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

      i {
        svg {
          transform: rotate(0);

          path {
            stroke: #fff;
          }
        }
      }
    }

    i {
      display: flex;
      align-items: center;
      width: 24px;
      height: 24px;
      padding: 6px 5px;
      margin-right: 8px;

      svg {
        width: 100%;
        height: 100%;
        transform: rotate(-90deg);
        transition: transform 0.1s ease;

        path {
          stroke: $surface-neutral-30;
        }
      }
    }
  }

  .collapsing {
    transition: 0.1s ease;
  }
}
</style>
