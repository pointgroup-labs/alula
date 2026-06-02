import type { Axios } from 'axios'
import axios from 'axios'

export * from './api'

export function createApiClient(baseURL: string): Axios {
  const client = axios.create({ baseURL })

  client.interceptors.request.use((config) => {
    try {
      config.params = { ...config.params }
    } catch {}
    return config
  })

  return client
}
