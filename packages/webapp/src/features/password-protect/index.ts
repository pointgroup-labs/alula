export { default as PasswordProtect } from './components/PasswordProtect.vue'

export function usePasswordProtect() {
  const passwordStorage = useLocalStorage<string>('password-protect', '', { initOnMounted: true })

  const config = useRuntimeConfig()
  // @ts-expect-error...
  const passwordEnv = config.public.PASSWORD_PROTECT

  const pass = ref('')
  const error = ref('')

  const isLogin = computed(() => !passwordEnv || passwordStorage.value === passwordEnv)

  function login() {
    if (String(pass.value).trim() !== String(passwordEnv)) {
      error.value = 'Password incorrect'
      pass.value = ''
      return
    }

    passwordStorage.value = String(pass.value).trim()
  }

  return {
    pass,
    error,
    isLogin,
    isNeedLogin: computed(() => !!passwordEnv && !isLogin.value),
    login,
  }
}
