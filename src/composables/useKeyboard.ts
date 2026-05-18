import { onMounted, onUnmounted } from 'vue'
import { panel, uid, popUndo } from './useAppState'

export function useKeyboard(opts: {
  onSave?: () => void
  onBackToGallery?: () => void
}) {
  const PANEL_MAP: Record<string, string> = {
    '1': 'avatars',
    '2': 'weapons',
    '3': 'equips',
    '4': 'hadal_zone',
    '5': 'player_info',
    '6': 'settings',
    '7': 'quick_launch',
  }

  function handler(e: KeyboardEvent) {
    const target = e.target as HTMLElement
    if (target.tagName === 'INPUT' || target.tagName === 'SELECT' || target.tagName === 'TEXTAREA') return

    // ESC
    if (e.key === 'Escape') {
      opts.onBackToGallery?.()
      return
    }

    // 1-7: panel switch
    if (e.key >= '1' && e.key <= '7' && !e.ctrlKey && !e.altKey && !e.metaKey) {
      const p = PANEL_MAP[e.key]
      if (p && uid.value) {
        panel.value = p
      }
      return
    }

    // Ctrl+S: save
    if (e.ctrlKey && e.key === 's') {
      e.preventDefault()
      opts.onSave?.()
      return
    }

    // Ctrl+Z: undo
    if (e.ctrlKey && !e.shiftKey && e.key === 'z') {
      e.preventDefault()
      const snap = popUndo()
      if (snap) Promise.resolve(snap.restore()).catch(() => {})
      return
    }

    // Ctrl+F: focus search input
    if (e.ctrlKey && e.key === 'f') {
      e.preventDefault()
      const input = document.querySelector('.search-input') as HTMLInputElement | null
      if (input) { input.focus(); input.select() }
      return
    }
  }

  onMounted(() => document.addEventListener('keydown', handler))
  onUnmounted(() => document.removeEventListener('keydown', handler))
}
