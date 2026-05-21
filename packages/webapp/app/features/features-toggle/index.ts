export { default as FeaturesToggle } from './components/FeaturesToggle.vue'

export function useFeatureToggle() {
  // @ts-expect-error...
  const config = useRuntimeConfig()?.public

  const baseToggles = config.featureToggle as Record<string, boolean> || {}
  const env = config.APP_ENV

  const localOverrides = useLocalStorage<Record<string, boolean>>(
    'feature-toggles',
    {},
  )

  const isProd = env === 'prod' || (!env && process.env.NODE_ENV !== 'development')

  const toggles = computed(() => {
    return {
      ...baseToggles,
      ...localOverrides.value,
    }
  })

  function isEnabled(name: string) {
    return Boolean(toggles.value[name])
  }

  function enable(name: string) {
    localOverrides.value[name] = true
  }

  function disable(name: string) {
    localOverrides.value[name] = false
  }

  function toggle(name: string) {
    localOverrides.value[name] = !isEnabled(name)
  }

  function reset(name?: string) {
    if (name) {
      delete localOverrides.value[name]
    } else {
      localOverrides.value = {}
    }
  }

  return {
    isProd,
    toggles,
    isEnabled,
    enable,
    disable,
    toggle,
    reset,
  }
}
