/** 从单段模板文本提取 {{变量}} 列表 */
export function extractTemplateVariables(text: string): string[] {
  const vars: string[] = []
  let rest = text
  while (true) {
    const start = rest.indexOf('{{')
    if (start < 0) break
    const inner = rest.slice(start + 2)
    const end = inner.indexOf('}}')
    if (end < 0) break
    const key = inner.slice(0, end).trim()
    if (key && !vars.includes(key)) vars.push(key)
    rest = inner.slice(end + 2)
  }
  return vars
}

export type TemplateSegment =
  | { type: 'text'; value: string }
  | { type: 'var'; value: string }

/** 将模板拆成普通文本与变量段，便于内联填空渲染 */
export function parseTemplateSegments(text: string): TemplateSegment[] {
  const segments: TemplateSegment[] = []
  let rest = text
  while (rest.length > 0) {
    const start = rest.indexOf('{{')
    if (start < 0) {
      if (rest) segments.push({ type: 'text', value: rest })
      break
    }
    if (start > 0) {
      segments.push({ type: 'text', value: rest.slice(0, start) })
    }
    const inner = rest.slice(start + 2)
    const end = inner.indexOf('}}')
    if (end < 0) {
      segments.push({ type: 'text', value: rest })
      break
    }
    const key = inner.slice(0, end).trim()
    if (key) segments.push({ type: 'var', value: key })
    rest = inner.slice(end + 2)
  }
  return segments
}

/** 将 {{变量}} 替换为填空预览 */
export function previewTemplateText(text: string, values: Record<string, string>): string {
  let output = text
  for (const [key, value] of Object.entries(values)) {
    if (value.trim()) {
      output = output.split(`{{${key}}}`).join(value.trim())
    }
  }
  return output
}
