<script setup lang="ts">
import { uid, panel, avatarCache, weaponCache, equipCache, dirty, configured } from '@/composables/useAppState'
import { toggleTheme, currentTheme } from '@/composables/useTheme'
import { api } from '@/lib/api'
import { ref, onMounted, watch } from 'vue'
import { Users, CircleDot, Hexagon, Triangle, User, Rocket, Settings, Sun, Moon } from 'lucide-vue-next'
import { EXCLUDED_AVATAR_IDS, NPC_WEAPON_ID_MIN, NPC_WEAPON_ID_MAX } from '@/constants'

const players = ref<number[]>([])
const avatarCount = ref(0)
const weaponCount = ref(0)
const equipCount = ref(0)

const navItems = [
  { key: 'avatars', label: '角色管理', icon: Users, countKey: 'avatar' as const },
  { key: 'weapons', label: '音擎仓库', icon: CircleDot, countKey: 'weapon' as const },
  { key: 'equips', label: '驱动盘仓库', icon: Hexagon, countKey: 'equip' as const },
  { key: 'hadal_zone', label: '防卫战·危局', icon: Triangle, countKey: null },
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
  avatarCount.value = avatarCache.value.filter(a => !EXCLUDED_AVATAR_IDS.includes(a.avatar_id)).length
  weaponCount.value = weaponCache.value.filter(w => w.id < NPC_WEAPON_ID_MIN || w.id > NPC_WEAPON_ID_MAX).length
  equipCount.value = equipCache.value.length
}

async function loadPlayers() {
  try {
    const data = await api.getPlayerList()
    if (data && Array.isArray(data.players)) {
      players.value = data.players
      if (data.players.length > 0 && uid.value === null) {
        uid.value = data.players[0]
      }
    }
    updateCounts()
  } catch (e) {
    console.error('Failed to load players:', e)
  }
}

onMounted(loadPlayers)

watch(configured, (val) => {
  if (val) loadPlayers()
})

// 自动监听缓存变化，更新导航栏计数
watch(avatarCache, () => updateCounts())
watch(weaponCache, () => updateCounts())
watch(equipCache, () => updateCounts())
</script>

<template>
  <aside class="sidebar">
    <!-- Logo -->
    <div class="sidebar-logo">
      <img src="@/assets/logo-128.png" alt="Logo" class="sidebar-logo__icon" style="filter: none !important;" />
      <div>
        <h1 class="sidebar-logo__text" style="font-family:Consolas,'Microsoft YaHei','PingFang SC',sans-serif;font-size:17px;font-weight:700;line-height:1.2;">Yoshunko<br>Admin</h1>
        <span style="font-size:10px;color:var(--text-dim)">Game Data Manager</span>
      </div>
    </div>

    <!-- Player select (visible on hover) -->
    <div class="sidebar-player">
      <select class="sidebar-select" :value="uid ?? ''" @change="onPlayerChange">
        <option value="">-- 选择玩家 --</option>
        <option v-for="pid in players" :key="pid" :value="pid">UID: {{ pid }}</option>
      </select>
    </div>

    <!-- Navigation -->
    <nav class="sidebar-nav" role="navigation" aria-label="功能导航">
      <div
        v-for="item in navItems"
        :key="item.key"
        class="nav-item"
        :class="{ active: panel === item.key, dirty: dirty && panel === item.key }"
        role="tab"
        :aria-selected="panel === item.key"
        @click="selectPanel(item.key)"
      >
        <div class="nav-item__icon-wrap">
          <component :is="item.icon" :size="22" />
        </div>
        <span class="nav-item__label">{{ item.label }}</span>
        <span v-if="item.countKey === 'avatar'" class="nav-badge" id="avatar-count">{{ avatarCount }}</span>
        <span v-if="item.countKey === 'weapon'" class="nav-badge" id="weapon-count">{{ weaponCount }}</span>
        <span v-if="item.countKey === 'equip'" class="nav-badge" id="equip-count">{{ equipCount }}</span>
      </div>
    </nav>

    <!-- Footer -->
    <div class="sidebar-footer">
      <button class="sidebar-icon-btn" aria-label="设置" @click="selectPanel('settings')">
        <Settings :size="20" />
      </button>
      <button class="sidebar-icon-btn" aria-label="切换主题" @click="toggleTheme">
        <Sun v-if="currentTheme === 'dark'" :size="20" />
        <Moon v-else :size="20" />
      </button>
    </div>
  </aside>
</template>
