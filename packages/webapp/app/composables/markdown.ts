const ADMONITION_LABELS: Record<string, string> = {
  info: 'Info',
  warning: 'Warning',
}

function escapeHtml(value: string): string {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll('\'', '&#39;')
}

function isAsciiLetter(char: string): boolean {
  return /^[a-z]$/i.test(char)
}

function isDigit(char: string): boolean {
  return char >= '0' && char <= '9'
}

function hasProtocol(value: string): boolean {
  const colonIndex = value.indexOf(':')

  if (colonIndex <= 0) {
    return false
  }

  for (let index = 0; index < colonIndex; index += 1) {
    if (!isAsciiLetter(value[index] ?? '')) {
      return false
    }
  }

  return true
}

function isEmailAddress(value: string): boolean {
  const parts = value.trim().split('@')

  if (parts.length !== 2 || !parts[0] || !parts[1]) {
    return false
  }

  return parts[1].includes('.')
}

function normalizeHref(href: string): string {
  const normalizedHref = href.trim()

  if (!normalizedHref) {
    return '#'
  }

  if (hasProtocol(normalizedHref) || normalizedHref.startsWith('/') || normalizedHref.startsWith('#')) {
    return normalizedHref
  }

  if (isEmailAddress(normalizedHref)) {
    return `mailto:${normalizedHref}`
  }

  if (normalizedHref.startsWith('www.')) {
    return `https://${normalizedHref}`
  }

  return normalizedHref
}

function renderAnchor(label: string, href: string): string {
  const normalizedHref = escapeHtml(normalizeHref(href))
  const isExternal = normalizedHref.startsWith('http://') || normalizedHref.startsWith('https://')
  const attrs = isExternal
    ? ' target="_blank" rel="noreferrer noopener"'
    : ''

  return `<a href="${normalizedHref}"${attrs}>${escapeHtml(label)}</a>`
}

function renderInline(markdown: string): string {
  const placeholders = new Map<string, string>()
  let placeholderIndex = 0

  const store = (html: string) => {
    const key = `__MD_${placeholderIndex++}__`
    placeholders.set(key, html)
    return key
  }

  let html = markdown
    .replaceAll(/`([^`]+)`/g, (_, code: string) => store(`<code>${escapeHtml(code)}</code>`))
    .replaceAll(/\[([^\]]+)\]\(([^)]+)\)/g, (_, label: string, href: string) => store(renderAnchor(label, href)))
    .replaceAll(/([\w.%+\-]+@[a-z0-9.\-]+\.[a-z]{2,})/gi, (_, email: string) => store(renderAnchor(email, email)))

  html = escapeHtml(html)
    .replaceAll(/\*\*([^*\n]+)\*\*/g, '<strong>$1</strong>')
    .replaceAll(/(^|[\s(>])\*([^*\n]+)\*(?=[$\s).,;:!?<])/g, '$1<em>$2</em>')
    .replaceAll(/(^|[^\w">])(https?:\/\/[^\s<]+)/g, (_, prefix: string, url: string) => {
      // Strip trailing punctuation that is sentence punctuation, not part of the URL
      let stripped = url
      let prev = ''
      while (stripped !== prev) {
        prev = stripped
        stripped = stripped.replace(/[.,;:!?)\]]+$/, '')
      }
      return `${prefix}${renderAnchor(stripped, stripped)}`
    })

  for (const [key, value] of placeholders) {
    html = html.replaceAll(key, value)
  }

  return html
}

function getListType(line: string): 'ol' | 'ul' | null {
  const trimmedLine = line.trimStart()

  if (!trimmedLine) {
    return null
  }

  if ('-*+'.includes(trimmedLine[0] ?? '') && trimmedLine[1] === ' ') {
    return 'ul'
  }

  let cursor = 0
  while (cursor < trimmedLine.length && isDigit(trimmedLine[cursor] ?? '')) {
    cursor += 1
  }

  if (cursor > 0 && trimmedLine[cursor] === '.' && trimmedLine[cursor + 1] === ' ') {
    return 'ol'
  }

  return null
}

function stripListMarker(line: string): string {
  const trimmedLine = line.trimStart()

  if (getListType(trimmedLine) === 'ul') {
    return trimmedLine.slice(2)
  }

  let cursor = 0
  while (cursor < trimmedLine.length && isDigit(trimmedLine[cursor] ?? '')) {
    cursor += 1
  }

  if (cursor > 0 && trimmedLine[cursor] === '.' && trimmedLine[cursor + 1] === ' ') {
    return trimmedLine.slice(cursor + 2)
  }

  return trimmedLine
}

function isListItem(line: string): boolean {
  return getListType(line) !== null
}

function isHorizontalRule(line: string): boolean {
  const trimmedLine = line.trim()

  if (trimmedLine.length < 3) {
    return false
  }

  for (const char of trimmedLine) {
    if (char !== '-') {
      return false
    }
  }

  return true
}

function isHeading(line: string): boolean {
  return /^#{1,6}\s/.test(line)
}

function isAdmonitionStart(line: string): boolean {
  return /^:::(info|warning)\b/.test(line)
}

function parseAdmonitionLine(line: string): { title: string, variant: 'info' | 'warning' } | null {
  for (const variant of ['info', 'warning'] as const) {
    const prefix = `:::${variant}`

    if (line.startsWith(prefix)) {
      return {
        variant,
        title: line.slice(prefix.length).trim(),
      }
    }
  }

  return null
}

function parseHeadingLine(line: string): { content: string, level: number } | null {
  let level = 0

  while (level < line.length && line[level] === '#') {
    level += 1
  }

  if (level < 1 || level > 6 || line[level] !== ' ') {
    return null
  }

  return {
    level,
    content: line.slice(level + 1),
  }
}

function isBlockBoundary(line: string): boolean {
  const trimmedLine = line.trim()

  return (
    !trimmedLine
    || isHeading(trimmedLine)
    || trimmedLine.startsWith('```')
    || isAdmonitionStart(trimmedLine)
    || trimmedLine === ':::'
    || isHorizontalRule(trimmedLine)
    || isListItem(trimmedLine)
  )
}

