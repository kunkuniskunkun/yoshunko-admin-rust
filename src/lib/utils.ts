import { ref } from 'vue'

/** HTML 转义 */
export function escHtml(s: string): string {
  const div = document.createElement('div')
  div.textContent = s
  return div.innerHTML
}

/** Toast 通知 */
export interface ToastItem {
  id: number
  message: string
  type: 'success' | 'error' | 'info'
}

export const toasts = ref<ToastItem[]>([])
let toastId = 0

export function toast(message: string, type: ToastItem['type'] = 'info') {
  const id = ++toastId
  toasts.value.push({ id, message, type })
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, 3000)
}

export function removeToast(id: number) {
  toasts.value = toasts.value.filter(t => t.id !== id)
}

/** 确认对话框 */
export interface ConfirmState {
  visible: boolean
  message: string
  onConfirm: (() => void | Promise<void>) | null
}

export const confirmState = ref<ConfirmState>({
  visible: false,
  message: '',
  onConfirm: null,
})

export function showConfirm(message: string, onConfirm: () => void | Promise<void>) {
  confirmState.value = { visible: true, message, onConfirm }
}

export function closeConfirm() {
  confirmState.value = { ...confirmState.value, visible: false, onConfirm: null }
}
