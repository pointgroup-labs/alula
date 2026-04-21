<script lang="ts" setup>
import { usePasswordProtect } from '~/features/password-protect'

const { pass, error, login } = usePasswordProtect()
</script>

<template>
  <div class="password-protect">
    <div class="password-protect__bg" />

    <div class="password-protect__content">
      <div class="password-protect__eyebrow">Private access</div>

      <div class="password-protect__hero">
        <div class="password-protect__badge">
          <svg
            width="26"
            height="26"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M8 10V7.5C8 5.01472 10.0147 3 12.5 3C14.9853 3 17 5.01472 17 7.5V10M6.8 10H18.2C18.6418 10 19 10.3582 19 10.8V18.2C19 18.6418 18.6418 19 18.2 19H6.8C6.35817 19 6 18.6418 6 18.2V10.8C6 10.3582 6.35817 10 6.8 10ZM12.5 13V15.5"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </div>

        <div class="password-protect__copy">
          <h1 class="password-protect__title">Protected preview environment</h1>
          <p class="password-protect__subtitle">
            This build is restricted. Enter the access password to continue to the Alula web app.
          </p>
        </div>
      </div>

      <div class="password-protect__card">
        <div class="password-protect__card-header">
          <div>
            <p class="password-protect__label">Access password</p>
            <p class="password-protect__hint">Use the shared internal password to unlock the interface.</p>
          </div>
          <div class="password-protect__status">
            <span class="password-protect__status-dot" />
            Secure
          </div>
        </div>

        <j-input
          v-model="pass"
          type="password"
          placeholder="Enter password"
          autocomplete="current-password"
          class="password-protect__input"
          @keydown.enter.prevent="login"
          @input="error = ''"
        />

        <transition name="fade-up">
          <div
            v-if="error"
            class="password-protect__error"
          >{{ error }}</div>
        </transition>

        <j-btn
          variant="brand"
          class="password-protect__button"
          @click="login"
        >Unlock access</j-btn>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.password-protect {
  position: relative;
  min-height: 100dvh;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: #0a0e17;
  font-family: $font-family-base;
  padding: $spacing-2xl;

  &__bg {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background:
      radial-gradient(700px circle at 18% 20%, rgba(34, 211, 238, 0.08), transparent 62%),
      radial-gradient(560px circle at 82% 74%, rgba(245, 158, 11, 0.07), transparent 60%),
      linear-gradient(180deg, rgba(255, 255, 255, 0.02), transparent 28%);

    &::before {
      content: '';
      position: absolute;
      inset: 0;
      background-image:
        linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
        linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
      background-size: 48px 48px;
      mask-image: radial-gradient(ellipse 85% 70% at 50% 50%, black 42%, transparent 100%);
    }
  }

  &__content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: $spacing-xl;
    width: min(100%, 560px);
  }

  &__eyebrow {
    align-self: center;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 9px 14px;
    border: 1px solid rgba(34, 211, 238, 0.18);
    border-radius: 999px;
    background: rgba(34, 211, 238, 0.08);
    color: #8ceaf5;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    backdrop-filter: blur(16px);

    &::before {
      content: '';
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #22d3ee;
      box-shadow: 0 0 14px rgba(34, 211, 238, 0.9);
    }
  }

  &__hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: $spacing-lg;
    text-align: center;
  }

  &__badge {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 88px;
    height: 88px;
    border-radius: 28px;
    color: #22d3ee;
    background: linear-gradient(180deg, rgba(34, 211, 238, 0.16), rgba(34, 211, 238, 0.04));
    border: 1px solid rgba(34, 211, 238, 0.22);
    box-shadow:
      0 24px 60px rgba(5, 10, 20, 0.48),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
  }

  &__copy {
    display: flex;
    flex-direction: column;
    gap: $spacing-sm;
  }

  &__title {
    margin: 0;
    font-size: clamp(30px, 5vw, 46px);
    line-height: 1.02;
    font-weight: 800;
    letter-spacing: -0.04em;
    color: $text-primary;
    text-wrap: balance;
  }

  &__subtitle {
    margin: 0 auto;
    max-width: 480px;
    font-size: 15px;
    line-height: 1.7;
    color: $text-tertiary;
  }

  &__card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: $spacing-lg;
    padding: $spacing-2xl;
    border-radius: 28px;
    background: linear-gradient(180deg, rgba(17, 24, 39, 0.92), rgba(10, 14, 23, 0.96)), rgba(10, 14, 23, 0.9);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow:
      0 28px 90px rgba(3, 8, 18, 0.5),
      inset 0 1px 0 rgba(255, 255, 255, 0.04);
    backdrop-filter: blur(18px);

    &::before {
      content: '';
      position: absolute;
      inset: 0;
      border-radius: inherit;
      padding: 1px;
      background: linear-gradient(
        135deg,
        rgba(34, 211, 238, 0.18),
        rgba(255, 255, 255, 0.03),
        rgba(245, 158, 11, 0.12)
      );
      -webkit-mask:
        linear-gradient(#fff 0 0) content-box,
        linear-gradient(#fff 0 0);
      mask:
        linear-gradient(#fff 0 0) content-box,
        linear-gradient(#fff 0 0);
      -webkit-mask-composite: xor;
      mask-composite: exclude;
      pointer-events: none;
    }
  }

  &__card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: $spacing-md;
  }

  &__label {
    margin: 0 0 6px;
    font-size: 18px;
    font-weight: 700;
    color: $text-primary;
  }

  &__hint {
    margin: 0;
    max-width: 360px;
    font-size: 14px;
    line-height: 1.6;
    color: $text-tertiary;
  }

  &__status {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
    padding: 8px 12px;
    border-radius: 999px;
    background: rgba(34, 211, 238, 0.08);
    border: 1px solid rgba(34, 211, 238, 0.14);
    color: #8ceaf5;
    font-size: 12px;
    font-weight: 600;
  }

  &__status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #22d3ee;
    box-shadow: 0 0 12px rgba(34, 211, 238, 0.9);
  }

  &__input {
    :deep(.input-group) {
      border-radius: 18px;
      border: 1px solid rgba(255, 255, 255, 0.08);
      background: rgba(255, 255, 255, 0.03);
      transition:
        border-color 0.2s ease,
        box-shadow 0.2s ease,
        background 0.2s ease;
    }

    :deep(.input-wrapper) {
      background: transparent;
    }

    :deep(input) {
      min-height: 56px;
      padding: 16px 18px;
      border: 0;
      background: transparent;
      color: $text-primary;
      font-size: 16px;

      &::placeholder {
        color: rgba(255, 255, 255, 0.34);
      }

      &:focus {
        box-shadow: none;
      }
    }

    :deep(.input-group:focus-within) {
      border-color: rgba(34, 211, 238, 0.42);
      background: rgba(34, 211, 238, 0.06);
      box-shadow: 0 0 0 4px rgba(34, 211, 238, 0.08);
    }
  }

  &__error {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 16px;
    background: rgba(239, 68, 68, 0.08);
    border: 1px solid rgba(239, 68, 68, 0.16);
    color: #fda4af;
    font-size: 14px;
    line-height: 1.4;

    &::before {
      content: '!';
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 20px;
      height: 20px;
      border-radius: 50%;
      background: rgba(239, 68, 68, 0.18);
      color: #fecdd3;
      font-size: 12px;
      font-weight: 700;
      flex-shrink: 0;
    }
  }

  &__button {
    justify-content: center !important;
    width: 100%;
    min-height: 56px;
    border-radius: 18px !important;
    font-size: 15px !important;
    font-weight: 700 !important;
    letter-spacing: 0.01em;
    box-shadow: 0 18px 40px rgba(34, 211, 238, 0.16);
  }
}

.fade-up-enter-active,
.fade-up-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.fade-up-enter-from,
.fade-up-leave-to {
  opacity: 0;
  transform: translateY(6px);
}

@media (max-width: 767px) {
  .password-protect {
    padding: $spacing-lg;

    &__content {
      gap: $spacing-lg;
    }

    &__title {
      font-size: clamp(28px, 10vw, 36px);
    }

    &__card {
      padding: $spacing-xl;
      border-radius: 24px;
    }

    &__card-header {
      flex-direction: column;
    }

    &__status {
      align-self: flex-start;
    }
  }
}
</style>
