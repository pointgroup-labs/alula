<script lang="ts" setup>
import infoIcon from '~/assets/img/icons/info-circle.svg?raw'

const {
  iconColor = '#111',
  size = 16,
} = defineProps<{
  iconColor?: string
  size?: number
  text?: string
}>()

const slot = defineSlots()
</script>

<template>
  <div
    class="info-tooltip"
    :style="{ '--info-tp-size': `${size}px` }"
  >
    <j-tooltip>
      <i
        :style="{ color: iconColor }"
        v-html="infoIcon"
      />

      <template #content>
        <slot
          v-if="slot?.content"
          name="content"
        />
        <span
          v-else
          v-html="text"
        />
      </template>
    </j-tooltip>
  </div>
</template>

<style lang="scss">
.info-tooltip {
  width: var(--info-tp-size);
  height: var(--info-tp-size);

  i {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  svg,
  i {
    width: 100%;
    height: 100%;
  }
}
</style>
