<template>
  <table-mobile-card
    class="card"
    style="border-bottom: none;"
  >
    <div class="mobile-card-top">
      <div class="card-asset">
        <j-skeleton
          variant="rounded"
          width="32"
          height="32"
        />

        <j-skeleton
          height="16"
          pill
        />
      </div>

      <div
        class="card-top-info"
      >
        <j-skeleton
          v-for="value in 2"
          :key="value"
          height="16"
          pill
        />
      </div>
    </div>

    <div class="mobile-card-body">
      <j-skeleton
        height="16"
        pill
      />

      <div class="separator-vert" />

      <j-skeleton
        height="16"
        pill
      />
    </div>

    <div class="mobile-card-footer">
      <j-skeleton
        height="32"
        pill
        full-width
      />
    </div>
  </table-mobile-card>
</template>