function renderList(lines: string[]): string {
  const tagName = getListType(lines[0] ?? '') === 'ol' ? 'ol' : 'ul'
  const items = lines
    .map(line => `<li>${renderInline(stripListMarker(line).trim())}</li>`)
    .join('')

  return `<${tagName}>${items}</${tagName}>`
}

function stripFrontmatter(lines: string[]): string[] {
  if (lines[0]?.trim() !== '---') {
    return lines
  }

  const endIndex = lines.findIndex((line, index) => index > 0 && line.trim() === '---')

  if (endIndex === -1) {
    return lines
  }

  return lines.slice(endIndex + 1)
}

function renderBlocks(lines: string[]): string {
  const blocks: string[] = []

  for (let index = 0; index < lines.length; index += 1) {
    const trimmedLine = lines[index]?.trim() ?? ''

    if (!trimmedLine) {
      continue
    }

    const admonition = parseAdmonitionLine(trimmedLine)
    if (admonition) {
      const { title, variant } = admonition
      const body: string[] = []
      let cursor = index + 1

      while (cursor < lines.length && lines[cursor]?.trim() !== ':::') {
        body.push(lines[cursor] ?? '')
        cursor += 1
      }

      const label = title.trim() || ADMONITION_LABELS[variant] || variant
      blocks.push(
        `<section class="md-note md-note--${variant}">`
        + `<div class="md-note__title">${escapeHtml(label)}</div>`
        + `<div class="md-note__body">${renderBlocks(body)}</div>`
        + '</section>',
      )

      index = cursor
      continue
    }

    const heading = parseHeadingLine(trimmedLine)
    if (heading) {
      blocks.push(`<h${heading.level}>${renderInline(heading.content.trim())}</h${heading.level}>`)
      continue
    }

    if (isHorizontalRule(trimmedLine)) {
      blocks.push('<hr>')
      continue
    }

    if (trimmedLine.startsWith('```')) {
      const codeLines: string[] = []
      let cursor = index + 1

      while (cursor < lines.length && !(lines[cursor]?.trim() ?? '').startsWith('```')) {
        codeLines.push(lines[cursor] ?? '')
        cursor += 1
      }

      blocks.push(`<pre><code>${escapeHtml(codeLines.join('\n'))}</code></pre>`)
      index = cursor
      continue
    }

    if (isListItem(trimmedLine)) {
      const listLines = [trimmedLine]
      let cursor = index + 1

      while (cursor < lines.length && isListItem(lines[cursor]?.trim() ?? '')) {
        listLines.push(lines[cursor]?.trim() ?? '')
        cursor += 1
      }

      blocks.push(renderList(listLines))
      index = cursor - 1
      continue
    }

    const paragraphLines = [trimmedLine]
    let cursor = index + 1

    while (cursor < lines.length && !isBlockBoundary(lines[cursor] ?? '')) {
      paragraphLines.push(lines[cursor]?.trim() ?? '')
      cursor += 1
    }

    blocks.push(`<p>${renderInline(paragraphLines.join(' '))}</p>`)
    index = cursor - 1
  }

  return blocks.join('')
}

export function parseMarkdownToHtml(markdown: string): string {
  const normalizedMarkdown = markdown.replaceAll('\r\n', '\n').trim()

  if (!normalizedMarkdown) {
    return ''
  }

  return renderBlocks(stripFrontmatter(normalizedMarkdown.split('\n')))
}
