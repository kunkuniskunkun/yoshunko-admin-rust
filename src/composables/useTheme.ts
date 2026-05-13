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
  currentTheme.value = target
  document.documentElement.setAttribute('data-theme', target)
  try { localStorage.setItem('yos-theme', target) } catch {}

  // Overlay fade animation
  const overlay = document.createElement('div')
  overlay.className = 'theme-fade-overlay'
  overlay.style.background = oldBg
  document.body.appendChild(overlay)
  overlay.addEventListener('animationend', () => overlay.remove())
  setTimeout(() => { if (overlay.parentNode) overlay.remove() }, 700)
}
