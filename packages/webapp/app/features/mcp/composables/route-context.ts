export function useRouteContext() {
  const route = useRoute()

  const {
    BASE_HELPERS,
  } = useHelpersText()

  const helpers = computed(() => {
    const { path/* , params, query  */ } = route

    // simple routes
    if (path === '/') {
      return BASE_HELPERS.value
    }

    return []
  })

  return {
    helpers,
  }
}

function useHelpersText() {
  const BASE_HELPERS = computed(() => {
    return [
      {
        id: '1',
        question: 'What is a Alula Lending?',
        icon: '💰',
      },
      {
        id: '2',
        question: 'Show markets info',
        icon: '🎁',
      },
    ]
  })

  return {
    BASE_HELPERS,
  }
}
