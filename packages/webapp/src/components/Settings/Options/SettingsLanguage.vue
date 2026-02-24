<script lang="ts" setup>
import arrowRight from '~/assets/img/icons/arrow-right.svg?raw'

const { locales, locale, setLocale } = useI18n() as any

const pop = inject<() => void>('sidebarPop')

const labels = {
  en: 'English',
  de: 'Deutsch',
  jp: '日本語',
  cn: '中文',
  hi: 'हिन्दी',
  id: 'Indonesian',
  ru: 'Русский',
  ua: 'Українська',
} as Record<string, string>

const keys = Object.keys(labels)

const languages = computed(() => keys.filter(key => locales.value.some((l: Record<string, string>) => l.code === key)))
const currentLanguage = computed(() => locale.value)

async function handleLanguage(lang: string) {
  await setLocale(lang)
  pop?.()
}
</script>

<template>
  <sidebar-panel :title="$t('common.language')">
    <template #trigger>
      <div class="setting-item language">
        <div class="setting-item__title">
          {{ $t('common.language') }}
        </div>
        <div class="language-selected-lang">
          {{ labels[String(currentLanguage) || 'en'] }} <i v-html="arrowRight" />
        </div>
      </div>
    </template>

    <div class="languages-list">
      <div
        v-for="lang in languages"
        :key="lang"
        :class="{ active: lang === currentLanguage }"
        class="languages-list__item"
        @click="handleLanguage(lang)"
      >
        {{ labels[lang] }}
      </div>
    </div>
  </sidebar-panel>
</template>

<style lang="scss">
.setting-item.language {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;

  .language-selected-lang {
    color: $text-primary;
    font-size: 16px;
    font-style: normal;
    font-weight: 400;
    line-height: 20px;
    display: flex;
    align-items: center;
    gap: $spacing-8;
    cursor: pointer;

    i {
      display: flex;
      align-items: center;
      color: $text-primary;

      svg {
        width: 16px;
        height: 16px;

        path {
          stroke: $text-primary;
        }
      }
    }
  }
}

.languages-list {
  &__item {
    color: $text-primary;
    padding: $spacing-12 0;
    cursor: pointer;

    &.active {
      font-weight: 700;
    }
  }
}
</style>
