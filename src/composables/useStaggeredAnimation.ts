import { nextTick } from 'vue'

let pendingFrames = new Set<number>()

export function applyStaggeredAnimation(selector: string) {
  for (const id of pendingFrames) cancelAnimationFrame(id)
  pendingFrames.clear()

  // Use setTimeout to ensure DOM is fully re-inserted by KeepAlive before querying
  setTimeout(() => {
    const cards = document.querySelectorAll(selector)
    if (!cards.length) return

    cards.forEach((card, i) => {
      const el = card as HTMLElement
      // Reset to start position
      el.style.transition = 'none'
      el.style.opacity = '0'
      el.style.transform = 'translateY(16px)'
      // Force reflow so reset is painted before animating
      void el.offsetHeight

      // Animate in with stagger (+15% slower)
      const delay = (i % 8) * 35
      el.style.transition = `opacity 0.35s cubic-bezier(0.34,1.56,0.64,1) ${delay}ms, transform 0.35s cubic-bezier(0.34,1.56,0.64,1) ${delay}ms`
      const id = requestAnimationFrame(() => {
        el.style.opacity = '1'
        el.style.transform = 'translateY(0)'
        pendingFrames.delete(id)
      })
      pendingFrames.add(id)
    })
  }, 0)
}

export function applyEditorSlideIn(el: HTMLElement) {
  el.classList.remove('editor-slide-in')
  void el.offsetHeight // force reflow
  el.classList.add('editor-slide-in')
}
