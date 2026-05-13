<script setup lang="ts">
const props = defineProps<{
  skills: { type: string; name: string; level: number; max: number }[]
}>()

const emit = defineEmits<{
  (e: 'update', type: string, level: number): void
}>()

function onInput(type: string, e: Event) {
  let v = parseInt((e.target as HTMLInputElement).value) || 0
  const skill = props.skills.find(s => s.type === type)
  if (skill) {
    if (v < 0) v = 0
    if (v > skill.max) v = skill.max
  }
  emit('update', type, v)
}
</script>

<template>
  <div class="skill-grid">
    <div v-for="skill in skills" :key="skill.type" class="skill-card">
      <div class="skill-name">{{ skill.name }}</div>
      <input
        type="number"
        class="skill-input"
        :value="skill.level"
        :min="0"
        :max="skill.max"
        @change="onInput(skill.type, $event)"
      />
      <div class="skill-hint text-xs text-muted">最大 {{ skill.max }}</div>
    </div>
  </div>
</template>
