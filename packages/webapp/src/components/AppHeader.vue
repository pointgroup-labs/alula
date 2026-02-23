<script lang="ts" setup>
import marketsTabIcon from '~/assets/img/icons/chart-square-icon.svg?raw'
import multiplyTabIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import accountTabIcon from '~/assets/img/icons/scan-barcode-icon.svg?raw'
import logoDark from '~/assets/img/logo-dark.svg'
import logoLight from '~/assets/img/logo-light.svg'
import logoMobile from '~/assets/img/logo-mobile.svg'
import { isDark } from '~/hooks/theme'

const { width } = useWindowSize()

const route = useRoute()
const logo = computed(() => width.value >= 1024 ? (isDark.value ? logoDark : logoLight) : logoMobile)

const tabs = [{
  label: 'Markets',
  route: '/',
  icon: marketsTabIcon,
},
{
  label: 'Multiply',
  route: '/multiply',
  icon: multiplyTabIcon,
},
{
  label: 'My Account',
  route: '/account',
  icon: accountTabIcon,
  shortLabel: 'Account',
}]

provide('navTabs', tabs)

const activeTab = ref()

watch(() => route.path, (p) => {
  if (p === '/' || p.includes('lend')) {
    activeTab.value = tabs[0]
    return
  }
  const tab = tabs.slice(1).find(t => p.includes(t.route))
  if (tab) {
    activeTab.value = tab
  }
}, { immediate: true, once: true })
</script>

<template>
  <header>
    <div class="header-wrapper container">
      <client-only>
        <img
          :src="logo"
          alt="stellar logo"
          class="app-logo"
        >
      </client-only>

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
      </nav>

      <div class="header-actions">
        <connect-wallet size="md" />
        <app-settings />
      </div>
    </div>
  </header>
</template>

<style lang="scss">
header {
  .header-wrapper {
    padding-top: $spacing-16;
    padding-bottom: $spacing-16;
    display: flex;
    align-items: stretch;
    gap: 56px;
  }

  .app-logo {
    width: 120px;
    height: 52px;
  }

  .header-nav {
    display: flex;
    align-items: flex-end;
    gap: $spacing-16;

    .nav-link {
      height: 42px;
      padding: $spacing-12 $spacing-16;
      border-radius: 10px;
      color: $text-secondary;
      font-family: $font-family-base;
      font-size: 16px;
      font-style: normal;
      font-weight: 500;
      line-height: 20px;
      display: flex;
      align-items: center;
      cursor: pointer;
    }
  }

  .header-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: $spacing-12;
  }
}

.theme-dark {
  header {
    .nav-link {
      &--active {
        color: $text-primary;
        background: $surface-neutral-04;
      }

      &:hover {
        background-color: rgba(255, 255, 255, 0.06);
      }
    }
  }
}
</style>
