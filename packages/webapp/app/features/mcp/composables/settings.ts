const defaultSettings = {
  isGlass: false,
}

export function useMcpSettings() {
  const settings = useLocalStorage('mcp-settings', defaultSettings, { initOnMounted: true })
  return { settings }
}
