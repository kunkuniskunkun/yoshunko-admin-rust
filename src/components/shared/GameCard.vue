<script setup lang="ts">
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

function onPress(e: Event) {
  const target = e.currentTarget as HTMLElement
  target.style.transition = 'transform 0.35s cubic-bezier(0.34, 1.56, 0.64, 1)'
  target.style.transform = 'scale(0.92)'
  setTimeout(() => { target.style.transform = 'scale(1)' }, 120)
}
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
