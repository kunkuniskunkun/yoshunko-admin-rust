<script setup lang="ts">
import { uid, panel, avatarCache, weaponCache, equipCache, dirty, configured } from '@/composables/useAppState'
import { toggleTheme, currentTheme } from '@/composables/useTheme'
import { api } from '@/lib/api'
import { ref, onMounted, watch } from 'vue'
import { Users, CircleDot, Hexagon, Triangle, User, Rocket, Settings, Sun, Moon } from 'lucide-vue-next'

const players = ref<number[]>([])
const avatarCount = ref(0)
const weaponCount = ref(0)
const equipCount = ref(0)

const navItems = [
  { key: 'avatars', label: '角色管理', icon: Users, countKey: 'avatar' as const },
  { key: 'weapons', label: '音擎仓库', icon: CircleDot, countKey: 'weapon' as const },
  { key: 'equips', label: '驱动盘仓库', icon: Hexagon, countKey: 'equip' as const },
  { key: 'hadal_zone', label: '式舆防卫战', icon: Triangle, countKey: null },
  { key: 'player_info', label: '玩家信息', icon: User, countKey: null },
  { key: 'quick_launch', label: '快速启动', icon: Rocket, countKey: null },
]

function selectPanel(key: string) {
  panel.value = key
}

function onPlayerChange(e: Event) {
  const val = parseInt((e.target as HTMLSelectElement).value)
  uid.value = isNaN(val) ? null : val
}

function updateCounts() {
  avatarCount.value = avatarCache.value.filter(a => a.avatar_id !== 2071 && a.avatar_id !== 2121).length
  weaponCount.value = weaponCache.value.filter(w => w.id < 12000 || w.id > 12999).length
  equipCount.value = equipCache.value.length
}

async function loadPlayers() {
  try {
    const data = await api.getPlayerList()
    if (data && Array.isArray(data.players)) {
      players.value = data.players
      if (data.players.length > 0 && !uid.value) {
        uid.value = data.players[0]
      }
    }
  } catch (e) {
    console.error('Failed to load players:', e)
  }
}

onMounted(loadPlayers)

watch(configured, (val) => {
  if (val) loadPlayers()
})

defineExpose({ updateCounts, loadPlayers })
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-brand">
      <img src="@/assets/icon.png" alt="Logo" class="sidebar-brand__logo" />
      <div>
        <h1 class="sidebar-brand__title">Yoshunko Admin</h1>
        <p class="sidebar-brand__sub">Game Data Manager</p>
      </div>
    </div>

    <div class="sidebar-player">
      <select class="form-select" :value="uid ?? ''" @change="onPlayerChange">
        <option value="">-- 选择玩家 --</option>
        <option v-for="pid in players" :key="pid" :value="pid">玩家 UID: {{ pid }}</option>
      </select>
    </div>

    <nav class="sidebar-nav" role="navigation" aria-label="功能导航">
      <div role="tablist" aria-label="功能面板">
        <div
          v-for="item in navItems"
          :key="item.key"
          class="nav-item"
          :class="{ active: panel === item.key, dirty: dirty && panel === item.key }"
          role="tab"
          :aria-selected="panel === item.key"
          @click="selectPanel(item.key)"
        >
          <component :is="item.icon" class="nav-icon" :size="18" />
          <span class="nav-item__label">{{ item.label }}</span>
          <span v-if="item.countKey === 'avatar'" class="nav-badge" id="avatar-count">{{ avatarCount }}</span>
          <span v-if="item.countKey === 'weapon'" class="nav-badge" id="weapon-count">{{ weaponCount }}</span>
          <span v-if="item.countKey === 'equip'" class="nav-badge" id="equip-count">{{ equipCount }}</span>
        </div>
      </div>
    </nav>

    <div class="sidebar-footer">
      <button class="sidebar-settings-btn" aria-label="设置" @click="selectPanel('settings')">
        <Settings :size="18" />
      </button>
      <button class="theme-toggle--sidebar" aria-label="切换主题" @click="toggleTheme">
        <Sun v-if="currentTheme === 'dark'" class="theme-icon-sun" :size="18" />
        <Moon v-else class="theme-icon-moon" :size="18" />
      </button>
    </div>
  </aside>
</template>
