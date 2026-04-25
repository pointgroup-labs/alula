<script lang="ts" setup>
import { DOCS_URL } from '~/config'

// Submenu items live next to the trigger so growing the menu (audits page,
// bug-bounty link, etc.) is a one-line append. `kind` discriminates how each
// item is rendered: internal routes use `<nuxt-link>` so the SPA router takes
// over; external links open in a new tab.
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
    kind: 'external',
    label: 'Documentation',
    href: DOCS_URL,
    description: 'Guides and protocol reference',
  },
]

// Keep active-state detection inside the menu so the parent (`AppHeader`) does
// not need to know which routes belong to "More". Anything that matches a
// `route`-kind item lights the trigger up the same way the regular nav links
// do via their `--active` modifier.
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
        <i-app-chevron-down
          class="header-more__arrow"
          :class="{ 'header-more__arrow--open': active }"
        />
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
    transition: transform $transition-base ease;
    width: 12px;
    height: 12px;

    &--open {
      transform: rotate(180deg);
    }
  }

  &__menu {
    min-width: 240px;
    padding: 6px;
    background-color: $bg-card;
    border: 1px solid $border-primary;
    border-radius: $radius-lg;
    display: flex;
    flex-direction: column;
    gap: 2px;
    // Belt-and-suspenders: the header itself now sits at z-index 100, but the
    // popover root (`.popover` from bootstrap-vue-next) is `position: absolute`
    // inside the trigger's stacking context. A high local z-index ensures the
    // menu paints over neighboring nav links and any siblings that gain a
    // local context (e.g. an animated chart further down the row).
    z-index: 1080;
    position: relative;
  }

  &__item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    border-radius: $radius-md;
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
