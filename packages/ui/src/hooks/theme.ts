export const isDark = useDark({
  selector: 'body',
  valueDark: 'body--dark',
  valueLight: 'body--light',
  disableTransition: false,
})
export const toggleDark = useToggle(isDark)
export const preferredDark = usePreferredDark()

// TODO: remove when dark theme is enabled
isDark.value = false
