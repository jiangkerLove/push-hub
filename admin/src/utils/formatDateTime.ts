import { useTimezoneStore } from '@/stores/timezone'

const formatterCache = new Map<string, Intl.DateTimeFormat>()

function getDateTimeFormatter(timeZone: string): Intl.DateTimeFormat {
  let formatter = formatterCache.get(timeZone)
  if (!formatter) {
    formatter = new Intl.DateTimeFormat('zh-CN', {
      timeZone,
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: false,
    })
    formatterCache.set(timeZone, formatter)
  }
  return formatter
}

function part(parts: Intl.DateTimeFormatPart[], type: Intl.DateTimeFormatPartTypes): string {
  return parts.find((p) => p.type === type)?.value ?? ''
}

/** 将 ISO 8601 / Date 格式化为当前所选时区：yyyy-MM-dd HH:mm:ss */
export function formatDateTime(
  value: string | Date | null | undefined,
  timeZone?: string,
): string {
  if (value == null || value === '') return '—'
  const date = value instanceof Date ? value : new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)

  const zone = timeZone ?? useTimezoneStore().timeZone
  const parts = getDateTimeFormatter(zone).formatToParts(date)
  return `${part(parts, 'year')}-${part(parts, 'month')}-${part(parts, 'day')} ${part(parts, 'hour')}:${part(parts, 'minute')}:${part(parts, 'second')}`
}
