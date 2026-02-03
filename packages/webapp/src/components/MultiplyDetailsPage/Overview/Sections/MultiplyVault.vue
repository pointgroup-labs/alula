<script lang="ts" setup>
import { VAULT_INFO } from '~/config/vault'

const showMore = ref(false)

const vaultText = computed(() => {
  return showMore.value ? VAULT_INFO.description : `${VAULT_INFO.description.slice(0, 340)}`
})

const vaultHtml = computed(() =>
  `${vaultText.value} <span class="show-more">${showMore.value ? 'Show less' : 'Show more'}</span>`,
)

function moreHandler() {
  showMore.value = !showMore.value
}

onMounted(() => {
  document.querySelector('#vault-text-with-action')
    ?.addEventListener('click', moreHandler)
})
</script>

<template>
  <section id="multiply-vault">
    <h2>{{ VAULT_INFO.title }}</h2>

    <div class="stat-card">
      <p
        id="vault-text-with-action"
        v-html="vaultHtml"
      />
    </div>
  </section>
</template>

<style lang="scss">
section#multiply-vault {
  .stat-card {
    display: flex;

    p {
      font-size: 14px;
      line-height: normal;
    }

    span {
      color: $primary;
      cursor: pointer;
    }
  }
}
</style>
