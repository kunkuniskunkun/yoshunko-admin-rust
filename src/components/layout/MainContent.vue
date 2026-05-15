<script setup lang="ts">
import { panel, uid, templates, cacheDirty, avatarCache, weaponCache, equipCache, configured } from '@/composables/useAppState'
import { api } from '@/lib/api'
import { ref, onMounted, watch, defineAsyncComponent, nextTick } from 'vue'
import { initTheme } from '@/composables/useTheme'

import SetupPanel from '@/components/panels/SetupPanel.vue'
import SkeletonGrid from '@/components/shared/SkeletonGrid.vue'

const AvatarsPanel = defineAsyncComponent(() => import('@/components/panels/AvatarsPanel.vue'))
const WeaponsPanel = defineAsyncComponent(() => import('@/components/panels/WeaponsPanel.vue'))
const EquipsPanel = defineAsyncComponent(() => import('@/components/panels/EquipsPanel.vue'))
const HadalPanel = defineAsyncComponent(() => import('@/components/panels/HadalPanel.vue'))
const PlayerPanel = defineAsyncComponent(() => import('@/components/panels/PlayerPanel.vue'))
const QuickLaunchPanel = defineAsyncComponent(() => import('@/components/panels/QuickLaunchPanel.vue'))
const SettingsPanel = defineAsyncComponent(() => import('@/components/panels/SettingsPanel.vue'))
const ShortcutsPanel = defineAsyncComponent(() => import('@/components/panels/ShortcutsPanel.vue'))

const loading = ref(true)
const mainRef = ref<HTMLElement | null>(null)

function applySlideIn() {
  nextTick(() => {
    if (mainRef.value) {
      mainRef.value.classList.remove('editor-slide-in')
      void mainRef.value.offsetHeight
      mainRef.value.classList.add('editor-slide-in')
    }
  })
}

async function checkConfig() {
  try {
    const cfg = await api.getConfig()
    if (cfg.configured && cfg.config_exists) {
      templates.value = await api.getTemplates()
      configured.value = true
    }
  } catch (e) {
    console.error('Config check failed:', e)
  }
  loading.value = false
}

async function loadCounts() {
  if (!uid.value) return
  try {
    const [av, wp, eq] = await Promise.all([
      api.getAvatars(uid.value),
      api.getWeapons(uid.value),
      api.getEquips(uid.value),
    ])
    avatarCache.value = av.avatars
    weaponCache.value = wp.weapons
    equipCache.value = eq.equips
    cacheDirty.value = false
  } catch (e) {
    console.error('Failed to load counts:', e)
  }
}

onMounted(() => {
  initTheme()
  checkConfig()
})

watch(uid, async () => {
  if (uid.value && configured.value) {
    // Clear caches before fetching to prevent stale data display
    avatarCache.value = []
    weaponCache.value = []
    equipCache.value = []
    cacheDirty.value = true
    await loadCounts()
    applySlideIn()
  }
})

watch(panel, () => {
  applySlideIn()
})

function onConnected() {
  checkConfig()
}
</script>

<template>
  <main ref="mainRef" class="main-content" role="main" aria-label="主内容区">
    <div v-if="loading" class="loading-wrap"><div class="spinner"></div></div>
    <SetupPanel v-else-if="!configured" @connected="onConnected" />
    <div v-else-if="!uid" class="empty-state">
      <div class="empty-state__icon"></div>
      <p>选择一个玩家开始管理游戏数据</p>
    </div>
    <KeepAlive v-else>
      <component :is="{
        avatars: AvatarsPanel,
        weapons: WeaponsPanel,
        equips: EquipsPanel,
        hadal_zone: HadalPanel,
        player_info: PlayerPanel,
        quick_launch: QuickLaunchPanel,
        settings: SettingsPanel,
        shortcuts: ShortcutsPanel,
      }[panel] || AvatarsPanel" />
    </KeepAlive>
  </main>
</template>
