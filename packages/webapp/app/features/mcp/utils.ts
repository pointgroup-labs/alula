import MarkdownIt from 'markdown-it'

const SOLANA_ADDRESS_REGEX = /\b[1-9A-HJ-NP-Za-km-z]{32,44}\b/g

export function updateMessageVoteId(
  text: string,
  newVoteId?: string,
): string {
  // удалить все voteId из текста
  let cleaned = text
    .replaceAll(SOLANA_ADDRESS_REGEX, '')
    .replaceAll(/\s+/g, ' ')
    .trim()

  if (newVoteId) {
    cleaned = cleaned.length > 0
      ? `${cleaned} ${newVoteId}`
      : newVoteId
  }

  return cleaned
}

const md = new MarkdownIt({
  html: false,
  linkify: true,
  breaks: true,
})

export function typeText(
  text: string,
  onUpdate: (chunk: string) => void,
  speed = 14,
) {
  return new Promise<void>((resolve) => {
    let i = 0

    function tick() {
      i++
      onUpdate(text.slice(0, i))

      if (i < text.length) {
        setTimeout(tick, speed)
      } else {
        resolve()
      }
    }

    tick()
  })
}

// add target="_blank"
const defaultRender
  = md.renderer.rules.link_open
    || function (tokens, idx, options, env, self) {
      return self.renderToken(tokens, idx, options)
    }

md.renderer.rules.link_open = function (tokens, idx, options, env, self) {
  const token = tokens[idx]

  const hrefIndex = token.attrIndex('href')

  if (hrefIndex >= 0) {
    let href = token.attrs![hrefIndex][1]

    // ✅ 1. replace ../ links
    if (href.startsWith('../')) {
      href = `https://docs.jpool.one/${href.replace(/^\.\.\//, '')}`
      token.attrs![hrefIndex][1] = href
    }

    const isInternalAppLink = href.includes('app.jpool.one')

    // ✅ 2. external links → open in new tab
    if (!isInternalAppLink) {
      const targetIndex = token.attrIndex('target')

      if (targetIndex < 0) {
        token.attrPush(['target', '_blank'])
      } else {
        token.attrs![targetIndex][1] = '_blank'
      }

      // rel (avoid duplicates)
      if (token.attrIndex('rel') < 0) {
        token.attrPush(['rel', 'noopener noreferrer'])
      }
    }
  }

  return defaultRender(tokens, idx, options, env, self)
}

export function parseMd(text: string) {
  return md.render(text)
}
