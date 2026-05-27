<script lang="ts" setup>
import marketsTabIcon from '~/assets/img/icons/chart-square-icon.svg?raw'
import multiplyTabIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import accountTabIcon from '~/assets/img/icons/scan-barcode-icon.svg?raw'
import statisticsTabIcon from '~/assets/img/icons/statistics-icon.svg?raw'
import { useFeatureToggle } from '~/features/features-toggle'

const { width } = useWindowSize()

const { isEnabled } = useFeatureToggle()

const route = useRoute()

const tabs = computed(() => {
  const navTabs = [{
    label: 'Markets',
    route: '/',
    icon: marketsTabIcon,
  },
  {
    label: 'Portfolio',
    route: '/portfolio',
    icon: accountTabIcon,
    shortLabel: 'Portfolio',
  },
  {
    label: 'Statistics',
    route: '/statistics',
    icon: statisticsTabIcon,
    shortLabel: 'Statistics',
  }]

  if (isEnabled('multiply')) {
    navTabs.splice(1, 0, {
      label: 'Multiply',
      route: '/multiply',
      icon: multiplyTabIcon,
    })
  }

  if (isEnabled('swap')) {
    navTabs.splice(2, 0, {
      label: 'Swap',
      route: '/swap',
      icon: multiplyTabIcon,
    })
  }
  return navTabs
})

provide('navTabs', tabs)

const activeTab = ref()

watch(() => route.path, (p) => {
  if (p === '/' || p.includes('lend')) {
    activeTab.value = tabs.value[0]
    return
  }
  const tab = tabs.value.slice(1).find(t => p.includes(t.route))
  activeTab.value = tab
}, { immediate: true })
</script>

<template>
  <header>
    <div class="header-wrapper container">
      <nuxt-link
        to="/"
      >
        <img
          src="/logo.svg"
          alt="stellar logo"
          class="app-logo"
        >
      </nuxt-link>

      <nav
        v-if="width >= 1024"
        class="header-nav"
      >
        <nuxt-link
          v-for="tab in tabs"
          :key="tab.label"
          :to="tab.route"
          class="nav-link"
          :class="{ 'nav-link--active': activeTab?.route === tab?.route }"
          @click="activeTab = tab"
        >
          {{ tab.label }}
        </nuxt-link>
        <header-more-menu />
      </nav>

      <div class="header-actions">
        <connect-wallet />
        <app-settings />
      </div>
    </div>
  </header>
</template>

<style lang="scss">
header {
  border-bottom: 1px solid $border-primary;
  // Lift the header above page content (was 1) so its inline popovers — like
  // the More-menu dropdown — can stack over sticky panels, charts, and other
  // page elements that create their own stacking contexts. 100 is high enough
  // to win against regular content but stays well below modal layers.
  z-index: 100;
  position: relative;

  .header-wrapper {
    padding-top: $spacing-2xl;
    padding-bottom: $spacing-2xl;
    display: flex;
    align-items: stretch;
    gap: 64px;
  }

  .app-logo {
    width: 80px;
    height: 42px;
  }

  .header-nav {
    display: flex;
    align-items: flex-end;
    gap: 16px;

    .nav-link {
      height: 32px;
      padding: $spacing-md $spacing-2xl;
      border-radius: $radius-sm;
      color: $text-tertiary;
      font-size: 14px;
      font-style: normal;
      font-weight: 500;
      line-height: 16px;
      display: flex;
      align-items: center;
      cursor: pointer;

      &:hover {
        color: $text-secondary;
        background-color: $navi-450;
      }

      &--active {
        background-color: $navi-600;
        color: $text-primary;
      }
    }
  }

  .header-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 12px;
  }
}
</style>
