/**
 * Re-trigger CSS staggered card animation.
 * Uses .staggered-anim / .no-animate CSS classes + requestAnimationFrame
 * to avoid per-element forced reflows.
 */
export function applyStaggeredAnimation() {
  const container = document.querySelector('.main-content')
  if (!container) return
  container.classList.add('no-animate')
  requestAnimationFrame(() => {
    container.classList.remove('no-animate')
  })
}

export function applyEditorSlideIn(el: HTMLElement) {
  el.classList.remove('editor-slide-in')
  void el.offsetHeight // force reflow — single element, negligible cost
  el.classList.add('editor-slide-in')
}

export function applyEditorSlideBack(el: HTMLElement) {
  el.classList.remove('editor-slide-back')
  void el.offsetHeight
  el.classList.add('editor-slide-back')
}
