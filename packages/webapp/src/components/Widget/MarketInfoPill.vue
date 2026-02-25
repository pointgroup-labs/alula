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
  padding: $spacing-6 $spacing-12;
  background-color: rgba(26, 34, 54, 0.4);
  border-radius: 0.75rem;
  border: 1px solid $surface-neutral-10;
  font-family: $font-JetBrainsMono;
  font-size: $text-xs;
  font-style: normal;

  span {
    color: $foreground;

    &[data-name='title'] {
      color: $muted-foreground;
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
