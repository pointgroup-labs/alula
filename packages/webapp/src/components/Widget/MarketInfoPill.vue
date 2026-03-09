<script lang="ts" setup>

</script>

<template>
  <div class="market-info-pill">
    <slot />
  </div>
</template>

<style lang="scss">
.market-info-pill {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: $spacing-sm $spacing-lg;
  background-color: rgba(26, 34, 54, 0.4);
  border-radius: $radius-2xl;
  border: 1px solid $surface-neutral-10;
  font-family: $font-JetBrainsMono;
  font-size: $text-xs;
  font-style: normal;

  span {
    color: $text-primary;

    &[data-name='title'] {
      color: $text-tertiary;
    }
  }

  img {
    width: 14px;
    height: 14px;
    object-fit: contain;

    & + img {
      margin-left: -10px;
    }
  }
}
</style>
