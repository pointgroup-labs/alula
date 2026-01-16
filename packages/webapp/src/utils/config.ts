export function getRuntimeConfig() {
  // @ts-expect-error...
  return globalThis.__NUXT__?.config.public ?? {}
}

export async function sleep(ms: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms)
  })
}
