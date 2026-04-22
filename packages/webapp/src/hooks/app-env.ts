export function useAppEnv() {
  const config = useRuntimeConfig()
  // @ts-expect-error...
  const env = config.public.APP_ENV

  return {
    env,
    isDev: env === 'dev' || process.env.NODE_ENV === 'development',
    isStage: env === 'stage',
    isProd: env === 'prod' || (!env && process.env.NODE_ENV !== 'development'),
  }
}
