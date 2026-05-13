<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()

function minimize() { appWindow.minimize() }
function toggleMax() { appWindow.toggleMaximize() }
function close() { appWindow.close() }
</script>

<template>
  <div data-tauri-drag-region class="title-bar">
    <div class="title-bar__title" data-tauri-drag-region>
      <span class="title-bar__brand">Yoshunko Admin</span>
    </div>
    <div class="title-bar__controls">
      <button class="tb-btn" aria-label="最小化" @click="minimize">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect y="5" width="12" height="2" fill="currentColor"/></svg>
      </button>
      <button class="tb-btn" aria-label="最大化" @click="toggleMax">
        <svg width="12" height="12" viewBox="0 0 12 12"><rect x="1" y="1" width="10" height="10" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
      </button>
      <button class="tb-btn tb-btn--close" aria-label="关闭" @click="close">
        <svg width="12" height="12" viewBox="0 0 12 12"><path d="M1 1L11 11M11 1L1 11" stroke="currentColor" stroke-width="1.5"/></svg>
      </button>
    </div>
  </div>
</template>
