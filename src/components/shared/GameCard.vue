<script setup lang="ts">
import { onUnmounted } from 'vue'

defineProps<{
  rarity?: 's' | 'a' | 'b'
  title: string
  subtitle?: string
  tags?: string[]
  level?: number
}>()

const emit = defineEmits<{
  (e: 'click'): void
}>()

let pressTimer: ReturnType<typeof setTimeout> | null = null

function onPress(e: Event) {
  const target = e.currentTarget as HTMLElement
  target.style.transition = 'transform 0.15s cubic-bezier(0.23, 1, 0.32, 1)'
  target.style.transform = 'translate(4px, -4px) scale(0.96)'
  if (pressTimer) clearTimeout(pressTimer)
  pressTimer = setTimeout(() => {
    target.style.transform = ''
    target.style.transition = ''
  }, 150)
}

onUnmounted(() => { if (pressTimer) clearTimeout(pressTimer) })
</script>

<template>
  <div
    class="game-card"
    :class="{
      'rarity-s-card': rarity === 's',
      'rarity-a-card': rarity === 'a',
    }"
    tabindex="0"
    role="button"
    @click="onPress($event); emit('click')"
    @keydown.enter="emit('click')"
    @keydown.space.prevent="emit('click')"
  >
    <div class="card-header">
      <span v-if="rarity" class="game-card__rarity" :class="'rarity-' + rarity">{{ rarity.toUpperCase() }}</span>
      <span v-if="level !== undefined" class="game-card__level">Lv.{{ level }}</span>
    </div>
    <div class="card-title">{{ title }}</div>
    <div v-if="subtitle" class="card-subtitle text-xs text-muted">{{ subtitle }}</div>
    <div v-if="tags && tags.length" class="card-tags">
      <span v-for="tag in tags" :key="tag" class="card-tag">{{ tag }}</span>
    </div>
  </div>
</template>
