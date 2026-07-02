import { formatDateTime } from './formatDateTime'

export function addDays(date: Date, days: number): Date {
  const next = new Date(date)
  next.setDate(next.getDate() + days)
  return next
}

export function defaultCacheUntil(days = 7): Date {
  return addDays(new Date(), days)
}

export function daysFromCacheUntil(until: Date): number {
  const ms = until.getTime() - Date.now()
  return Math.max(1, Math.ceil(ms / 86_400_000))
}

export function cacheUntilFromDays(days: number): Date {
  return defaultCacheUntil(Math.max(1, days))
}

export const cacheUntilShortcuts = [
  { text: '1 天后', value: () => addDays(new Date(), 1) },
  { text: '3 天后', value: () => addDays(new Date(), 3) },
  { text: '7 天后', value: () => addDays(new Date(), 7) },
  { text: '14 天后', value: () => addDays(new Date(), 14) },
]

export function formatCacheUntil(date: Date): string {
  return date.toISOString()
}

export function formatCacheUntilLabel(date: Date): string {
  return formatDateTime(date)
}
