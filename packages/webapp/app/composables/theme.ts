import { useToggle } from '@vueuse/core'
// import Cookies from 'js-cookie'

export const isDark = useDark({
  selector: 'html',
  valueDark: 'theme-dark',
  valueLight: 'theme-light',
  disableTransition: false,
})
export const toggleDark = useToggle(isDark)

/**
 * Default dark theme! You can remove this.
 */
isDark.value = true

// const config = getRuntimeConfig()

// const DOMAIN = config.COOKIE_DOMAIN as string

// const cookieName = 'cookie-theme'
// const themeCookie = Cookies.get(cookieName)

// if (themeCookie) {
//   isDark.value = themeCookie === 'dark'
// }

// watch(isDark, (value) => {
//   Cookies.set(cookieName, value ? 'dark' : 'light', { domain: DOMAIN })
// }, { immediate: true })
