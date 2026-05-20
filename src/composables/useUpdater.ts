import { ref } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import type { UpdateInfo } from '@/lib/types'

export const updateInfo = ref<UpdateInfo | null>(null)
export const updateAvailable = ref(false)

let pendingUpdate: Awaited<ReturnType<typeof check>> = null

let lastError = ''

export async function checkUpdate(): Promise<boolean> {
  lastError = ''
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
  } catch (e) {
    // 开发环境或网络不可用时静默失败
    lastError = String(e)
    return false
  }
}

export function getLastError() { return lastError }

export async function installUpdate(onProgress?: (pct: number) => void): Promise<boolean> {
  if (!pendingUpdate) return false
  try {
    let total = 0
    let downloaded = 0
    await pendingUpdate.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength ?? 0
        if (total === 0 && onProgress) {
          onProgress(-1)
        }
      } else if (event.event === 'Progress' && onProgress) {
        downloaded += event.data.chunkLength
        if (total > 0) {
          onProgress(Math.min(Math.round((downloaded / total) * 100), 99))
        }
      }
    })
    return true
  } catch (e) {
    console.error('[Updater] install failed:', e)
    return false
  }
}

export function openReleasePage() {
  const url = 'https://github.com/kunkuniskunkun/yoshunko-admin-rust/releases'
  // Use Tauri shell plugin to open URL in default browser
  import('@tauri-apps/plugin-shell').then(({ open }) => {
    open(url)
  }).catch((e) => console.error('[Updater] open release page failed:', e))
}
