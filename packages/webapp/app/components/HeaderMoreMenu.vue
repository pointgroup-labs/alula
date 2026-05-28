<script lang="ts" setup>
import { DOCS_URL } from '~/config'

type MoreMenuItem
  = | { kind: 'route', label: string, to: string, description?: string }
    | { kind: 'external', label: string, href: string, description?: string }

const items: MoreMenuItem[] = [
  {
    kind: 'route',
    label: 'Transparency',
    to: '/transparency',
    description: 'Live contracts, parameters, audits',
  },
   {
    kind: 'route',
    label: 'Statistics',
    to: '/statistics',
    description: 'Live market & pool data',
  },
  {
    kind: 'external',
    label: 'Documentation',
    href: DOCS_URL,
    description: 'Guides and protocol reference',
  },
]

const route = useRoute()
const isActive = computed(() => items.some(
  item => item.kind === 'route' && route.path.startsWith(item.to),
))
</script>

<template>
  <j-popover
    position="bottom"
    placement="bottom-start"
    :teleport-to-body="false"
    hover
    no-fade
    close-popup
    class="header-more"
  >
    <template #target="{ active }">
      <button
        type="button"
        class="nav-link header-more__trigger"
        :class="{ 'nav-link--active': isActive || active }"
        aria-label="More navigation options"
      >
        More
        <i-app-chevron-down class="header-more__arrow" />
      </button>
    </template>

    <div class="select-pool-menu header-more__menu">
      <template
        v-for="item in items"
        :key="item.label"
      >
        <nuxt-link
          v-if="item.kind === 'route'"
          :to="item.to"
          class="header-more__item"
        >
          <span class="header-more__item-label">{{ item.label }}</span>
          <span
            v-if="item.description"
            class="header-more__item-desc"
          >{{ item.description }}</span>
        </nuxt-link>
        <a
          v-else
          :href="item.href"
          target="_blank"
          rel="noopener noreferrer"
          class="header-more__item"
        >
          <span class="header-more__item-label">
            {{ item.label }}
            <i-app-export-icon class="header-more__item-export" />
          </span>
          <span
            v-if="item.description"
            class="header-more__item-desc"
          >{{ item.description }}</span>
        </a>
      </template>
    </div>
  </j-popover>
</template>

<style lang="scss">
.header-more {
  .popover {
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
    padding: 0 !important;
    --bs-popover-bg: transparent;
    --bs-popover-border-color: transparent;
  }

  .popover-body,
  .popover-header {
    background: transparent !important;
    border: none !important;
    padding: 0 !important;
    color: inherit;
  }

  .popover-arrow {
    display: none !important;
  }

  // The popover wraps its trigger in `.popover-target { width: fit-content }`
  // (see JPopover.vue), so the trigger pill stays content-sized and the menu
  // anchors directly under it via the `bottom-start` placement.
  display: inline-flex;

  &__trigger {
    background: none;
    border: 1px solid transparent;
    cursor: pointer;
    font: inherit;
    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  &__arrow {
    color: $text-tertiary;
    width: 10px;
    height: 10px;
  }

  &__menu {
    min-width: 240px;
    padding: 0;
    background-color: $bg-card;
    border: 1px solid $border-primary;
    border-radius: $radius-lg;
    // Items now span edge-to-edge with no inner padding. `overflow: hidden`
    // lets the menu's rounded corners clip the first/last item's hover
    // background so the rounded shape stays clean even though items are
    // square.
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 0;
    z-index: 1080;
    position: relative;
  }

  &__item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 10px 12px;
    // No per-item radius — the menu container clips the corners instead.
    border-radius: 0;
    color: $text-primary;
    text-decoration: none;
    transition: background-color 0.12s ease;

    &:hover {
      background-color: $navi-600;
    }

    &.router-link-active {
      background-color: color-mix(in oklab, $brand-700 25%, transparent);
    }
  }

  &__item-label {
    font-size: 14px;
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  &__item-export {
    width: 12px;
    height: 12px;
    color: $text-tertiary;
  }

  &__item-desc {
    font-size: 11px;
    color: $text-tertiary;
    line-height: 1.3;
  }
}
</style>
