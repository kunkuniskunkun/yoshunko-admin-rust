import { ref, computed, watch, onMounted, onActivated, nextTick, type Ref, type ComputedRef } from 'vue'
import { uid, panel, searchQuery, scrollPos, dirty, cacheDirty, markDirty, markClean, markCacheDirty, pushUndo } from './useAppState'
import { api } from '@/lib/api'
import { toast } from '@/lib/utils'
import { applyStaggeredAnimation, applyEditorSlideIn, applyEditorSlideBack } from './useStaggeredAnimation'

// ─── Types ──────────────────────────────────────────

export interface PanelEditorOpts<ListItem, Detail> {
  panelKey: 'avatars' | 'weapons' | 'equips'
  entityName: string  // for toasts: '角色' | '音擎' | '驱动盘'

  // API
  loadList: () => Promise<{ items: ListItem[] }>
  loadDetail: (uid: number, id: number) => Promise<Detail | null>
  saveDetail: (uid: number, id: number, data: Record<string, unknown>) => Promise<{ ok: boolean; error?: string }>
  deleteDetail?: (uid: number, id: number) => Promise<{ ok: boolean; error?: string }>
  copyDetail?: (uid: number, id: number) => Promise<{ ok: boolean; uid?: number; error?: string }>

  // ID extraction
  getItemId: (item: ListItem) => number
  getDetailId: (detail: Detail) => number

  // Cache ref (from useAppState)
  cache: Ref<ListItem[]>
  selectedId: Ref<number | null>
  viewRef: Ref<'gallery' | 'editor'>

  // Search & group
  filterFn: (item: ListItem, query: string) => boolean
  sortFn?: (a: ListItem, b: ListItem) => number
  groupFn: (item: ListItem) => string
  groupSort?: (a: [string, ListItem[]], b: [string, ListItem[]]) => number

  // Editor field mapping
  mapDetailToEdit: (detail: Detail) => void  // populate edit refs from detail
  buildSavePayload: () => Record<string, unknown>
  snapshotOldData?: (detail: Detail) => Record<string, unknown>

  // Undo restore (optional customization)
  buildUndoRestore?: (id: number, oldData: Record<string, unknown>) => () => Promise<void>

  // Optional hooks
  onBeforeSave?: () => void
  onAfterSave?: () => void
}

// ─── Composable ──────────────────────────────────────

