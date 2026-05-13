import { nextTick } from 'vue'

export function applyStaggeredAnimation(selector: string) {
  nextTick(() => {
    const cards = document.querySelectorAll(selector)
    cards.forEach((card, i) => {
      const el = card as HTMLElement
      el.style.opacity = '0'
      el.style.transform = 'translateY(16px)'
      el.style.transition = 'opacity 0.3s cubic-bezier(0.34,1.56,0.64,1), transform 0.3s cubic-bezier(0.34,1.56,0.64,1)'
      el.style.transitionDelay = (i % 6) * 35 + 'ms'
      requestAnimationFrame(() => {
        el.style.opacity = '1'
        el.style.transform = 'translateY(0)'
      })
    })
  })
}

export function applyEditorSlideIn(el: HTMLElement) {
  el.classList.remove('editor-slide-in')
  void el.offsetHeight // force reflow
  el.classList.add('editor-slide-in')
}
