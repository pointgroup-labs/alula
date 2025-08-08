<script setup lang="ts">
import type { ButtonVariant, Size } from 'bootstrap-vue-next'
import chevronIcon from '~/assets/img/icons/arrow-up.svg?raw'
import closeIcon from '~/assets/img/icons/cross-icon.svg?raw'

type SelectOption = {
  label?: string
  value?: any
  default?: boolean
  [key: string]: any
}

const {
  disabled,
  modelValue,
  multiple,
  label,
  search,
  options,
  optionHeight = 34,
  optionValue,
  noReset = false,
} = defineProps<{
  options: SelectOption[]
  modelValue: any
  size?: Size
  variant?: ButtonVariant
  outline?: boolean
  disabled?: boolean
  rounded?: boolean
  label?: string
  multiple?: boolean
  resetIcon?: boolean
  search?: boolean
  optionHeight?: number
  optionValue?: string
  menuClass?: string
  noReset?: boolean
}>()

const emit = defineEmits(['update:modelValue', 'resetSelect'])
const slot = defineSlots()

const element = ref()
const uniqueSelectClass = ref('')
const uniqueMenuClass = ref('')

const selectedOption = ref<SelectOption[]>([])
const showOptions = ref(false)

const searchEl = ref()
const searchInput = ref()

const optionKey = computed(() => optionValue || 'value')

const filteredOptions = computed(() => {
  if (!searchInput.value?.length) {
    return options
  }

  return options.filter(opt => opt.label?.toLowerCase().includes(searchInput.value?.toLowerCase()))
})

const toggleOptions = () => {
  if (disabled) {
    return
  }

  showOptions.value = !showOptions.value
  if (showOptions.value) {
    updateDropdownPosition()
  }
}

function closeOptions() {
  showOptions.value = false
}

defineExpose({
  closeOptions,
})

function resetSelect() {
  selectedOption.value = []
}

function resetWithEmit() {
  resetSelect()
  emit('resetSelect')
}

function selectOption(option: SelectOption) {
  if (option?.default) {
    resetSelect()
    showOptions.value = false
    return
  }

  const isAlreadySelect = selectedOption.value?.some(item => item?.[optionKey.value] === option?.[optionKey.value])

  if (multiple) {
    if (isAlreadySelect) {
      selectedOption.value = selectedOption.value.filter(item => item?.[optionKey.value] !== option?.[optionKey.value])
    } else {
      selectedOption.value.push(option)
    }
    return
  }
  if (noReset && isAlreadySelect) {
    return
  }

  resetSelect()

  if (!isAlreadySelect) {
    selectedOption.value.push(option)
  }
  showOptions.value = false
}

const dropdownStyles = ref<Record<string, string>>({})

const menuHeight = computed(() => {
  const height = filteredOptions.value.length * optionHeight
  let minHeight = Math.max(height, optionHeight + 3)
  let maxHeight = 308
  if (search || slot.search) {
    minHeight += 50
    maxHeight += 10
  }
  return height < 300 ? `${minHeight}px` : `${maxHeight}px`
})

function updateDropdownPosition() {
  if (element.value) {
    const rect = element.value.getBoundingClientRect()

    dropdownStyles.value = {
      position: 'absolute',
      top: `${rect.bottom + window.scrollY + 8}px`,
      left: `${rect.left + window.scrollX}px`,
      width: `${rect.width}px`,
    }
  }
}

function handleClickOutside(event: MouseEvent) {
  const target = event.target as HTMLElement
  if (!target.closest(`.${uniqueSelectClass.value}`) && !target.closest('.dropdown-menu-options')) {
    showOptions.value = false
  }
}

const selectedLabel = computed(() => selectedOption.value.map(item => item?.label).join(', ') || label)

const { list, containerProps, wrapperProps } = useVirtualList(
  filteredOptions,
  {
    // Keep `itemHeight` in sync with the item's row.
    itemHeight: optionHeight,
  },
)

watch(selectedOption, (val) => {
  emit('update:modelValue', multiple ? val : val[0])
})

watch(() => modelValue, (val) => {
  selectedOption.value = Array.isArray(val) ? val : (val ? [val] : [])
}, { immediate: true })

watch(showOptions, (val) => {
  if (val) {
    setTimeout(() => document.addEventListener('dialogHandler', handleClickOutside), 100)
  } else {
    document.removeEventListener('dialogHandler', handleClickOutside)
  }
})

watch(searchEl, (el) => {
  if (el) {
    const input = el.querySelector('input')
    input?.focus()
  }
})

onMounted(() => {
  uniqueSelectClass.value = `j-select-${Math.random().toString(36).slice(2, 11)}`
  uniqueMenuClass.value = `j-dropdown-${Math.random().toString(36).slice(2, 11)}`
})
</script>

<template>
  <div
    ref="element"
    :class="uniqueSelectClass"
  >
    <j-btn
      class="trigger btn-select"
      :variant="variant"
      :size="size"
      :outline="outline"
      :disabled="disabled"
      @click="toggleOptions"
    >
      <div class="select-label">
        <slot
          v-if="slot?.label"
          name="label"
        />
        <span v-else>{{ selectedLabel }}</span>
      </div>
      <template #append>
        <i
          v-if="resetIcon && selectedOption?.length > 0"
          class="chevron-icon"
          @click="resetWithEmit"
          v-html="closeIcon"
        />
        <i
          v-else
          class="chevron-icon"
          :class="{ open: showOptions }"
          v-html="chevronIcon"
        />
      </template>
    </j-btn>
    <teleport to="body">
      <transition name="fade">
        <div
          v-if="showOptions"
          class="dropdown-menu-options"
          :class="[uniqueMenuClass, menuClass]"
          v-bind="containerProps"
          :style="[dropdownStyles, { height: menuHeight }]"
        >
          <div
            v-if="slot?.search"
            ref="searchEl"
          >
            <slot name="search" />
          </div>

          <ul v-bind="wrapperProps">
            <li v-if="search">
              <j-input
                v-model="searchInput"
                placeholder="Search"
                size="sm"
              />
            </li>
            <li
              v-if="filteredOptions.length === 0"
              class="dropdown-options-item no-data"
            >
              No data
            </li>
            <li
              v-for="(option, index) in list"
              :key="option?.data?.[optionKey] || index"
              :class="{ 'active-option': selectedOption?.findIndex(item => item?.[optionKey] === option?.data?.[optionKey]) !== -1 }"
              class="dropdown-options-item"
              @click="selectOption(option.data)"
            >
              <slot
                v-if="slot?.option"
                name="option"
                :option="option.data"
              />
              <span v-else>{{ option?.data.label || option }}</span>
            </li>
          </ul>
        </div>
      </transition>
    </teleport>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.1s linear;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>

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

// body.body--dark {
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