export function usePanelEditor<ListItem, Detail>(opts: PanelEditorOpts<ListItem, Detail>) {
  const loading = ref(true)
  const refreshing = ref(false)
  const editorData = ref<Detail | null>(null) as Ref<Detail | null>
  const editorLoading = ref(false)
  const saving = ref(false)
  const hasAnimated = ref(false)
  const editorReady = ref(false)

  const searchKey = opts.panelKey as 'avatars' | 'weapons' | 'equips'

  // ─── Filtered items ─────────────────────
  const filteredItems = computed(() => {
    const query = searchQuery[searchKey]
    const defaultSort = (a: ListItem, b: ListItem) => opts.getItemId(a) - opts.getItemId(b)
    return opts.cache.value
      .filter(item => opts.filterFn(item, query))
      .sort(opts.sortFn || defaultSort)
  })

  // ─── Grouped items ──────────────────────
  const groupedItems = computed(() => {
    const groups = new Map<string, ListItem[]>()
    for (const item of filteredItems.value) {
      const key = opts.groupFn(item)
      if (!groups.has(key)) groups.set(key, [])
      groups.get(key)!.push(item)
    }
    if (opts.groupSort) {
      const sorted = new Map<string, ListItem[]>()
      const entries = [...groups.entries()].sort(opts.groupSort)
      for (const [k, v] of entries) sorted.set(k, v)
      return sorted
    }
    return groups
  })

  // ─── Stagger index ──────────────────────
  const staggerIndex = computed(() => {
    const map = new Map<number, number>()
    let idx = 0
    for (const [, items] of groupedItems.value) {
      for (const item of items) map.set(opts.getItemId(item), idx++)
    }
    return map
  })

  // ─── Select item (enter editor) ─────────
  function selectItem(id: number, event?: MouseEvent) {
    // Save scroll position before gallery collapses
    const main = document.querySelector('.main-content')
    if (main) scrollPos.value[opts.panelKey] = main.scrollTop

    // Card press animation
    if (event) {
      const card = (event.currentTarget as HTMLElement)
      card.style.transition = 'transform 0.12s ease'
      card.style.transform = 'skewX(-2deg) scale(0.96)'
      setTimeout(() => {
        card.style.transition = 'transform 0.2s ease'
        card.style.transform = ''
        setTimeout(() => { card.style.transition = ''; card.style.transform = '' }, 200)
      }, 120)
    }

    opts.selectedId.value = id
    opts.viewRef.value = 'editor'
    nextTick(() => {
      const mainEl = document.querySelector('.main-content') as HTMLElement
      if (mainEl) applyEditorSlideIn(mainEl)
    })
    loadEditor(id)
  }

  // ─── Back to gallery ────────────────────
  function backToGallery() {
    editorReady.value = false
    opts.viewRef.value = 'gallery'
    opts.selectedId.value = null
    editorData.value = null
    // Reverse slide animation before restoring scroll
    setTimeout(() => {
      const mainEl = document.querySelector('.main-content') as HTMLElement
      if (mainEl) applyEditorSlideBack(mainEl)
    }, 10)
    // Restore scroll after gallery un-hides
    requestAnimationFrame(() => {
      const main = document.querySelector('.main-content')
      if (main && scrollPos.value[opts.panelKey] != null) {
        main.scrollTop = scrollPos.value[opts.panelKey]
      }
    })
  }

  // ─── Load editor ────────────────────────
  async function loadEditor(id: number) {
    if (!uid.value) return
    editorLoading.value = true
    try {
      const data = await opts.loadDetail(uid.value, id)
      if (opts.selectedId.value !== id) return
      if (!data) {
        toast(`${opts.entityName}数据加载失败`, 'error')
        backToGallery()
        return
      }
      editorData.value = data
      opts.mapDetailToEdit(data)
      nextTick(() => { editorReady.value = true })
    } catch (e: unknown) {
      toast(`加载失败: ${e instanceof Error ? e.message : ''}`, 'error')
      backToGallery()
    }
    editorLoading.value = false
  }

  // ─── Refresh cache ──────────────────────
  async function refreshCache() {
    if (!uid.value) return
    if (opts.cache.value.length && !cacheDirty[searchKey]) {
      loading.value = false
      return
    }
    if (refreshing.value) return
    refreshing.value = true
    try {
      const { items } = await opts.loadList()
      opts.cache.value = items
      cacheDirty[searchKey] = false
    } catch (e: unknown) {
      toast(`加载失败: ${e instanceof Error ? e.message : ''}`, 'error')
    }
    refreshing.value = false
    loading.value = false
  }

  // ─── Save item ──────────────────────────
  async function saveItem() {
    if (!editorData.value || !uid.value || !opts.selectedId.value) return
    saving.value = true
    try {
      opts.onBeforeSave?.()
      const payload = opts.buildSavePayload()
      const oldData = opts.snapshotOldData ? opts.snapshotOldData(editorData.value) : {}
      const id = opts.selectedId.value

      const r = await opts.saveDetail(uid.value, id, payload)
      if (!r.ok) { toast(`保存失败: ${r.error}`, 'error'); saving.value = false; return }

      toast(`${opts.entityName}已保存`, 'success')

      // Build undo restore
      const restoreFn = opts.buildUndoRestore
        ? opts.buildUndoRestore(id, oldData)
        : async () => {
            await opts.saveDetail(uid.value!, id, oldData)
            opts.viewRef.value = 'editor'
            opts.selectedId.value = id
            await loadEditor(id)
            toast('已撤回保存', 'info')
          }

      pushUndo({ restore: restoreFn })
      markClean(searchKey)
      markCacheDirty(searchKey)
      await refreshCache()
      backToGallery()
    } catch (e: unknown) {
      toast(`保存失败: ${e instanceof Error ? e.message : ''}`, 'error')
    }
    saving.value = false
  }

  // ─── Delete item ────────────────────────
  async function deleteItem() {
    if (!opts.deleteDetail || !uid.value || !opts.selectedId.value) return
    const id = opts.selectedId.value
    const oldData = editorData.value ? (opts.snapshotOldData ? opts.snapshotOldData(editorData.value) : {}) : {}

    try {
      const r = await opts.deleteDetail(uid.value, id)
      if (!r.ok) { toast(`删除失败: ${r.error}`, 'error'); return }
      toast(`${opts.entityName}已删除`, 'success')

      // Default undo: re-save the old data
      pushUndo({
        restore: async () => {
          await opts.saveDetail(uid.value!, id, oldData)
          markClean(searchKey)
          toast('已撤回删除', 'info')
        }
      })
      markClean(searchKey)
      markCacheDirty(searchKey)
      backToGallery()
      await refreshCache()
    } catch (e: unknown) {
      toast(`删除失败: ${e instanceof Error ? e.message : ''}`, 'error')
    }
  }

  // ─── Copy item ──────────────────────────
  async function copyItem() {
    if (!opts.copyDetail || !uid.value || !opts.selectedId.value) return
    const id = opts.selectedId.value

    try {
      const r = await opts.copyDetail(uid.value, id)
      if (!r.ok) { toast(`复制失败: ${r.error}`, 'error'); return }
      toast(`${opts.entityName}已复制 #${r.uid}`, 'success')

      pushUndo({
        restore: async () => {
          if (r.uid && opts.deleteDetail) {
            await opts.deleteDetail(uid.value!, r.uid)
            markCacheDirty(searchKey)
            toast('已撤回复制', 'info')
          }
        }
      })
      markCacheDirty(searchKey)
      await refreshCache()
      backToGallery()
    } catch (e: unknown) {
      toast(`复制失败: ${e instanceof Error ? e.message : ''}`, 'error')
    }
  }

  // ─── Lifecycle ──────────────────────────
  watch(panel, (_, old) => {
    if (old === opts.panelKey) {
      opts.viewRef.value = 'gallery'
      opts.selectedId.value = null
      searchQuery[searchKey] = ''
      hasAnimated.value = false
    }
  })

  onMounted(async () => {
    await refreshCache()
    if (opts.viewRef.value === 'editor' && opts.selectedId.value) {
      await loadEditor(opts.selectedId.value)
    } else {
      nextTick(() => { applyStaggeredAnimation(); hasAnimated.value = true })
    }
  })

  onActivated(async () => {
    await refreshCache()
    if (!hasAnimated.value) {
      nextTick(() => { applyStaggeredAnimation(); hasAnimated.value = true })
    }
  })

  return {
    loading,
    refreshing,
    editorData,
    editorLoading,
    saving,
    editorReady,
    filteredItems,
    groupedItems,
    staggerIndex,
    selectItem,
    backToGallery,
    refreshCache,
    saveItem,
    deleteItem,
    copyItem,
    loadEditor,
  }
}
