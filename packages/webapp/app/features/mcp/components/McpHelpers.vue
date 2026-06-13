<script lang="ts" setup>
import { useRouteContext } from '../composables'

type Helper = {
  id: string
  question: string
  icon?: string
}

const {
  canSend = true,
} = defineProps<{
  canSend?: boolean
}>()

const emit = defineEmits<{
  select: [question: string]
}>()

const { helpers } = useRouteContext()

const isHelpers = ref(false)

function helpersHandler() {
  isHelpers.value = !isHelpers.value
}

function selectHelper(helper: Helper) {
  emit('select', helper.question)
}
</script>

<template>
  <div
    v-if="helpers.length > 0"
    class="mcp-helpers"
  >
    <div
      class="mcp-helpers__header"
      @click="helpersHandler"
    >
      <div class="mcp-helpers__icon">
        💡
      </div>
      <div class="mcp-helpers__title">
        Quick questions
      </div>

      <i-app-accordion-arrow-down
        v-if="!isHelpers"
        class="mcp-helpers__arrow"
      />
      <i-app-accordion-arrow-up
        v-else
        class="mcp-helpers__arrow"
      />
    </div>
    <div
      v-if="isHelpers"
      class="mcp-helpers__grid"
    >
      <j-btn
        v-for="helper in helpers"
        :key="helper.id"
        class="mcp-helpers__btn"
        :disabled="!canSend"
        variant="secondary"
        @click="selectHelper(helper)"
      >
        <span
          v-if="helper.icon"
          class="mcp-helpers__emoji"
        >{{ helper.icon }}</span>
        <span class="mcp-helpers__text">{{ helper.question }}</span>
      </j-btn>
    </div>
  </div>
</template>

<style lang="scss">
.mcp-helpers {
  position: sticky;
  top: 0;
  display: flex;
  flex-direction: column;
  margin-left: -16px;
  width: calc(100% + 32px);
  backdrop-filter: blur(22px);
  background-color: rgba(#0e5069, 60%);
  box-shadow: 0px 2px 10px 0px rgb(0 0 0 / 8%);

  &__header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: $spacing-md $spacing-lg $spacing-lg;
    cursor: pointer;
  }

  &__arrow {
    margin-left: auto;
    width: 16px;
    height: 16px;
    
    path {
      stroke: $navi-50;
    }
  }

  &__icon {
    font-size: 18px;
    line-height: 1;
  }

  &__title {
    font-size: 13px;
    font-weight: 600;
    color: navi;
  }

  &__grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    padding: 0 $spacing-lg $spacing-lg;

    @media (max-width: $breakpoint-xs) {
      grid-template-columns: 1fr;
    }
  }

  &__btn {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px $spacing-md;
    background: $navi-50;
    border: 1px solid $navi-100;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    height: auto !important;

    &:hover {
      background: $navi-100;
      border-color: $navi-200;
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
    }

    &:active {
      transform: translateY(0);
      box-shadow: 0 1px 4px rgba(0, 0, 0, 0.06);
    }
  }

  &__emoji {
    font-size: 16px;
    line-height: 1;
    flex-shrink: 0;
  }

  &__text {
    font-size: 13px;
    line-height: 16px;
    color: $navi-100;
    font-weight: 500;
    text-align: left;
  }
}
</style>
