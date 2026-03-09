<script lang="ts" setup>
import { ALULA_URL } from '~/config'
</script>

<template>
  <footer>
    <div class="container">
      <div class="copyright">
        © Copyright {{ new Date().getFullYear() }} <a
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
  color: $text-tertiary;
  font-size: 12px;
  font-style: normal;
  font-weight: 500;
  line-height: 16px;
  border-top: 1px solid $border-color;
  z-index: 1;
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
  color: $text-tertiary;
}
</style>
