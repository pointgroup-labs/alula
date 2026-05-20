export function useAddTrustLine() {
  const clientStore = useClientStore()
  const alulaClient = computed(() => clientStore.alulaClient)

  return {
    alulaClient,
  }
}
