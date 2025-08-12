export function useExplorerLink() {
  const rpcStore = useRpcStore()

  const network = computed(() => rpcStore.network)

  function generateExplorerLink(hash: string, entityId = 'tx'): string {
    return `https://stellar.expert/explorer/${network.value}/${entityId}/${hash}`
  }

  return {
    generateExplorerLink,
  }
}
