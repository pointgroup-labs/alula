export default defineNuxtRouteMiddleware((to) => {
  if (to.path !== '/' && to.path.endsWith('/')) {
    const cleanPath = to.path.replace(/\/+$/, '') || '/'
    return navigateTo({ path: cleanPath, query: to.query, hash: to.hash }, { redirectCode: 301 })
  }
})
