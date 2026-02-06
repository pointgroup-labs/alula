export function withTimeoutAbort<T>(promise: Promise<T>, ms = 30_000): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error('Transaction timeout'))
    }, ms)

    promise
      .then(resolve)
      .catch(reject)
      .finally(() => clearTimeout(timer))
  })
}