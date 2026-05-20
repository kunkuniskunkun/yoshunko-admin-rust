import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import type { UpdateInfo } from '@/lib/types'

export const updateInfo = ref<UpdateInfo | null>(null)
export const updateAvailable = ref(false)

let pendingUpdate: Awaited<ReturnType<typeof check>> = null

export async function checkUpdate(): Promise<boolean> {
  try {
    const update = await check()
    if (update) {
      pendingUpdate = update
      updateInfo.value = {
        version: update.version,
        body: update.body || '',
        date: update.date || '',
      }
      updateAvailable.value = true
      return true
    }
    return false
  } catch {
    // 静默失败—开发环境或网络不可用时不影响正常使用
    return false
  }
}

export async function installUpdate(onProgress?: (pct: number) => void) {
  if (!pendingUpdate) return
  try {
    let total = 0
    let downloaded = 0
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength ?? 0
      } else if (event.event === 'Progress' && onProgress) {
        downloaded += event.data.chunkLength
        if (total > 0) {
          onProgress(Math.min(Math.round((downloaded / total) * 100), 99))
        }
      }
    })
  } catch (e) {
    console.error('[Updater] install failed:', e)
  }
}

export function openReleasePage() {
  const url = 'https://github.com/kunkunr/yoshunko-admin-rust/releases'
  // Use Tauri shell plugin to open URL in default browser
  import('@tauri-apps/plugin-shell').then(({ open }) => {
    open(url)
  })
}
