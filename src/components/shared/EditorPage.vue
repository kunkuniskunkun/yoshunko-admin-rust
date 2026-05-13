<script setup lang="ts">
defineProps<{
  title: string
  subtitle?: string
}>()

const emit = defineEmits<{
  (e: 'back'): void
  (e: 'save'): void
}>()
</script>

<template>
  <div class="editor-page">
    <div class="editor-page__top">
      <a class="editor-back" href="#" @click.prevent="emit('back')">← 返回</a>
      <div class="editor-page__header">
        <h2 class="editor-title">{{ title }}</h2>
        <span v-if="subtitle" class="editor-subtitle text-muted">{{ subtitle }}</span>
      </div>
    </div>
    <div class="editor-page__body">
      <slot />
    </div>
    <div class="editor-page__actions">
      <button class="btn btn-primary" @click="emit('save')">保存</button>
    </div>
  </div>
</template>
