export function formatDateToDMY(date: Date): string {
  const formatted = new Intl.DateTimeFormat('en-GB', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  }).format(date).replaceAll('/', '.')
  return formatted
}
