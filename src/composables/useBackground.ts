import { ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'

export const bgUrl = ref('')
export const bgOpacity = ref(0.85)
export const bgPath = ref('')

export function setBackground(path: string, opacity: number) {
  bgPath.value = path
  bgOpacity.value = opacity
  bgUrl.value = path ? convertFileSrc(path) : ''
}
