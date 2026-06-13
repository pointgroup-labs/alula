import { MCP_SERVICE_LINK } from '../constants'

export async function fetchMCP(
  message: string,
  sessionId: string,
  args: Record<string, unknown>,
): Promise<string> {
  const res = await fetch(MCP_SERVICE_LINK, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      message,
      sessionId,
      ...args,
    }),
  })

  if (!res.ok) {
    throw new Error(`MCP request failed: ${res.status}`)
  }

  const data = await res.json()

  return data.response
}
