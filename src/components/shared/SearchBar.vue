<script setup lang="ts">
import { ref } from 'vue'

defineProps<{
  modelValue: string
  placeholder?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: string): void
}>()

const isComposing = ref(false)

function onInput(e: Event) {
  if (isComposing.value) return
  emit('update:modelValue', (e.target as HTMLInputElement).value)
}

function onCompositionStart() { isComposing.value = true }
function onCompositionEnd(e: CompositionEvent) {
  isComposing.value = false
  emit('update:modelValue', (e.target as HTMLInputElement).value)
}

function clear() {
  emit('update:modelValue', '')
}
</script>

<template>
  <div class="search-wrap">
    <input
      type="text"
      class="search-input"
      :value="modelValue"
      :placeholder="placeholder || '搜索...'"
      @input="onInput"
      @compositionstart="onCompositionStart"
      @compositionend="onCompositionEnd"
    />
    <button v-if="modelValue" class="search-clear" aria-label="清除搜索" @click="clear">×</button>
  </div>
</template>
