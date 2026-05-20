<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { darkTheme, lightTheme } from 'naive-ui'
import { currentTheme, initAccent } from '@/composables/useTheme'
import { bgUrl, bgOpacity, setBackground } from '@/composables/useBackground'
import { api } from '@/lib/api'
import TitleBar from '@/components/layout/TitleBar.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import MainContent from '@/components/layout/MainContent.vue'
import { toasts, removeToast, confirmState, closeConfirm } from '@/lib/utils'
import { checkUpdate } from '@/composables/useUpdater'

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

onMounted(async () => {
  initAccent()
  try {
    const config = await api.getConfig()
    console.log('[App] config.background:', config.background)
    if (config.background?.path) {
      await setBackground(config.background.path, config.background.opacity)
    }
  } catch (e) { console.error('[App] getConfig failed:', e) }
  // 后台检查更新（静默，有新版也不弹窗，只设 updateInfo 触发角标）
  checkUpdate()
})
</script>

<template>
  <n-config-provider :theme="naiveTheme" :theme-overrides="themeOverrides" inline-theme-disabled>
    <n-loading-bar-provider>
      <n-dialog-provider>
        <n-notification-provider>
          <n-message-provider>
            <div class="app-layout" :class="{ 'has-bg': bgUrl }">
              <!-- Background layer -->
              <div v-if="bgUrl" class="bg-layer" :style="{ backgroundImage: `url(${bgUrl})` }" />
              <div v-if="bgUrl" class="bg-overlay" :style="{ opacity: bgOpacity }" />
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

<style scoped>
.bg-layer {
  position: fixed;
  inset: 0;
  z-index: -2;
  background-size: cover;
  background-position: center;
  pointer-events: none;
}

.bg-overlay {
  position: fixed;
  inset: 0;
  z-index: -1;
  background: var(--bg-void);
  pointer-events: none;
}
</style>
