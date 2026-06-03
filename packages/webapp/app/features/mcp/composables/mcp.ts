import { fetchMCP } from '../api/api'
import { parseMd, typeText } from '../utils'

const bpsToPct = (x: bigint) => Number(x) / 100

export function useMcp() {
  const sessionId = crypto.randomUUID()

  const route = useRoute()
  const router = useRouter()

  const marketsTableStore = useMarketTableStore()
  const preparedMarkets = computed(() => marketsTableStore.marketWithTableItems?.map((m) => {
    const { assets, marketSize, tableItems, ...rest } = m
    const pools = tableItems?.map((a) => {
      const { raw, action, assetDecimals, asset, ...rest } = a
      const symbol = asset.symbol
      const kink2_ur_bps = raw.pool.config.interest_rate_model.values[0].kink2_ur_bps ?? 8000n
      const close_ltv = bpsToPct(BigInt(kink2_ur_bps)) || 80
      return {
        ...rest,
        symbol,
        close_ltv,
      }
    })
    return {
      ...rest,
      pools,
    }
  }))

  const isOpen = ref(false)
  const hasUnreadMessage = ref(false)
  const analizyng = ref(false)
  const isTyping = ref(false)
  const draft = ref('')
  const messageId = ref(1)

  const messages = ref<ChatMessage[]>([])

  const canSend = computed(() => draft.value.trim().length > 0 && !analizyng.value && !isTyping.value)

  function addMessage(
    from: ChatMessage['from'],
    text: string,
    streaming = false,
  ) {
    const msg: ChatMessage = {
      id: messageId.value++,
      from,
      text,
      html: streaming ? '' : parseMd(text),
      streaming,
      time: formatTime(new Date()),
    }

    messages.value.push(msg)
    return msg
  }

  function generateArgs() {
    const { params } = route
    const marketsData = preparedMarkets.value
    const args: Record<string, unknown> = {}
    if (params?.market) {
      args.market = params.market
    }
    if (params?.pool) {
      args.pool = params.pool
    }
    if (marketsData) {
      args.marketsData = marketsData
    }

    return args
  }

  async function sendMessage() {
    if (!canSend.value) {
      return
    }

    const args = generateArgs()

    const content = draft.value.trim()
    draft.value = ''

    addMessage('user', content)
    analizyng.value = true
    isTyping.value = true

    try {
      const response = await fetchMCP(content, sessionId, args)

      if (!response) {
        throw new Error('Empty response')
      }

      addMessage('support', '', true)
      const msgIndex = messages.value.length - 1
      analizyng.value = false

      if (!isOpen.value) {
        hasUnreadMessage.value = true
      }

      // 👉 streaming RAW text
      await typeText(response, (chunk) => {
        const msg = messages.value[msgIndex]
        if (msg) {
          msg.text = chunk
          msg.html = renderStreamingMarkdown(chunk)
        }
      })

      isTyping.value = false

      const msg = messages.value[msgIndex]
      if (msg) {
        msg.html = parseMd(msg.text)
        msg.streaming = false
      }
    } catch (error) {
      console.error(error)
      addMessage('support', 'Oops \u2014 something didn\u2019t work as expected. \uD83E\uDEE3 Please try again.')
      if (!isOpen.value) {
        hasUnreadMessage.value = true
      }
    } finally {
      analizyng.value = false
      isTyping.value = false
    }
  }

  // @ts-expect-error...
  watchDebounced(() => route.query, (query: Record<string, string>) => {
    const question = query?.question
    if (question) {
      isOpen.value = true
      draft.value = String(question)
      nextTick(() => {
        sendMessage()
        const query = { ...route.query }
        delete query.question
        router.replace({ query })
      })
    }
  }, { immediate: true, debounce: 1000 })

  return {
    messages,
    isOpen,
    analizyng,
    isTyping,
    draft,
    canSend,
    openChat: () => {
      isOpen.value = true
      hasUnreadMessage.value = false
    },
    closeChat: () => (isOpen.value = false),
    hasUnreadMessage,

    addMessage,
    sendMessage,
    renderStreamingMarkdown,
  }
}

function renderStreamingMarkdown(text: string) {
  if (!text) {
    return ''
  }

  const fenceMatches = text.match(/```/g)

  if (fenceMatches && fenceMatches.length % 2 === 1) {
    const lastFence = text.lastIndexOf('```')
    const head = text.slice(0, lastFence)
    const tail = text.slice(lastFence)
    const renderedHead = head ? parseMd(head) : ''
    const renderedTail = `<span class="streaming-tail">${escapeHtml(tail).replaceAll('\n', '<br>')}</span>`

    return renderedHead + renderedTail
  }

  return parseMd(text)
}

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll('\'', '&#39;')
}

export function formatTime(date: Date) {
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

type ChatMessage = {
  id: number
  from: 'user' | 'support'
  text: string // RAW markdown
  html: string // parsed markdown
  streaming: boolean
  time: string
}

export type McpContext = ReturnType<typeof useMcp>
