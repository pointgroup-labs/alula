<script lang="ts" setup>
import { useFeatureToggle } from '~/features/features-toggle'

const { toggles, toggle } = useFeatureToggle()

const features = computed(() => {
  return Object.entries(toggles.value).map(([key, value]) => {
    return {
      label: key,
      value,
    }
  })
})
</script>

<template>
  <div
    v-if="features"
    class="setting-item features"
  >
    <div class="features-list">
      <div
        v-for="f in features"
        :key="f.label"
        class="feature-item"
        @click="toggle(f.label)"
      >
        <j-checkbox v-model="f.value" />
        {{ f.label }}
      </div>
    </div>
  </div>
</template>

<style lang="scss">
.setting-item.features {
  color: #fff;
  .features-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .feature-item {
    display: flex;
    align-items: center;
    font-size: 14px;
    text-transform: capitalize;
    cursor: pointer;
  }
}
</style>
