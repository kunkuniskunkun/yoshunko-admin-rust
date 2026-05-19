import { ref } from 'vue'

export const currentTheme = ref<'light' | 'dark'>('dark')

export function initTheme() {
  try {
    const saved = localStorage.getItem('yos-theme')
    if (saved === 'dark' || saved === 'light') {
      currentTheme.value = saved
      document.documentElement.setAttribute('data-theme', saved)
    } else {
      currentTheme.value = 'dark'
      document.documentElement.setAttribute('data-theme', 'dark')
    }
  } catch {
    currentTheme.value = 'dark'
    document.documentElement.setAttribute('data-theme', 'dark')
  }
}

export function toggleTheme() {
  const next = currentTheme.value === 'dark' ? 'light' : 'dark'
  setTheme(next)
}

export function setTheme(target: 'light' | 'dark') {
  if (currentTheme.value === target) return

  const oldBg = getComputedStyle(document.body).backgroundColor

  // 先创建 overlay 遮住整个页面
  const overlay = document.createElement('div')
  overlay.className = 'theme-fade-overlay'
  overlay.style.background = oldBg
  overlay.style.opacity = '1'
  document.body.appendChild(overlay)

  // overlay 出现后再切换主题
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      currentTheme.value = target
      document.documentElement.setAttribute('data-theme', target)
      try { localStorage.setItem('yos-theme', target) } catch {}

      // overlay 渐隐
      overlay.style.transition = 'opacity 0.75s cubic-bezier(0.4, 0, 0.2, 1)'
      overlay.style.opacity = '0'
      overlay.addEventListener('transitionend', () => overlay.remove())
      setTimeout(() => { if (overlay.parentNode) overlay.remove() }, 850)
    })
  })
}

// ─── Accent Color ─────────────────────────────────────

export type AccentColor = 'blue' | 'green' | 'purple' | 'red' | 'orange' | 'pink'

export const ACCENT_COLORS: { key: AccentColor; label: string; hex: string }[] = [
  { key: 'blue',   label: '海蓝', hex: '#4a9fd8' },
  { key: 'green',  label: '翠绿', hex: '#4caf7d' },
  { key: 'purple', label: '藤紫', hex: '#8b6cc1' },
  { key: 'red',    label: '珊瑚红', hex: '#e06060' },
  { key: 'orange', label: '琥珀橙', hex: '#e09050' },
  { key: 'pink',   label: '樱粉', hex: '#d07090' },
]

export const currentAccent = ref<AccentColor>('blue')

export function initAccent() {
  try {
    const saved = localStorage.getItem('yos-accent')
    if (saved && ACCENT_COLORS.some(c => c.key === saved)) {
      currentAccent.value = saved as AccentColor
    }
  } catch {}
  applyAccent(currentAccent.value)
}

export function setAccent(color: AccentColor) {
  currentAccent.value = color
  applyAccent(color)
  try { localStorage.setItem('yos-accent', color) } catch {}
}

function applyAccent(color: AccentColor) {
  if (color === 'blue') {
    document.documentElement.removeAttribute('data-accent')
  } else {
    document.documentElement.setAttribute('data-accent', color)
  }
}
