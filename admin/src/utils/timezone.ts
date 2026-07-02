/** 常用时区，置顶展示 */
export const COMMON_TIME_ZONES = [
  'UTC',
  'Asia/Shanghai',
  'Asia/Hong_Kong',
  'Asia/Taipei',
  'Asia/Tokyo',
  'Asia/Seoul',
  'Asia/Singapore',
  'Europe/London',
  'Europe/Paris',
  'America/New_York',
  'America/Los_Angeles',
] as const

export function detectBrowserTimeZone(): string {
  try {
    return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
  } catch {
    return 'UTC'
  }
}

export function isValidTimeZone(timeZone: string): boolean {
  try {
    Intl.DateTimeFormat(undefined, { timeZone })
    return true
  } catch {
    return false
  }
}

export function listAllTimeZones(): string[] {
  try {
    const intl = Intl as typeof Intl & {
      supportedValuesOf?: (key: string) => string[]
    }
    if (typeof intl.supportedValuesOf === 'function') {
      return intl.supportedValuesOf('timeZone')
    }
  } catch {
    // ignore
  }
  return [...COMMON_TIME_ZONES]
}

export function buildTimeZoneOptions(): { label: string; value: string; group: string }[] {
  const all = listAllTimeZones()
  const common = new Set<string>(COMMON_TIME_ZONES)
  const options: { label: string; value: string; group: string }[] = []

  for (const tz of COMMON_TIME_ZONES) {
    if (all.includes(tz) || isValidTimeZone(tz)) {
      options.push({ value: tz, label: formatTimeZoneLabel(tz), group: '常用' })
    }
  }

  for (const tz of all) {
    if (common.has(tz)) continue
    options.push({ value: tz, label: formatTimeZoneLabel(tz), group: '全部' })
  }

  return options
}

export function formatTimeZoneLabel(timeZone: string): string {
  const offset = formatTimeZoneOffset(timeZone)
  const name = timeZone.replace(/_/g, ' ')
  return offset ? `${name} (${offset})` : name
}

export function formatTimeZoneOffset(timeZone: string, date = new Date()): string {
  try {
    const part = new Intl.DateTimeFormat('en-US', {
      timeZone,
      timeZoneName: 'longOffset',
    })
      .formatToParts(date)
      .find((p) => p.type === 'timeZoneName')
      ?.value
    if (!part) return ''
    return part.replace(/^GMT/, 'UTC')
  } catch {
    return ''
  }
}
