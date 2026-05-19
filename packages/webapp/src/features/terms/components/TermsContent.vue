<script lang="ts" setup>
import { getDocumentProxy } from 'unpdf'

const buffer = await fetch('/files/Alula_Finance_Terms_of_Service.pdf')
  .then(r => r.arrayBuffer())

const pdf = await getDocumentProxy(new Uint8Array(buffer))

function escapeHtml(value: string) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
}

function formatInline(text: string) {
  return escapeHtml(text)
    .replaceAll(
      /(https?:\/\/\S+)/g,
      '<a href="$1" target="_blank">$1</a>',
    )
    .replaceAll(
      /([\w.-]+@[\w.-]+\.[a-z]+)/gi,
      '<a href="mailto:$1">$1</a>',
    )
}

function isMainSection(line: string) {
  return /^\d+\.\s/.test(line)
}

function isSubSection(line: string) {
  return /^\d+\.\d+\./.test(line)
}

function isUpperList(line: string) {
  return /^\([A-Z]\)/.test(line)
}

function isLowerList(line: string) {
  return /^\([a-z]\)/.test(line)
}

function isRomanList(line: string) {
  return /^\([ivx]+\)/i.test(line)
}

const lines: string[] = []

for (let pageNum = 1; pageNum <= pdf.numPages; pageNum++) {
  const page = await pdf.getPage(pageNum)

  const content = await page.getTextContent()

  let lastY: number | null = null
  let currentLine = ''

  for (const item of content.items as any[]) {
    if (!item.str?.trim()) {
      continue
    }

    const y = item.transform[5]

    // new line
    if (lastY !== null && Math.abs(lastY - y) > 4) {
      lines.push(currentLine.trim())
      currentLine = ''
    }

    currentLine += `${item.str} `

    lastY = y
  }

  if (currentLine.trim()) {
    lines.push(currentLine.trim())
  }
}

const cleanedLines = lines
  .map(v => v.trim())
  .filter(Boolean)
  .filter(v => !/^Page \d+ of \d+$/i.test(v))

let html = '<div class="terms-content">'

let sectionOpen = false
let subsectionOpen = false

let listOpen = false

function closeList() {
  if (listOpen) {
    html += '</ol>'
    listOpen = false
  }
}

function closeSubsection() {
  closeList()

  if (subsectionOpen) {
    html += '</div>'
    subsectionOpen = false
  }
}

function closeSection() {
  closeSubsection()

  if (sectionOpen) {
    html += '</section>'
    sectionOpen = false
  }
}

for (const line of cleanedLines) {
  // title
  if (line === 'TERMS OF SERVICE') {
    html += `
      <div class="terms-header">
        <h1>${line}</h1>
    `
    continue
  }

  // company
  if (line === 'Alula Finance Corp') {
    html += `<p>${line}</p>`
    continue
  }

  // updated
  if (line.startsWith('Last updated:')) {
    html += `<p>${line}</p></div>`
    continue
  }

  // main section
  if (isMainSection(line)) {
    closeSection()

    sectionOpen = true

    html += `
      <section>
        <h2>${formatInline(line)}</h2>
    `

    continue
  }

  // subsection
  if (isSubSection(line)) {
    closeSubsection()

    subsectionOpen = true

    html += `
      <div>
        <h3>${formatInline(line)}</h3>
    `

    continue
  }

  // lists
  if (
    isUpperList(line)
    || isLowerList(line)
    || isRomanList(line)
  ) {
    if (!listOpen) {
      html += '<ol>'
      listOpen = true
    }

    html += `
      <li>
        ${formatInline(
          line.replace(/^\([^)]+\)\s*/, ''),
        )}
      </li>
    `

    continue
  }

  closeList()

  html += `
    <p>
      ${formatInline(line)}
    </p>
  `
}

closeSection()

html += '</div>'
</script>

<template>
  <div v-html="html" />
</template>

<style lang="scss">
@import '../assets/legal.scss';
</style>
