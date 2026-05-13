<script setup lang="ts">
defineProps<{
  rank: number
  max?: number
}>()
</script>

<template>
  <div class="rank-dots">
    <span v-for="i in (max || 6)" :key="i" class="rank-dot" :class="{ active: i <= rank }"></span>
  </div>
</template>
