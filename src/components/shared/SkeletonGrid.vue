<script setup lang="ts">
defineProps<{
  count?: number
}>()
</script>

<template>
  <div class="skeleton-wrap">
    <div class="skeleton skeleton--title"></div>
    <div class="skeleton-grid">
      <div v-for="i in (count ?? 6)" :key="i" class="skeleton skeleton--card"></div>
    </div>
  </div>
</template>
