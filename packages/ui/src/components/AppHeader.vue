<script lang="ts" setup>
import logoDark from '~/assets/img/logo-dark.svg'
import logoLight from '~/assets/img/logo-light.svg'

const router = useRouter()
const route = useRoute()
const logo = computed(() => isDark.value ? logoDark : logoLight)

const tabs = [{
  label: 'Markets',
  route: '/',
},
{
  label: 'Multiply',
  route: '/multiply',
},
{
  label: 'My Account',
  route: '/account',
}]

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
      <img
        :src="logo"
        alt="stellar logo"
        class="app-logo"
      >

      <nav class="header-nav">
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
        <faucet-menu />

        <connect-wallet />
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
</style>
