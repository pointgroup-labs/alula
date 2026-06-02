export type Fetcher<T> = () => Promise<T>

class CacheManager {
  private store = new Map<string, any>()
  private inflight = new Map<string, Promise<any>>()

  key(...parts: Array<string | number | null | undefined>): string {
    return parts.filter(Boolean).join(':')
  }

  get<T>(key: string): T | undefined {
    return this.store.get(key)
  }

  set<T>(key: string, value: T): void {
    this.store.set(key, value)
  }

  invalidate(key: string): void {
    this.store.delete(key)
    this.inflight.delete(key)
  }

  async getOrSet<T>(key: string, fetcher: Fetcher<T>): Promise<T> {
    const cached = this.get<T>(key)
    if (cached !== undefined) {
      return cached
    }

    const running = this.inflight.get(key) as Promise<T> | undefined
    if (running) {
      return running
    }

    const p = (async () => {
      try {
        const v = await fetcher()
        this.set<T>(key, v)
        return v
      } finally {
        this.inflight.delete(key)
      }
    })()

    this.inflight.set(key, p)
    return p
  }
}

export const cacheManager = new CacheManager()
