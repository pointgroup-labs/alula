<script lang="ts" setup>
import marketsTabIcon from '~/assets/img/icons/chart-square-icon.svg?raw'
import multiplyTabIcon from '~/assets/img/icons/percentage-square-icon.svg?raw'
import accountTabIcon from '~/assets/img/icons/scan-barcode-icon.svg?raw'
import logoDark from '~/assets/img/logo-dark.svg'
import logoLight from '~/assets/img/logo-light.svg'
import logoMobileDark from '~/assets/img/logo-mobile-dark.svg'
import logoMobileLight from '~/assets/img/logo-mobile-light.svg'
import { isDark } from '~/hooks/theme'

const { width } = useWindowSize()

const router = useRouter()
const route = useRoute()
const logo = computed(() => width.value >= 1024 ? (isDark.value ? logoDark : logoLight) : (isDark.value ? logoMobileDark : logoMobileLight))

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

watch(activeTab, (t) => {
  router.push(t?.route || '/')
})

watch(() => route.path, (p) => {
  const tabIdx = tabs.findIndex(t => t?.route === p)
  if (tabIdx !== -1) {
    activeTab.value = tabs[tabIdx]
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
        <div
          v-for="tab in tabs"
          :key="tab.label"
          class="nav-link"
          :class="{ 'nav-link--active': activeTab.route === tab.route }"
          @click="activeTab = tab"
        >
          {{ tab.label }}
        </div>
      </nav>

      <div class="header-actions">
        <faucet-menu v-if="width >= 1024" />

        <connect-wallet :size="width > 650 ? 'lg' : 'md'" />
        <app-settings />
      </div>
    </div>
  </header>
</template>

<style lang="scss">
header {
  background-color: #fff;

  .header-wrapper {
    padding-top: $spacing-16;
    padding-bottom: $spacing-16;
    display: flex;
    align-items: center;
    gap: $spacing-24;
  }

  .header-nav {
    display: flex;

    .nav-link {
      margin: $spacing-12 $spacing-16;
      padding: $spacing-8 $spacing-20;
      border-radius: $spacing-32;
      font-size: 16px;
      font-style: normal;
      font-weight: 700;
      line-height: normal;
      cursor: pointer;

      &--active {
        background-color: $neutral-13;
      }

      &:hover {
        background-color: $neutral-2;
      }
    }
  }

  .header-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: $spacing-12;
  }
}

body.body--dark {
  header {
    background-color: $dark;

    .nav-link {
      &:hover {
        background-color: $neutral-18;
      }

      &--active {
        background-color: $neutral-16;
      }
    }
  }
}
</style>
