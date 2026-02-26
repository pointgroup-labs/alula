<script setup lang="ts">
export type SelectOption = {
  label: string | number
  value: string | number
  default?: boolean
  disabled?: boolean
  [key: string]: any
}

type RawOption = string | number | SelectOption

type NormalizedOption = SelectOption & {
  raw: RawOption
}

const props = withDefaults(defineProps<{
  options: RawOption[]
  className?: string
  menuClass?: string
  label?: string
  isSearch?: boolean
  unselected?: boolean
}>(), {
  className: '',
  menuClass: '',
  label: '',
  isSearch: false,
  unselected: true,
})

const slots = defineSlots()

const isShow = ref(false)
const search = ref<string | undefined>()

const uniqueSelectClass = ref<string>()
const uniqueMenuClass = ref<string>()

const selectedOption = defineModel<RawOption | undefined>()

const normalizedOptions = computed<NormalizedOption[]>(() =>
  props.options.map((option) => {
    if (typeof option !== 'object') {
      return {
        label: String(option),
        value: String(option),
        raw: option,
      }
    }

    return {
      ...option,
      raw: option,
    }
  }),
)

const selectedValue = computed(() => {
  if (!selectedOption.value) {
    return
  }

  return typeof selectedOption.value === 'object'
    ? selectedOption.value.value
    : selectedOption.value
})

const selectedLabel = computed(() => {
  const found = normalizedOptions.value.find(o => String(o.value) === String(selectedValue.value))
  return found?.label
})

const filteredOptions = computed(() => {
  if (!props.isSearch || !search.value) {
    return normalizedOptions.value
  }

  const q = search.value.toLowerCase()

  return normalizedOptions.value.filter(option =>
    String(option.label).toLowerCase().includes(q)
    || String(option.value).toLowerCase().includes(q),
  )
})

function selectHandler(option: NormalizedOption) {
  if (option.disabled) {
    return
  }

  if (selectedValue.value === option.value && props.unselected) {
    const allVal = normalizedOptions.value.find(o => [String(o.label).toLowerCase(), String(o.value).toLowerCase()].includes('all'))
    selectedOption.value = allVal?.raw
    return
  }

  selectedOption.value = option.raw
}

function keypressEnterHandler() {
  if (filteredOptions.value.length !== 1) {
    return
  }

  selectHandler(filteredOptions.value[0]!)
  document
    .querySelector<HTMLElement>(`.${uniqueSelectClass.value}`)
    ?.click()
}

function showHandler(actionType: 'show' | 'hide') {
  isShow.value = actionType === 'show'
  setTimeout(() => search.value = undefined, 200)
}

// watch(isShow, (val) => {
//   if (val) {
//     focusElWithDelay(`.${uniqueSelectClass.value} input`)
//   }
// })

onMounted(() => {
  uniqueSelectClass.value = `j-select-${Math.random().toString(36).slice(2, 11)}`
  uniqueMenuClass.value = `j-dropdown-${Math.random().toString(36).slice(2, 11)}`
})
</script>

<template>
  <j-dropdown
    :menu-class="`select-menu ${menuClass} ${uniqueMenuClass}`"
    :class-name="`j-select ${className} ${uniqueSelectClass}`"
    variant="outline"
    auto-close
    @show-handler="showHandler"
  >
    <!-- Trigger -->
    <slot
      v-if="slots.label"
      name="label"
    />
    <slot v-else>
      {{ selectedLabel || label }}
    </slot>

    <i-app-chevron-down
      class="chevron-icon"
      :class="isShow ? 'chevron-icon--show' : 'chevron-icon--hide'"
    />

    <!-- Menu -->
    <template #menu>
      <j-input
        v-if="isSearch"
        v-model="search"
        class="search-select-input"
        @click.stop
        @keypress.enter="keypressEnterHandler"
      />

      <template v-if="filteredOptions.length > 0">
        <li
          v-for="option in filteredOptions"
          :key="option.value"
          class="select-item"
          :class="[
            String(option.value) === String(selectedValue) && 'select-item--active',
            option.disabled && 'select-item--disabled',
          ]"
          @click="selectHandler(option)"
        >
          <slot
            v-if="slots.option"
            name="option"
            :option="option.raw"
          />
          <template v-else>
            {{ option.label }}
          </template>
        </li>
      </template>

      <template v-else>
        <li class="select-item no-data">
          No Data
        </li>
      </template>
    </template>
  </j-dropdown>
</template>

<style lang="scss">
.btn-select {
  display: flex !important;
  align-items: center;
  gap: 6px;
  white-space: nowrap;

  .btn-content {
    width: 100%;
  }

  .select-label {
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    text-align: left;
  }

  .chevron-icon {
    transform: rotate(180deg);
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    width: 24px;
    height: 24px;

    &.open {
      transform: rotate(0deg);
    }
  }
}

.dropdown-menu-options {
  background: #fff;
  position: absolute;
  z-index: 9999;
  box-shadow: 0 0 2px 0px #00000054;
  border-radius: 4px;
  width: max-content;

  ul {
    list-style: none;
    padding: 6px;
    margin-bottom: 0;
  }

  .j-input {
    margin-top: $spacing-4;
    padding: 0 $spacing-4;
  }

  .active-option {
    background-color: $neutral-5;
  }

  .dropdown-options-item {
    outline: none;
    padding: 6px;
    cursor: pointer;

    &:hover {
      background-color: darken($neutral-5, 2%);
    }

    &.no-data {
      text-align: center;
      justify-content: center;
      pointer-events: none;
    }
  }

  .j-input {
    .input-group {
      border-radius: $spacing-4;
    }
  }
}

// .theme-dark {
//   .dropdown-menu-options {
//     background-color: $dark-bg;
//     border: 1px solid $neutral-16;
//   }

//   .dropdown-options-item:hover {
//     background-color: $neutral-16;
//   }

//   .dropdown-menu-options .active-option {
//     background-color: $neutral-16;
//   }
// }
</style>
