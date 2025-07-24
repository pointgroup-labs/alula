export function focusInput(className: string) {
  const input = document.querySelector(className)?.querySelector('input') as HTMLInputElement
  input?.focus()
}

export function clickElement(className: string) {
  const element = document.querySelector(className) as HTMLInputElement
  element?.click()
}
