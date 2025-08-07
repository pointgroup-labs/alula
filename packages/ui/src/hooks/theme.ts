import { useToggle } from '@vueuse/core'
import Cookies from 'js-cookie'

export const isDark = useDark({
  selector: 'body',
  valueDark: 'body--dark',
  valueLight: 'body--light',
  disableTransition: false,
})
export const toggleDark = useToggle(isDark)
export const preferredDark = usePreferredDark()

const config = getRuntimeConfig()

const DOMAIN = config.COOKIE_DOMAIN as string

const cookieName = 'cookie-theme'
const themeCookie = Cookies.get(cookieName)

if (themeCookie) {
  isDark.value = themeCookie === 'dark'
}

watch(isDark, (value) => {
  Cookies.set(cookieName, value ? 'dark' : 'light', { domain: DOMAIN })
}, { immediate: true })
