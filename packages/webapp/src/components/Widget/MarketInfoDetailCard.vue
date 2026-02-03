<template>
  <div class="market-info-detail-card">
    <slot />
  </div>
</template>

<style lang="scss">
.market-info-detail-card {
  position: relative;
  display: flex;
  flex-direction: column;
  text-wrap: nowrap;
  justify-content: center;
  align-items: center;
  padding: 20px 28px;
  background: linear-gradient(135deg, rgba(255, 255, 255, 0.95) 0%, rgba(249, 250, 251, 0.98) 100%);
  border: 1px solid rgba(33, 44, 66, 0.08);
  box-shadow:
    0px 2px 8px rgba(33, 44, 66, 0.04),
    0px 8px 24px rgba(33, 44, 66, 0.06),
    inset 0px 1px 0px rgba(255, 255, 255, 0.6);
  border-radius: 16px;
  font-weight: 500;
  height: fit-content;
  position: relative;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);

  &:hover {
    box-shadow:
      0px 4px 12px rgba(33, 44, 66, 0.08),
      0px 12px 32px rgba(33, 44, 66, 0.1),
      inset 0px 1px 0px rgba(255, 255, 255, 0.8);
    border-color: rgba(92, 108, 255, 0.3);
  }

  .info-label {
    font-size: 24px;
    line-height: 32px;
    font-weight: 600;
    background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    letter-spacing: -0.02em;
  }

  .info-label,
  .info-value {
    &.positive {
      background: linear-gradient(135deg, #059669 0%, #10b981 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
    &.warning {
      background: linear-gradient(135deg, #dc2626 0%, #f59e0b 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;
    }
  }

  .info-value {
    font-size: 13px;
    line-height: 20px;
    display: flex;
    align-items: center;
    gap: 6px;
    color: #64748b;
    font-weight: 500;
    margin-top: 2px;

    &.positive {
      color: $success;
    }
    &.warning {
      color: $warning;
    }
  }

  .token-icon {
    width: 16px;
    height: 16px;
    object-fit: contain;
    filter: drop-shadow(0px 1px 2px rgba(0, 0, 0, 0.1));
  }

  .info-icon {
    position: absolute;
    top: 10px;
    right: 10px;
  }
}

.theme-dark {
  .market-info-detail-card {
    background: linear-gradient(135deg, rgba(30, 41, 59, 0.6) 0%, rgba(15, 23, 42, 0.8) 100%);
    border-color: rgba(148, 163, 184, 0.1);
    box-shadow:
      0px 2px 8px rgba(0, 0, 0, 0.2),
      0px 8px 24px rgba(0, 0, 0, 0.3),
      inset 0px 1px 0px rgba(255, 255, 255, 0.05);
    backdrop-filter: blur(12px);

    &:hover {
      box-shadow:
        0px 4px 12px rgba(0, 0, 0, 0.3),
        0px 12px 32px rgba(99, 102, 241, 0.15),
        inset 0px 1px 0px rgba(255, 255, 255, 0.1);
      border-color: rgba(92, 108, 255, 0.3);
    }

    .info-label {
      background: linear-gradient(135deg, #e2e8f0 0%, #cbd5e1 100%);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
      background-clip: text;

      &.positive {
        background: linear-gradient(135deg, #10b981 0%, #34d399 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
      }
      &.warning {
        background: linear-gradient(135deg, #f59e0b 0%, #fbbf24 100%);
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
      }
    }

    .info-value {
      color: #94a3b8;
    }
  }
}
</style>
