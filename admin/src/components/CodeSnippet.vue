<template>
  <div class="code-snippet">
    <div class="code-snippet__bar">
      <div class="code-snippet__meta">
        <span class="code-snippet__title">{{ title }}</span>
        <span v-if="lang" class="code-snippet__lang">{{ lang }}</span>
      </div>
      <button type="button" class="code-snippet__copy" @click="onCopy">
        复制
      </button>
    </div>
    <pre class="code-snippet__body"><code>{{ code }}</code></pre>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  title: string
  code: string
  lang?: string
}>()

const emit = defineEmits<{
  copy: [code: string]
}>()

function onCopy() {
  emit('copy', props.code)
}
</script>

<style scoped>
.code-snippet {
  border: 1px solid rgba(15, 23, 42, 0.08);
  border-radius: 14px;
  overflow: hidden;
  background: #0b1220;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.code-snippet + .code-snippet {
  margin-top: 12px;
}

.code-snippet__bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 14px;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.07), rgba(255, 255, 255, 0.03));
  border-bottom: 1px solid rgba(148, 163, 184, 0.16);
}

.code-snippet__meta {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.code-snippet__title {
  font-size: 12px;
  font-weight: 700;
  color: #e2e8f0;
}

.code-snippet__lang {
  flex-shrink: 0;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(99, 102, 241, 0.22);
  color: #c7d2fe;
  font-size: 11px;
  font-weight: 600;
}

.code-snippet__copy {
  border: 0;
  border-radius: 999px;
  padding: 5px 12px;
  background: rgba(255, 255, 255, 0.08);
  color: #c7d2fe;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}

.code-snippet__copy:hover {
  background: rgba(99, 102, 241, 0.35);
  color: #fff;
}

.code-snippet__body {
  margin: 0;
  padding: 14px 16px 16px;
  overflow: auto;
  max-height: 360px;
  font-size: 12.5px;
  line-height: 1.6;
  color: #e2e8f0;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.code-snippet__body code {
  white-space: pre;
}
</style>
