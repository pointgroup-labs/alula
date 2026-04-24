// bind own methods to instance
export function bindOwnMethods<T extends object>(
  instance: T,
  { enumerable = true }: { enumerable?: boolean } = {},
): T {
  const proto = Object.getPrototypeOf(instance)
  for (const key of Object.getOwnPropertyNames(proto)) {
    if (key === 'constructor') {
      continue
    }
    const desc = Object.getOwnPropertyDescriptor(proto, key)
    if (!desc || typeof desc.value !== 'function') {
      continue
    }
    Object.defineProperty(instance, key, {
      value: (desc.value as (...args: any[]) => any).bind(instance),
      enumerable,
      configurable: true,
      writable: false,
    })
  }
  return instance
}

// hide private properties
export function hidePrivate(obj: object, key: string) {
  Object.defineProperty(obj, key, { enumerable: false })
}
