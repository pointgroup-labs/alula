<script lang="ts" setup>
import { HALBORN_AUDITOR } from '~/config/audits'

function normalizeDate(date: string): string {
  const dateObj = new Date(date)
  return `${String(dateObj.getDate()).padStart(2, '0')}.${String(dateObj.getMonth() + 1).padStart(2, '0')}.${dateObj.getFullYear()}`
}
</script>

<template>
  <section id="pool-info-overview">
    <div class="stat-card">
      <div class="stat-card__header">
        <h3 class="title">
          Audits
        </h3>
      </div>

      <div class="stat-card__body">
        <div class="info-list">
          <div class="info-list__item auditor-item">
            <div class="title">
              Audited ({{ normalizeDate(HALBORN_AUDITOR.auditedAt) }})
            </div>
            <a
              class="value"
              :href="HALBORN_AUDITOR.link"
              target="_blank"
              rel="noopener noreferrer nofollow"
            >
              <img
                :src="HALBORN_AUDITOR.logo"
                alt="auditor logo"
              >
              {{ HALBORN_AUDITOR.name }}
              <i-app-export-icon class="export-icon" />
            </a>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
