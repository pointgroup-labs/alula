<script setup lang="ts">
import type { NuxtError } from 'nuxt/app'

const props = defineProps<{ error: NuxtError }>()

const is404 = computed(() =>
  props.error?.statusCode === 404
  || String(props.error.statusMessage).includes('Page Not Found'),
)

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
  <div class="error-page">
    <div class="error-page__bg" />

    <div class="error-page__content">
      <template v-if="is404">
        <div class="error-page__code-wrap">
          <span class="error-page__code">404</span>
          <div class="error-page__glow" />
        </div>
        <p class="error-page__title">Page not found</p>
        <p class="error-page__subtitle">
          The page you're looking for doesn't exist or has been moved.
        </p>
      </template>

      <template v-else>
        <div class="error-page__icon">
          <svg
            width="56"
            height="56"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M12 9V13M12 17H12.01M3.44 18.42L11 4.84C11.43 4.07 12.57 4.07 13 4.84L20.56 18.42C20.97 19.17 20.42 20.08 19.56 20.08H4.44C3.58 20.08 3.03 19.17 3.44 18.42Z"
              stroke="#f59e0b"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>
        <p class="error-page__title">Something went wrong</p>
        <p class="error-page__subtitle">
          {{ (error as any)?.message || (error as any)?.statusMessage || 'An unexpected error occurred.' }}
        </p>
      </template>

      <nuxt-link
        to="/"
        class="error-page__link"
      >
        <j-btn
          variant="ghost"
          size="md"
          class="error-page__btn"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            style="margin-right:6px"
          >
            <path
              d="M19 12H5M5 12L12 19M5 12L12 5"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          Go home
        </j-btn>
      </nuxt-link>
    </div>
  </div>
</template>

<style lang="scss">
.error-page {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100dvh;
  background: #0a0e17;
  overflow: hidden;
  font-family: $font-family-base;

  &__bg {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background:
      radial-gradient(700px circle at 50% 40%, rgba(34, 211, 238, 0.07), transparent 65%),
      radial-gradient(500px circle at 50% 60%, rgba(245, 158, 11, 0.04), transparent 65%);

    &::before {
      content: '';
      position: absolute;
      inset: 0;
      background-image:
        linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
        linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
      background-size: 48px 48px;
      mask-image: radial-gradient(ellipse 80% 60% at 50% 50%, black 40%, transparent 100%);
    }
  }

  &__content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: $spacing-16;
    padding: $spacing-40 $spacing-24;
    max-width: 480px;
    width: 100%;
  }

  &__code-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    margin-bottom: $spacing-8;
  }

  &__code {
    position: relative;
    font-size: clamp(96px, 20vw, 160px);
    font-weight: 800;
    line-height: 1;
    letter-spacing: -0.04em;
    background: linear-gradient(135deg, #22d3ee 0%, #6ee7fb 40%, #a5f3ff 70%, #22d3ee 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    filter: drop-shadow(0 0 40px rgba(34, 211, 238, 0.35));
    user-select: none;
  }

  &__glow {
    position: absolute;
    inset: -20px;
    background: radial-gradient(ellipse at 50% 50%, rgba(34, 211, 238, 0.15), transparent 70%);
    pointer-events: none;
    z-index: -1;
    border-radius: 50%;
    animation: error-glow-pulse 3s ease-in-out infinite;
  }

  &__icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 80px;
    height: 80px;
    border-radius: 20px;
    background: rgba(245, 158, 11, 0.08);
    border: 1px solid rgba(245, 158, 11, 0.2);
    margin-bottom: $spacing-8;
  }

  &__title {
    font-size: clamp(20px, 4vw, 28px);
    font-weight: 700;
    color: $text-primary;
    margin: 0;
    line-height: 1.2;
  }

  &__subtitle {
    font-size: 15px;
    color: $muted-foreground;
    line-height: 1.6;
    margin: 0;
    max-width: 360px;
  }

  &__link {
    margin-top: $spacing-8;
    text-decoration: none;
  }

  &__btn {
    display: inline-flex !important;
    align-items: center !important;
    gap: 6px;
    padding: $spacing-10 $spacing-24 !important;
    border-radius: $radius-8 !important;
    border: 1px solid rgba(34, 211, 238, 0.25) !important;
    color: #22d3ee !important;
    background: rgba(34, 211, 238, 0.06) !important;
    font-size: 14px !important;
    font-weight: 500 !important;
    transition:
      background 0.2s ease,
      border-color 0.2s ease,
      box-shadow 0.2s ease !important;

    &:hover {
      background: rgba(34, 211, 238, 0.12) !important;
      border-color: rgba(34, 211, 238, 0.45) !important;
      box-shadow: 0 0 20px rgba(34, 211, 238, 0.1) !important;
    }
  }
}

@keyframes error-glow-pulse {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.6;
    transform: scale(0.92);
  }
}
</style>
