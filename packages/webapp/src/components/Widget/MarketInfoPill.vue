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
  padding: $spacing-6 $spacing-16;
  background-color: transparent;
  border-radius: 17px;
  border: 1px solid $surface-neutral-10;
  font-size: 12px;
  font-style: normal;
  font-weight: 500;
  line-height: 16px;

  span {
    color: $text-primary;

    &[data-name='title'] {
      color: $text-secondary;
    }
  }

  img {
    width: 16px;
    height: 16px;
    object-fit: contain;

    & + img {
      margin-left: -10px;
    }
  }
}
</style>
