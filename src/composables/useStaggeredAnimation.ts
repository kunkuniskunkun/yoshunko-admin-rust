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
  el.animate([
    { opacity: 0, transform: 'translateY(24px) scale(0.96)' },
    { opacity: 1, transform: 'translateY(0) scale(1)' },
  ], {
    duration: 720,
    easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
    fill: 'forwards',
  })
}

export function applyEditorSlideBack(el: HTMLElement) {
  el.animate([
    { opacity: 0, transform: 'translateY(-24px) scale(0.96)' },
    { opacity: 1, transform: 'translateY(0) scale(1)' },
  ], {
    duration: 720,
    easing: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
    fill: 'forwards',
  })
}
