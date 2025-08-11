<script setup lang="ts">
import type { NuxtError } from 'nuxt/app'

const props = defineProps<{ error: NuxtError }>()

const is404 = computed(() =>
  props.error?.statusCode === 404
  || String(props.error.statusMessage).includes('Page Not Found'),
)

const goBack = () => clearError({ redirect: '/' })

onMounted(() => {
  if (import.meta.client) {
    nextTick(() => {
      const body = document.querySelector('body') as HTMLElement
      if (body) {
        body.style.transition = 'opacity 0.3s ease-in-out'
        body.style.opacity = '1'
      }
    })
  }
})
</script>

<template>
  <div class="page-404">
    <div class="error-wrapper">
      <h1
        v-if="is404"
        data-name="error"
      >404</h1>
      <template v-else>
        <h1>Oops... something went wrong</h1>
        <p>
          {{ (error as any)?.message || (error as any)?.statusMessage || 'Unknown error' }}
        </p>
      </template>

      <j-btn
        variant="secondary"
        size="md"
        class="home-page-btn"
        @click="goBack"
      >
        Home Page
      </j-btn>
    </div>
  </div>
</template>

<style lang="scss">
.page-404 {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 87vh;
  font-size: 56px;
  color: #2a2e4694;

  .error-wrapper {
    text-align: center;
  }

  p {
    font-size: 22px;
  }
}

body.body--dark {
  .page-404 {
    color: $neutral-16;
    .home-page-btn {
      background-color: $neutral-18;
      border-color: $neutral-18;
      color: $neutral-3;
    }
  }
}
</style>
