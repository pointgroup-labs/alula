<script lang="ts" setup>
import { ALULA_URL } from '~/config'
</script>

<template>
  <footer>
    <div class="container">
      <div class="copyright">
        © Copyright 2025 <a
          :href="ALULA_URL"
          target="_blank"
        >Alula Finance</a>. All rights reserved.
      </div>

      <nav>
        <nuxt-link to="/terms">
          Terms of Use
        </nuxt-link>
        <nuxt-link to="/policy">
          Privacy Policy
        </nuxt-link>
      </nav>
    </div>
  </footer>
</template>

<style scoped lang="scss">
footer {
  padding: 40px 0;
  color: $neutral-6;
  font-size: 12px;
  font-style: normal;
  font-weight: 500;
  line-height: 16px;
}

.container {
  display: flex;
  justify-content: space-between;
}

nav {
  display: flex;
  gap: 24px;
}

a {
  text-decoration: none;
  color: $neutral-6;
}
</style>
