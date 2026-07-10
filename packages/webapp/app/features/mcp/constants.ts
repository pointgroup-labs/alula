const DEV_API_URL = 'http://localhost:8787'
const PROD_API_URL = 'https://ai.alula.finance'

export const MCP_SERVICE_LINK = import.meta.env.DEV ? DEV_API_URL : PROD_API_URL
