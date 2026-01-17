export function useAccordionMarketsHandler(entity: string) {
  const opened = useLocalStorage<string[]>(entity, [], { initOnMounted: true })

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
  return {
    opened,
    isOpened,
    toggleOpen,
  }
}
