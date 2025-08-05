export default defineNuxtPlugin(() => {
  if (import.meta.client && document.readyState === 'loading') {
    const p = globalThis.location.pathname
    if (p.length > 1 && p.endsWith('/')) {
      const np = p.replace(/\/+$/, '')
      globalThis.history.replaceState({}, '', np + globalThis.location.search + globalThis.location.hash)
    }
  }
})
