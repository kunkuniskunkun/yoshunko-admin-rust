import { ref } from 'vue'
import { api } from '@/lib/api'

export const bgUrl = ref('')
export const bgOpacity = ref(0.85)
export const bgPath = ref('')

export async function setBackground(path: string, opacity: number) {
  bgPath.value = path
  bgOpacity.value = opacity
  if (!path) {
    bgUrl.value = ''
    return
  }
  try {
    const r = await api.readImageDataUrl(path)
    if (r.ok && r.url) {
      bgUrl.value = r.url
    } else {
      console.error('[setBackground] readImageDataUrl failed:', r.error)
      bgUrl.value = ''
    }
  } catch (e) {
    console.error('[setBackground] error:', e)
    bgUrl.value = ''
  }
}
