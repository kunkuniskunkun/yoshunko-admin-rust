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
  overlay.style.opacity = '0.92'
  document.body.appendChild(overlay)

  // overlay 出现后再切换主题
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      currentTheme.value = target
      document.documentElement.setAttribute('data-theme', target)
      try { localStorage.setItem('yos-theme', target) } catch {}

      // overlay 渐隐
      overlay.style.transition = 'opacity 0.8s cubic-bezier(0.4, 0, 0.2, 1)'
      overlay.style.opacity = '0'
      overlay.addEventListener('transitionend', () => overlay.remove())
      setTimeout(() => { if (overlay.parentNode) overlay.remove() }, 900)
    })
  })
}
