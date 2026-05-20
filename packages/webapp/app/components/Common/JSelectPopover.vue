<template>
  <j-popover
    position="bottom"
    :teleport-to-body="false"
    close-popup
    class-name="j-popover-select"
    menu-class="j-popover-select__menu"
    :teleport="false"
  >
    <slot name="menu" />
    <template #target="{ active }">
      <slot name="target" />
      <i-app-arrow-up
        class="arrow-icon"
        :class="{ 'arrow-icon--active': active }"
      />
    </template>
  </j-popover>
</template>

<style lang="scss">
.j-popover-select {
  display: flex;
  align-items: center;
  gap: 2px;
  cursor: pointer;

  &__menu {
    --bs-popover-body-padding-y: 0.6rem;

    .popover-wrapper {
      cursor: pointer;
    }
  }

  .arrow-icon {
    width: 18px;
    height: 18px;
    transform: rotate(180deg);

    &--active {
      transform: rotate(0deg);
    }
  }
}
</style>
