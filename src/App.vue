<script setup lang="ts">
import { computed } from 'vue'
import { darkTheme, lightTheme } from 'naive-ui'
import { currentTheme } from '@/composables/useTheme'
import TitleBar from '@/components/layout/TitleBar.vue'
import Sidebar from '@/components/layout/Sidebar.vue'
import MainContent from '@/components/layout/MainContent.vue'
import { toasts, removeToast, confirmState, closeConfirm } from '@/lib/utils'

const naiveTheme = computed(() => currentTheme.value === 'light' ? lightTheme : darkTheme)

const themeOverrides = {
  common: {
    primaryColor: '#00d4aa',
    primaryColorHover: '#00b894',
    primaryColorPressed: '#009688',
    borderRadius: '6px',
    borderRadiusSmall: '4px',
    fontFamily: '"Microsoft YaHei", "Microsoft YaHei UI", "PingFang SC", "Segoe UI", sans-serif',
  },
  Button: {
    borderRadiusMedium: '6px',
    colorPrimary: '#00d4aa',
    colorHoverPrimary: '#00b894',
    colorPressedPrimary: '#009688',
    textColorPrimary: '#1a1a1a',
    textColorHoverPrimary: '#1a1a1a',
    textColorPressedPrimary: '#1a1a1a',
  },
  Input: {
    borderRadius: '6px',
    borderHover: '#00d4aa',
    borderFocus: '#00d4aa',
    boxShadowFocus: '0 0 0 3px rgba(0, 212, 170, 0.15)',
  },
  Card: {
    borderRadius: '10px',
  },
  Tag: {
    borderRadius: '4px',
  },
  Menu: {
    borderRadius: '6px',
    itemTextColorActive: '#00d4aa',
    itemTextColorActiveHover: '#00d4aa',
    itemIconColorActive: '#00d4aa',
    itemIconColorActiveHover: '#00d4aa',
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
                      <button class="btn btn-primary" @click="confirmState.onConfirm?.(); closeConfirm()">确认</button>
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
