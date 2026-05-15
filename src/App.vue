<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { darkTheme, lightTheme } from 'naive-ui'
import { currentTheme } from '@/composables/useTheme'
import TitleBar from '@/components/layout/TitleBar.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import MainContent from '@/components/layout/MainContent.vue'
import { toasts, removeToast, confirmState, closeConfirm } from '@/lib/utils'
import { dirty, markClean } from '@/composables/useAppState'

onMounted(async () => {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    const { ask } = await import('@tauri-apps/plugin-dialog')
    const win = getCurrentWindow()
    win.onCloseRequested(async (event) => {
      if (dirty.value) {
        event.preventDefault()
        try {
          const confirmed = await ask('有未保存的更改，确定要关闭吗？', { title: '未保存的更改', kind: 'warning' })
          if (confirmed) {
            markClean()
            await win.close()
          }
        } catch {
          // Dialog failed — allow close anyway
          markClean()
          await win.close()
        }
      }
    })
  } catch {
    // Not in Tauri — use browser beforeunload as fallback
    window.addEventListener('beforeunload', (e) => {
      if (dirty.value) { e.preventDefault() }
    })
  }
})

const naiveTheme = computed(() => currentTheme.value === 'light' ? lightTheme : darkTheme)

const themeOverrides = {
  common: {
    primaryColor: '#4a9fd8',
    primaryColorHover: '#5db8e8',
    primaryColorPressed: '#3a80b0',
    primaryColorSuppl: '#5db8e8',
    borderRadius: '10px',
    borderRadiusSmall: '6px',
    fontFamily: 'Consolas, "Microsoft YaHei", "Microsoft YaHei UI", "PingFang SC", sans-serif',
  },
  Button: {
    borderRadiusMedium: '10px',
    colorPrimary: '#4a9fd8',
    colorHoverPrimary: '#5db8e8',
    colorPressedPrimary: '#3a80b0',
    textColorPrimary: '#ffffff',
    textColorHoverPrimary: '#ffffff',
    textColorPressedPrimary: '#ffffff',
  },
  Input: {
    borderRadius: '6px',
    borderHover: '#4a9fd8',
    borderFocus: '#4a9fd8',
    boxShadowFocus: '0 0 0 3px rgba(74, 159, 216, 0.15)',
  },
  Card: {
    borderRadius: '10px',
  },
  Tag: {
    borderRadius: '4px',
  },
  Menu: {
    borderRadius: '6px',
    itemTextColorActive: '#4a9fd8',
    itemTextColorActiveHover: '#4a9fd8',
    itemIconColorActive: '#4a9fd8',
    itemIconColorActiveHover: '#4a9fd8',
  },
}
</script>

<template>
  <n-config-provider :theme="naiveTheme" :theme-overrides="themeOverrides" inline-theme-disabled>
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-message-provider>
            <div class="app-layout">
              <TitleBar />
              <div class="app-body">
                <Sidebar />
                <MainContent />
              </div>

              <!-- Toast Container -->
              <div id="toast-container" aria-live="polite" aria-atomic="true">
                <TransitionGroup name="toast">
                  <div
                    v-for="t in toasts"
                    :key="t.id"
                    class="toast"
                    :class="'toast--' + t.type"
                    role="alert"
                  >
                    <span>{{ t.message }}</span>
                    <button class="toast-close" @click="removeToast(t.id)">×</button>
                  </div>
                </TransitionGroup>
              </div>

              <!-- Confirm Dialog -->
              <Transition name="confirm">
                <div v-if="confirmState.visible" class="confirm-overlay">
                  <div class="confirm-dialog">
                    <p>{{ confirmState.message }}</p>
                    <div class="confirm-actions">
                      <button class="btn btn-ghost" @click="closeConfirm">取消</button>
                      <button class="btn btn-primary" @click="async () => { await confirmState.onConfirm?.(); closeConfirm() }">确认</button>
                    </div>
                  </div>
                </div>
              </Transition>
            </div>
          </n-message-provider>
        </n-notification-provider>
      </n-dialog-provider>
    </n-loading-bar-provider>
  </n-config-provider>
</template>
