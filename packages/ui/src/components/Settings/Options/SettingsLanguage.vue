<script lang="ts" setup>
import arrowRight from '~/assets/img/icons/arrow-right.svg?raw'

const { locales, locale, setLocale } = useI18n() as any

const subMenu = ref(false)

const isSidebar = inject<Ref<boolean>>('isSidebar')

function menuHandler() {
  subMenu.value = !subMenu.value
}

watch(() => isSidebar?.value, (val) => {
  if (!val) {
    subMenu.value = false
  }
}, { immediate: true })

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

function handleLanguage(lang: string) {
  setLocale(lang)
}
</script>

<template>
  <div
    class="setting-item language"
    @click="menuHandler"
  >
    <div class="setting-item__title">
      {{ $t('common.language') }}
    </div>
    <div class="language-selected-lang">
      {{ labels[String(currentLanguage) || 'en'] }} <i v-html="arrowRight" />
    </div>
  </div>

  <sidebar-sub-menu
    :is-sub-menu="subMenu"
    :title="$t('common.language')"
    @close="menuHandler"
  >
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
  </sidebar-sub-menu>
</template>

<style lang="scss">
.setting-item.language {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;

  .language-selected-lang {
    color: $neutral-6;
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
      color: $neutral-6;

      svg {
        width: 20px;
        height: 20px;

        path {
          stroke: $neutral-6;
        }
      }
    }
  }
}

.languages-list {
  &__item {
    padding: $spacing-12 0;
    cursor: pointer;

    &:first-child {
      padding-top: $spacing-24;
    }

    &.active {
      font-weight: 500;
    }
  }
}

body.body--dark {
  .setting-item.language {
    .language-selected-lang {
      color: $neutral-12;

      i {
        svg {
          path {
            stroke: $neutral-12;
          }
        }
      }
    }
  }
}
</style>
