<script setup lang="ts">
import { ref } from 'vue'

const command = 'curl -fsSL https://dantescur.github.io/iguard/install.sh | bash'
const copied = ref(false)
let resetTimer: ReturnType<typeof setTimeout> | undefined

async function copy() {
  try {
    await navigator.clipboard.writeText(command)
    copied.value = true
    if (resetTimer) clearTimeout(resetTimer)
    resetTimer = setTimeout(() => {
      copied.value = false
    }, 1600)
  } catch {
    copied.value = false
  }
}
</script>

<template>
  <div class="iguard-command">
    <span class="iguard-command__prompt" aria-hidden="true">$</span>
    <code class="iguard-command__code">{{ command }}</code>
    <button
      class="iguard-command__copy"
      type="button"
      :aria-label="copied ? 'Copied' : 'Copy install command'"
      @click="copy"
    >
      {{ copied ? 'Copied' : 'Copy' }}
    </button>
  </div>
</template>

<style scoped>
.iguard-command {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  max-width: 560px;
  margin-top: 22px;
  padding: 10px 10px 10px 14px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  background: var(--vp-c-bg-soft);
}

.iguard-command__prompt {
  color: var(--vp-c-brand-1);
  font-family: var(--iguard-font-mono);
}

.iguard-command__code {
  flex: 1;
  min-width: 0;
  overflow-x: auto;
  color: var(--vp-c-text-2);
  font-family: var(--iguard-font-mono);
  font-size: 0.78rem;
  scrollbar-width: none;
  white-space: nowrap;
}

.iguard-command__code::-webkit-scrollbar {
  display: none;
}

.iguard-command__copy {
  flex-shrink: 0;
  padding: 5px 9px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 5px;
  background: var(--vp-c-bg);
  color: var(--vp-c-text-2);
  cursor: pointer;
  font-family: var(--iguard-font-mono);
  font-size: 0.68rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  transition:
    border-color 160ms ease,
    color 160ms ease;
}

.iguard-command__copy:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.iguard-command__copy:focus-visible {
  outline: 2px solid var(--vp-c-brand-1);
  outline-offset: 2px;
}
</style>
