<script setup lang="ts">
defineProps<{
  star: number
  max?: number
}>()
</script>

<template>
  <div class="star-rating">
    <span v-for="i in (max ?? 5)" :key="i" class="star" :class="{ active: i <= star }">★</span>
  </div>
</template>
