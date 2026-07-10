import type { Ref } from 'vue'

export type LatestRequest<T> = {
  loading: Ref<boolean>
  error: Ref<string | undefined>
  // Runs `fn`. If a newer call lands first, the older call's success/error is dropped.
  // `onSuccess` / `onError` are invoked only for the still-current request.
  run: (
    fn: () => Promise<T>,
    onSuccess: (value: T) => void,
    onError?: (error: unknown) => void,
  ) => Promise<void>
  // Increments the internal request id so any in-flight call becomes stale.
  cancel: () => void
}

export function useLatestRequest<T>(): LatestRequest<T> {
  let requestId = 0
  const loading = ref(false)
  const error = ref<string>()

  async function run(
    fn: () => Promise<T>,
    onSuccess: (value: T) => void,
    onError?: (e: unknown) => void,
  ) {
    const id = ++requestId
    loading.value = true
    error.value = undefined

    try {
      const result = await fn()
      if (id !== requestId) {
        return
      }
      onSuccess(result)
    } catch (error_: any) {
      if (id !== requestId) {
        return
      }
      error.value = String(error_?.message || error_)
      onError?.(error_)
    } finally {
      if (id === requestId) {
        loading.value = false
      }
    }
  }

  function cancel() {
    requestId++
  }

  return { loading, error, run, cancel }
}
