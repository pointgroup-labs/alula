<script setup lang="ts">
import { identicon } from '@dicebear/collection'
import { createAvatar } from '@dicebear/core'

const props = defineProps<{ address: string }>()

const avatar = ref<string | null>(null)

async function render() {
  await nextTick()
  avatar.value = createAvatar(identicon, {
    seed: props.address,
    backgroundColor: ['0b1020', '111827'],
  }).toString()
  console.log(avatar.value)
}

watch(() => props.address, (address) => {
  if (!address) {
    avatar.value = null
    return
  }
  render()
}, { immediate: true })
</script>

<template>
  <div
    class="address-avatar"
    v-html="avatar"
  />
</template>

<style lang="css" scoped>
.address-avatar {
  width: 18px;
  height: 18px;
  border-radius: 50%;
  border: 1px solid rgba(255, 255, 255, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 3px;
}
</style>
