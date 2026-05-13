<script setup lang="ts">
const props = defineProps<{
  modelValue: number
  min: number
  max: number
  label?: string
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', v: number): void
}>()

function step(delta: number) {
  const next = props.modelValue + delta
  if (next >= props.min && next <= props.max) {
    emit('update:modelValue', next)
  }
}

function onInput(e: Event) {
  let v = parseInt((e.target as HTMLInputElement).value) || 0
  if (v < props.min) v = props.min
  if (v > props.max) v = props.max
  emit('update:modelValue', v)
}
</script>

<template>
  <div class="input-stepper">
    <button class="stepper-btn" :disabled="modelValue <= min" @click="step(-1)" :aria-label="'减少' + (label || '')">−</button>
    <input type="number" class="stepper-input" :value="modelValue" :min="min" :max="max" @change="onInput" />
    <button class="stepper-btn" :disabled="modelValue >= max" @click="step(1)" :aria-label="'增加' + (label || '')">+</button>
  </div>
</template>
