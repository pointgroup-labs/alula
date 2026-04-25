<script lang="ts" setup>
import { useFeatureToggle } from '~/features/features-toggle'

const { isSidebar } = defineProps<{
  isSidebar: boolean
}>()

const emit = defineEmits(['close'])

const { isProd } = useFeatureToggle()

const { width } = useWindowSize()

const isShowMobileElements = computed(() => width.value < 1024)

function close() {
  emit('close')
}
</script>

<template>
  <sidebar
    :is-sidebar="isSidebar"
    :title="$t('common.settings')"
    class-name="settings-sidebar"
    @close="close"
  >
    <!-- Hero: connection state is the primary surface of this drawer; it gets
         card treatment so it reads as "your account" rather than just another
         row in the list. -->
    <section
      class="settings-sidebar__hero"
      aria-label="Account"
    >
      <settings-connect />
    </section>

    <section
      v-if="isShowMobileElements"
      class="settings-section"
      aria-labelledby="settings-section-nav"
    >
      <h3
        id="settings-section-nav"
        class="settings-section__label"
      >
        Navigation
      </h3>
      <settings-navigation @close="close" />
      <markets-info />
    </section>

    <section
      class="settings-section"
      aria-labelledby="settings-section-prefs"
    >
      <h3
        id="settings-section-prefs"
        class="settings-section__label"
      >
        Preferences
      </h3>
      <settings-language />
      <!-- <settings-theme /> -->
    </section>

    <section
      class="settings-section"
      aria-labelledby="settings-section-network"
    >
      <h3
        id="settings-section-network"
        class="settings-section__label"
      >
        Network
      </h3>
      <settings-network />
      <settings-recent-activity />
    </section>

    <!-- Developer section is the right home for both feature flags and the
         testnet faucet. FaucetMenu self-hides on mainnet; we also gate the
         whole section behind `!isProd` so the divider doesn't appear empty. -->
    <section
      v-if="!isProd"
      class="settings-section settings-section--developer"
      aria-labelledby="settings-section-dev"
    >
      <h3
        id="settings-section-dev"
        class="settings-section__label"
      >
        Developer
      </h3>
      <settings-features />
      <faucet-menu />
    </section>
  </sidebar>
</template>

<style lang="scss">
.settings-sidebar {
  // Lift above the app header (z:100) and any open dialog (Bootstrap's modal
  // slot at 1055) so opening Settings always wins over whatever's behind it.
  // Stays below the popover/tooltip slot (1070/1080) so tooltips rendered
  // inside the drawer (e.g. Disconnect, Copy) still float above its panel.
  // Uses `.sidebar.settings-sidebar` to outrank the shell's `.sidebar { z:100 }`
  // by specificity rather than relying on CSS load order.
  &.sidebar {
    z-index: 1060;
  }

  .sidebar-panel-view {
    gap: 24px;
  }

  .sidebar-body {
    min-height: calc(100% - 36px);
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding-top: $spacing-3xl;
  }

  .setting-item__title {
    color: $text-primary;
    font-size: 14px;
    font-style: normal;
    font-weight: 500;
    line-height: 16px;
  }

  .logo-with-text {
    margin: auto 0 0 auto;
  }

  .markets-info {
    grid-template-columns: 1fr;
  }

  // Hero: the wallet card. Uses the existing bg-card surface so it inherits
  // the app's card language. On connected state `<settings-connect />` renders
  // address + wallet name; on disconnected state it renders the connect CTA
  // full-width, so the padding also wraps the CTA without it feeling detached.
  .settings-sidebar__hero {
    background-color: $bg-card;
    border: 1px solid $border-primary;
    border-radius: $radius-xl;
    padding: 16px;
  }

  // Sections share one layout shell. The inter-row gap drops from 30px to 16px
  // because the section label now does the work of "breathing room" the old
  // heavy 16px/bold title used to do — tighter rows read as a grouped list
  // rather than loose independent items.
  .settings-section {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding-top: 20px;
    border-top: 1px solid $border-primary;

    // Section labels become quiet organizers: 12px uppercase tracked, tertiary
    // color. This flips the old hierarchy where the "Options" title (16px 700
    // near-white) outshouted the actual items (14px 500).
    &__label {
      margin: 0;
      color: $text-tertiary;
      font-size: 11px;
      font-weight: 600;
      line-height: 16px;
      letter-spacing: 0.08em;
      text-transform: uppercase;
    }

    // Developer section is visually de-emphasized so testnet-only tools don't
    // compete with user-facing preferences for attention.
    &--developer {
      .faucet-btn {
        margin-top: 4px;
      }
    }
  }
}
</style>
