<template>
  <div class="multiply-position">
    <div class="multiply-position__wrapper">
      <my-multiply-positions />
    </div>
  </div>
</template>

<style lang="scss">
.multiply-position {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding-bottom: 120px;

  &__wrapper {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 64px;
  }
}
</style>
