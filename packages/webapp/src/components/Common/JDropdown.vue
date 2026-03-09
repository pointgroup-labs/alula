<script lang="ts" setup>
import type { BDropdownProps } from 'bootstrap-vue-next'

type JDropdownProps = Omit<BDropdownProps, 'variant' | 'size' | 'noCaret'>

const {
  autoClose = 'outside',
  menuClass = '',
  className = '',
  offset = { mainAxis: 10 },
  ...props
} = defineProps<{
  autoClose?: boolean | 'outside' | 'inside'
  menuClass?: string
  className?: string
} & JDropdownProps>()

const emit = defineEmits(['showHandler'])

function dropdownHandler(actionType: 'show' | 'hide') {
  emit('showHandler', actionType)
}
</script>

<template>
  <BDropdown
    :auto-close="autoClose"
    :menu-class="[menuClass, 'dropdown-menu']"
    :class="[className]"
    :offset="offset"
    class="j-menu"
    variant="link"
    size="sm"
    no-caret
    v-bind="props"
    @show="dropdownHandler('show')"
    @hide="dropdownHandler('hide')"
  >
    <template #button-content>
      <slot />
    </template>

    <template #default>
      <slot name="menu" />
    </template>
  </BDropdown>
</template>

<style lang="scss">
.j-menu {
  .btn {
    background-color: color-mix(in oklab, $new-secondary 80%, transparent);
    color: $text-primary;
    padding: 8px 12px;

    &.show {
      border-color: $border !important;
    }

    svg {
      width: 10px;
      height: 5px;
      margin-left: 10px;
    }
  }

  .dropdown-menu {
    display: flex;
    padding: 12px 8px 8px;
    align-items: center;
    border-radius: $radius-lg;
    border: 1px solid $border-color;
    background-color: $card;
    backdrop-filter: blur(5px);
    color: #fff;
    font-size: 14px;
    font-style: normal;
    font-weight: 500;
    line-height: 20px;

    li {
      padding: 6px;
      cursor: pointer;
      border-radius: $radius-xs;

      &:hover {
        background-color: color-mix(in oklab, $text-tertiary 40%, transparent);
      }
    }

    .separator {
      margin: 6px 0;
    }
  }
}
</style>
