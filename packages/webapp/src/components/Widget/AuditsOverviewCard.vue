<script lang="ts" setup>
import { AUDITORS } from '~/config/audits'

// Single source of truth for the audit list lives in `~/config/audits`.
// Both pool and multiply overview tabs render this card; the transparency
// page renders a richer per-auditor grid against the same `AUDITORS` array.
function normalizeDate(date: string): string {
  const d = new Date(date)
  const dd = String(d.getDate()).padStart(2, '0')
  const mm = String(d.getMonth() + 1).padStart(2, '0')
  return `${dd}.${mm}.${d.getFullYear()}`
}
</script>

<template>
  <section id="pool-info-overview">
    <div class="stat-card">
      <div class="stat-card__body">
        <div class="info-list">
          <div
            v-for="auditor in AUDITORS"
            :key="auditor.name"
            class="info-list__item auditor-item"
          >
            <div class="title">
              <template v-if="auditor.status === 'completed'">
                Audited ({{ normalizeDate(auditor.auditedAt) }})
              </template>
              <template v-else>
                Audit in progress
              </template>
            </div>
            <a
              v-if="auditor.status === 'completed'"
              class="value"
              :href="auditor.link"
              target="_blank"
              rel="noopener noreferrer nofollow"
            >
              <img
                :src="auditor.logo"
                alt="auditor logo"
              >
              {{ auditor.name }}
              <i-app-export-icon class="export-icon" />
            </a>
            <span
              v-else
              class="value auditor-item__pending"
            >
              <img
                :src="auditor.logo"
                alt="auditor logo"
              >
              {{ auditor.name }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style lang="scss" scoped>
// Logos come from arbitrary auditors with different aspect ratios, so we
// lock the height and let width flow naturally with `object-fit: contain`
// as a safety net. The previous unscoped global rule used a fixed 18x12
// box which squashed any logo whose native aspect didn't match Halborn's.
.auditor-item {
  img {
    height: 16px;
    width: auto;
    max-width: 96px;
    object-fit: contain;
    flex-shrink: 0;
  }
}

// Pending auditors render the same row layout but without a link, so the
// label looks identical to the completed entry minus the export icon and
// hover affordance.
.auditor-item__pending {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: $text-tertiary;
  cursor: default;
}
</style>
