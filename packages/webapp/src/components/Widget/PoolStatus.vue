<script lang="ts" setup>
import type { Pool } from '@alula/market-sdk'
import { decodePoolStatus, POOL_STATUS_RESTRICTED_MESSAGES } from '@alula/client-sdk'

const { pool } = defineProps<{
  pool: Pool
}>()

const allStatuses = ref<string[]>([])

const isRestricted = computed(() => allStatuses.value.length > 0)

watch(() => pool, () => {
  allStatuses.value = []
  const flags = pool?.config?.status?.flags
  if (!flags) {
    return
  }

  const decodedPoolStatus = decodePoolStatus(flags)

  for (const [key, value] of Object.entries(decodedPoolStatus)) {
    if (!value) {
      allStatuses.value.push(POOL_STATUS_RESTRICTED_MESSAGES[key as keyof typeof POOL_STATUS_RESTRICTED_MESSAGES])
    }
  }
}, { immediate: true })
</script>

<template>
  <div class="pool-status">
    <j-tooltip
      v-if="isRestricted"
      content-class="pool-status__tip"
    >
      <j-pill-label
        color="#ffb726"
        variant="outline-warning"
        size="sm"
      >
        Restricted
      </j-pill-label>
      <template #content>
        The following operations are unavailable in this pool:
        <ul>
          <li
            v-for="status in allStatuses"
            :key="status"
          >{{ status }}</li>
        </ul>
      </template>
    </j-tooltip>

  </div>
</template>

<style lang="scss">
.pool-status {
  position: absolute;
  top: -16px;
  left: 22px;

  .j-pill-label {
    padding: 0 8px;
  }
}

.pool-status__tip {
  text-align: left;

  ul {
    padding: 0 0 0 18px;
    margin: 0;
  }
}
</style>
