<script lang="ts" setup>
import VueSlider from 'vue-3-slider-component'

const {
  multiplier,
  maxMultiply,
} = defineProps<{
  multiplier: number | string
  maxMultiply: number | string
}>()

const marks = computed(() => {
  return {
    0: 'x1',
    100: `x${maxMultiply}`,
  }
})

const userSelectedMultiplier = defineModel({
  default: 0,
})

function checkDepositOrBorrow() {
  if (userSelectedMultiplier.value <= 5) {
    userSelectedMultiplier.value = 0
    return
  }
  if (userSelectedMultiplier.value > 5 && userSelectedMultiplier.value <= 10) {
    userSelectedMultiplier.value = 10
  }
}

function opacityHandler(val: any) {
  const multiplier = Number(userSelectedMultiplier.value)
  const value = Number(val.value)

  if (multiplier < 20 && value === 0) {
    return 1 + ((multiplier - 20) / 10)
  }
  if (multiplier > 80 && value === 100) {
    return 1 - (multiplier - 80) / 10
  }
  return 1
}
</script>

<template>
  <div class="loop-multiply">
    <div class="loan-ltv-value">
      <div class="loan-ltv-value__multiplier">
        Multiplier

        <div class="loan-ltv-value__multiplier__value">
          x{{ multiplier }}
        </div>
      </div>
    </div>
    <vue-slider
      v-model="userSelectedMultiplier"
      :interval="0.1"
      :min="0"
      :max="100"
      :dot-size="25"
      :contained="true"
      height="12px"
      tooltip="none"
      class="ltv-select-slider"
      :marks="marks"
      @drag-end="checkDepositOrBorrow"
    >
      <template #dot>
        <div
          class="slider-thumb"
        />
      </template>
      <template #mark="value">
        <div
          class="vue-slider-mark"
          :style="{ left: `${value.pos}%` }"
        >
          <div
            class="vue-slider-mark__label"
            :style="{ opacity: opacityHandler(value) }"
          >
            {{ value?.label }}
          </div>
        </div>
      </template>
    </vue-slider>
  </div>
</template>

<style lang="scss">
$multiplier-color: #c7c7c7;

.loop-multiply {
  --dp-animation-duration: 0;
  display: flex;
  flex-direction: column;
  gap: $spacing-8;
  padding: 8px;

  @media (max-width: 1370px) {
    overflow: inherit;
  }

  .loan-ltv-value {
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 14px;
    font-style: normal;
    font-weight: 500;
    line-height: 16px;

    &__multiplier {
      width: 100%;
      font-size: 12px;
      font-style: normal;
      font-weight: 700;
      line-height: 16px;
      display: flex;
      align-items: center;
      justify-content: space-between;

      &__value {
        display: flex;
        justify-content: center;
        align-items: center;
        gap: 12px;
        padding: $spacing-4 $spacing-8;
        border-radius: $spacing-4;
        background: #d9f7eb;
      }
    }
  }

  .ltv-select-slider {
    cursor: pointer;
    padding-right: 18px !important;
    padding-left: 18px !important;

    .vue-slider-rail,
    .vue-slider-process {
      position: relative;
      background-color: transparent;
    }

    .vue-slider-rail {
      &::before {
        content: '';
        height: 2px;
        width: 100%;
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        left: 0;
        background-color: $multiplier-color;
      }
    }

    .vue-slider-marks {
      position: initial;
      &::before {
        content: '';
        height: 2px;
        width: 9%;
        position: absolute;
        top: 50%;
        transform: translateY(-50%);
        left: 0;
        background-color: $multiplier-color;
      }
    }

    .vue-slider-mark {
      width: 10px;
      height: 10px;
      border-radius: 50%;

      &:first-child,
      &:last-child {
        background-color: $multiplier-color;
      }

      &__label {
        position: absolute;
        top: 14px;
        left: 40%;
        transform: translateX(-50%);
        white-space: nowrap;
        color: #878787;
        font-size: 10px;
        font-style: normal;
        font-weight: 600;
        line-height: 12px;
      }
    }

    .vue-slider-dot {
      cursor: grab;

      .slider-thumb {
        width: 35px;
        height: 35px;
        position: absolute;
        left: 50%;
        top: 50%;
        transform: translate(-50%, -50%);
        background-color: $primary;
        border-radius: 50%;
        box-shadow: 2px 2px 1px 0px rgba(138, 138, 138, 0.25);
      }
    }

    .vue-slider-dot-focus {
      cursor: grab;
    }
  }
}

// body.body--dark {
//   .loop-multiply {
//     .loan-ltv-value__hf {
//       background-color: $green-900;
//       color: #fff;
//     }

//     .ltv-select-slider {
//       .vue-slider-rail::before {
//         background-color: $neutral-600;
//       }

//       .vue-slider-marks {
//         &::before {
//           // background-color: $neutral-800;
//           background-color: $neutral-600;
//         }
//       }

//       .vue-slider-mark {
//         &:first-child,
//         &:last-child {
//           background-color: $neutral-600;
//         }
//       }

//       .vue-slider-dot {
//         .slider-thumb {
//           &--mid {
//             background-color: $purple-300;
//           }
//         }
//       }
//     }
//   }
// }
</style>
