export function useAccordionMarketsHandler(entity: string) {
  const opened = useLocalStorage<string[]>(entity, [], { initOnMounted: true })

  const marketsStore = useMarketsStore()

  function toggleOpen(entity: string) {
    const index = opened.value.indexOf(entity)
    if (index === -1) {
      opened.value.push(entity)
    } else {
      opened.value.splice(index, 1)
    }
  }

  function isOpened(entity: string) {
    return opened.value.includes(entity)
  }

  function showAll() {
    const markets = marketsStore.state.markets ?? {}
    const keys = Object.keys(markets)
    opened.value = keys
  }

  function collapseAll() {
    opened.value = []
  }
  return {
    opened,
    isOpened,
    toggleOpen,
    showAll,
    collapseAll,
  }
}
