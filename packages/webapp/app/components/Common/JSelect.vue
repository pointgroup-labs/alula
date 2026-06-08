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

    <i-app-cross-icon
      v-if="selectedOption && unselected"
      class="cross-icon"
      @click.stop="selectedOption = undefined"
    />

    <i-app-chevron-down
      v-else
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
.j-select {
  .cross-icon {
    cursor: pointer;
    width: 10px !important;
    height: 10px !important;
  }

  .chevron-icon {
    cursor: pointer;
    min-width: 8px !important;
    width: 8px !important;
    height: 8px !important;

    &--show {
      transform: rotate(180deg);
    }
  }
  .dropdown-menu {
    .select-item {
      &:hover {
        background-color: transparent;
        color: $text-primary;
      }

      &--active {
        color: $text-primary;
      }
    }
  }
}
</style>
