<script lang="ts" setup>
import { TERMS_VERSION } from '../constants'
import { TermsContent } from '../index'

const key = computed(() => `termsAccepted:${TERMS_VERSION}`)
const acceptedTerms = useLocalStorage(key, false, { initOnMounted: true })

const dialog = ref(false)
const accepted = ref(false)
const isScroll = ref(false)
const termsWrapper = ref<HTMLElement | null>(null)

function updateScrollState() {
  if (isScroll.value) {
    return
  }

  const element = termsWrapper.value

  if (!element) {
    return
  }

  const maxScrollTop = element.scrollHeight - element.clientHeight

  if (maxScrollTop <= 0) {
    isScroll.value = true
    return
  }

  const isAtBottom = element.scrollTop >= maxScrollTop - 20

  if (isAtBottom) {
    isScroll.value = true
  }
}

function acceptHandler() {
  if (!accepted.value) {
    return
  }
  dialog.value = false
  acceptedTerms.value = true
}

onMounted(() => {
  dialog.value = !acceptedTerms.value
})

watch(dialog, (isOpen) => {
  if (!isOpen) {
    return
  }

  isScroll.value = false
  accepted.value = false

  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      updateScrollState()
    })
  })
})

function updateAccepted(value: boolean) {
  if (!isScroll.value) {
    return
  }

  accepted.value = value
}
</script>

<template>
  <j-dialog
    v-model="dialog"
    class-name="terms-dialog"
    title="Terms"
    no-close-on-backdrop
  >
    <template #header>
      Terms
    </template>

    <div class="dialog-default__body terms-dialog__body">
      <div
        ref="termsWrapper"
        class="terms-wrapper"
        tabindex="0"
        @scroll="updateScrollState"
      >
        <terms-content />
      </div>

      <Transition name="terms-hint">
        <div
          v-if="!isScroll"
          class="terms-hint"
        >
          Scroll to the bottom to enable acceptance
        </div>
      </Transition>

      <div class="terms-dialog__actions">
        <j-checkbox
          :model-value="accepted"
          class="terms-dialog__checkbox"
          :class="{ 'terms-dialog__checkbox--locked': !isScroll }"
          :disabled="!isScroll"
          @update:model-value="updateAccepted"
        >
          I have read and agree to the Terms of Service
        </j-checkbox>

        <j-btn
          variant="brand"
          style="width: 100%;"
          :disabled="!accepted"
          @click="acceptHandler"
        >
          Accept
        </j-btn>
      </div>
    </div>
  </j-dialog>
</template>

<style lang="scss">
.terms-dialog {
  .modal-dialog .modal-content {
    max-width: 600px;

    @media (max-width: $breakpoint-xs) {
      padding-top: 12px;
    }
  }

  .modal-header {
    height: 65px;

    @media (max-width: $breakpoint-xs) {
      height: 35px;
    }

    .close-icon {
      display: none;
    }
  }

  .modal-body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  &__body {
    flex: 1;
    padding: $spacing-xl $spacing-3xl $spacing-3xl;
    display: flex;
    flex-direction: column;
    gap: $spacing-xl;
    min-height: 0;
  }

  &__actions {
    display: flex;
    flex-direction: column;
    gap: $spacing-lg;
    padding-top: $spacing-xs;
  }

  &__checkbox {
    transition: opacity 0.2s ease;

    &--locked {
      opacity: 0.55;
      pointer-events: none;
      user-select: none;
    }
  }

  .terms-hint {
    font-size: 12px;
    color: $text-tertiary;
    text-align: center;
    margin-top: -6px;
    margin-bottom: -8px;
  }

  .terms-wrapper {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    overflow-x: hidden;
    background: linear-gradient(180deg, color-mix(in oklab, $navi-500 88%, white) 0%, $navi-500 100%);
    border-radius: 12px;
    border: 1px solid $border-secondary;
    max-height: min(500px, 55vh);
    color: $text-secondary;
    font-size: 14px;
    line-height: 1.7;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.03),
      0 12px 32px rgba(0, 0, 0, 0.12);
    overscroll-behavior: contain;
    scroll-behavior: smooth;

    &::-webkit-scrollbar {
      width: 8px;
    }

    &::-webkit-scrollbar-thumb {
      border-radius: 999px;
      background-color: color-mix(in oklab, $border-secondary 72%, white);
    }

    &::-webkit-scrollbar-track {
      background: transparent;
    }

    &:focus-visible {
      outline: 1px solid color-mix(in oklab, $border-secondary 55%, white);
      outline-offset: 0;
    }
  }
}

/* enter */
.terms-hint-enter-from {
  opacity: 0;
  transform: translateY(6px) scale(0.98);
}

.terms-hint-enter-active {
  transition:
    opacity 0.25s ease,
    transform 0.25s cubic-bezier(0.22, 1, 0.36, 1);
}

.terms-hint-enter-to {
  opacity: 0.7;
  transform: translateY(0) scale(1);
}

/* leave */
.terms-hint-leave-from {
  opacity: 0.7;
  transform: translateY(0) scale(1);
}

.terms-hint-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.terms-hint-leave-to {
  opacity: 0;
  transform: translateY(4px) scale(0.98);
}
</style>
