import type { Router } from 'vue-router'
// @ts-expect-error...
import NProgress from 'nprogress'
import 'nprogress/nprogress.css'

export default defineNuxtPlugin((nuxtApp) => {
  NProgress.configure({
    showSpinner: false,
    // speed: 400,
    // trickleSpeed: 200,
  })

  const router = nuxtApp.$router as Router

  router.beforeEach(() => {
    NProgress.start()
  })

  router.afterEach(() => {
    NProgress.done()
  })

  router.onError(() => {
    NProgress.done()
  })
})
