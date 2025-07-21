<script lang="ts" setup>
import logoDark from '~/assets/img/logo-dark.svg'
import logoLight from '~/assets/img/logo-light.svg'

const router = useRouter()
const route = useRoute()
const logo = computed(() => isDark.value ? logoDark : logoLight)

const tabs = ['Markets', 'My Account']

const activeTab = ref(0)

watch(activeTab, (t) => {
  if (tabs[t] === tabs[0]) {
    router.push('/')
    return
  }
  if (tabs[t] === tabs[1]) {
    router.push({ name: 'account' })
  }
})

watch(() => route.path, (p) => {
  if (p === '/') {
    activeTab.value = 0
  }
  if (p === '/account') {
    activeTab.value = 1
  }
}, { immediate: true, once: true })
</script>

<template>
  <header>
    <div class="header-wrapper container">
      <img
        :src="logo"
        alt="stellar logo"
      >

      <nav class="header-nav">
        <div
          v-for="tab in tabs"
          :key="tab"
          class="nav-link"
          :class="{ 'nav-link--active': activeTab === tabs.indexOf(tab) }"
          @click="activeTab = tabs.indexOf(tab)"
        >
          {{ tab }}
        </div>
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
