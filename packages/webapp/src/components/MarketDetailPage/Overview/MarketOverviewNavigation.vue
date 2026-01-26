<script lang="ts" setup>
const sections = [
  { id: 'stats', label: 'General Stats' },
  { id: 'supply', label: 'Supply Overview' },
  { id: 'borrow', label: 'Borrow Overview' },
  { id: 'info', label: 'Market Info' },
]

const scrollTo = (id: string) => {
  const el = document.querySelector(`#${id}`)
  if (!el) { return }

  const y
    = el.getBoundingClientRect().top
      + window.pageYOffset
      - 40

  window.scrollTo({
    top: y,
    behavior: 'smooth',
  })
}

const activeSection = ref<string>('stats')

const headerOffset = 120

const onScroll = () => {
  const y = window.scrollY + headerOffset + 1

  let current = sections[0]!.id
  for (const section of sections) {
    const id = section.id
    const el = document.querySelector(`#${id}`) as HTMLElement
    if (!el) {
      continue
    }
    if (el.offsetTop <= y) {
      current = id
    }
  }

  activeSection.value = current
}

onMounted(() => {
  if (!isClient) {
    return
  }
  globalThis.addEventListener('scroll', onScroll, { passive: true })
  onScroll()
})
onBeforeUnmount(() => {
  if (!isClient) {
    return
  }
  globalThis.removeEventListener('scroll', onScroll)
})
</script>

<template>
  <div class="overview-navigation">
    <div
      v-for="section in sections"
      :key="section.id"
      class="overview-navigation__item"
      :class="{ active: activeSection === section.id }"
      @click="scrollTo(section.id)"
    >
      {{ section.label }}
    </div>
  </div>
</template>

<style lang="scss">
.overview-navigation {
  min-width: 200px;
  max-width: 240px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  height: fit-content;
  position: sticky;
  top: 20px;

  @media (max-width: $breakpoint-sm) {
    display: none;
  }

  &__item {
    height: 46px;
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 12px 16px;
    cursor: pointer;
    border-radius: 12px;

    &.active {
      background: $purple;
      color: white;
    }
  }
}
</style>
