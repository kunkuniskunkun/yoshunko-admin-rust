# Yoshunko Admin 前端重写设计文档 — 三次审阅意见

> 审阅对象：`specs/2026-05-13-frontend-rewrite-design.md`（二次更新版）
> 审阅日期：2026-05-13
> 审阅方法：逐条对照 `api.rs`、`lib.rs`、前端 JS 源码验证事实准确性
> 原则：仅指出文档与代码实际不符的事实性错误，不添加代码库中不存在的新设计

---

## 总体评价

二次更新版已修正首次审阅的全部 P0/P1 问题。当前文档的主要问题集中在第 10 节 TypeScript 类型定义——多处与 `api.rs` 实际返回的字段名和结构不一致。以下仅列出经代码验证的事实性错误。

---

## 事实性错误

### 1. `PlayerBasic.display_avatar_id` — 字段名错误

文档 L563：`display_avatar_id: number`

api.rs L232：`"avatar_id": zon_int(&info, "avatar_id", 2011)`

前端 `player.js` L67 发送的字段也是 `avatar_id`。

**修正**：`display_avatar_id` → `avatar_id`

### 2. `PlayerBasic.disguise_avatar_id` — 字段名错误

文档 L565：`disguise_avatar_id: number`

api.rs L234：`"control_guise_avatar_id": zon_int(&info, "control_guise_avatar_id", 1541)`

前端 `player.js` L69 发送的字段也是 `control_guise_avatar_id`。

**修正**：`disguise_avatar_id` → `control_guise_avatar_id`

### 3. `PlayerBasicUpdate` 同样的字段名错误

文档 L649-651 的 `display_avatar_id` 和 `disguise_avatar_id` 同样应改为 `avatar_id` 和 `control_guise_avatar_id`。

### 4. `Avatar` 列表类型 — 字段不匹配

文档 L568-574：
```typescript
export interface Avatar {
  avatar_id: number
  name: string
  level: number
  rank: number       // ❌
  awake_id: number   // ❌
}
```

api.rs L264-277 `get_avatars` 实际返回 9 个字段：
- `avatar_id`、`name`、`en_name`、`rarity`、`profession`、`level`、`unlocked_talent_num`、`is_favorite`、`camp_id`

列表中**没有** `rank` 和 `awake_id`（这两个只在 `get_avatar` 详情中有），**缺少** `en_name`、`rarity`、`profession`、`unlocked_talent_num`、`is_favorite`、`camp_id`。

### 5. `AvatarDetail` — `avatar` 嵌套对象类型错误

文档 L576-580：
```typescript
export interface AvatarDetail {
  avatar: Avatar           // ❌ 复用了列表类型
  forms: Record<string, unknown>
  skill_type_level: Record<string, number>  // ❌
}
```

api.rs L288-313 `get_avatar` 返回的 `avatar` 对象有 18 个字段（avatar_id, name, en_name, rarity, profession, level, exp, rank, unlocked_talent_num, talent_switch_list, passive_skill_level, cur_weapon_uid, is_favorite, avatar_skin_id, is_awake_available, awake_id, cur_form_id, is_awake_enabled, dressed_equip, show_weapon_type, skill_type_level），与列表项 `Avatar` 完全不同。

`skill_type_level` 是 `Array<{type: string, level: number}>`（api.rs L535-547 `extract_skills`），不是 `Record<string, number>`。

`forms` 实际返回空数组 `[]`（api.rs L312），不是 `Record<string, unknown>`。

### 6. `WeaponDetail` — 不存在嵌套包装

文档 L591-593：
```typescript
export interface WeaponDetail {
  weapon: Weapon    // ❌
}
```

api.rs L368-391 `get_weapon` 直接返回扁平对象（uid, id, name, en_name, profession, level, star, refine_level, lock, max_star, max_refine），没有嵌套的 `weapon` 包装。

### 7. `Weapon` 列表类型 — 缺少字段

文档 L582-589 缺少 `en_name`、`profession`、`max_star`、`max_refine`，这些在 api.rs L347-363 中都有返回。

### 8. `EquipDetail` — 不存在嵌套包装

文档 L612-614：
```typescript
export interface EquipDetail {
  equip: Equip    // ❌
}
```

api.rs L435-458 `get_equip` 直接返回扁平对象（uid, id, suit_name, suit_en_name, slot, slot_name, level, exp, star, lock, properties, sub_properties），没有嵌套的 `equip` 包装。

### 9. `Equip` 列表类型 — 字段不匹配

文档 L595-604：
```typescript
export interface Equip {
  uid: number
  id: number
  level: number
  star: number
  suit_type: number      // ❌ 列表中不返回
  slot: number
  properties: EquipProperty[]      // ❌ 列表中不返回
  sub_properties: EquipProperty[]  // ❌ 列表中不返回
}
```

api.rs L416-428 `get_equips` 实际返回：uid, id, suit_name, suit_en_name, slot, slot_name, level, star。没有 `suit_type`、`properties`、`sub_properties`（这些只在详情中有），缺少 `suit_name`、`suit_en_name`、`slot_name`。

### 10. `EquipProperty` — 字段名全部错误

文档 L606-610：
```typescript
export interface EquipProperty {
  property_id: number    // ❌
  value: number          // ❌
  enhance_level?: number // ❌
}
```

api.rs L588-604 `extract_equip_properties` 实际返回：
```json
{ "key": 0, "key_name": "", "base_value": 0, "add_value": 0 }
```

前端 `equips.js` L246-249、L533-536 也使用 `key`、`base_value`、`add_value`。

### 11. `HadalZone` — `rooms` 字段不存在

文档 L616-619：
```typescript
export interface HadalZone {
  entrances: HadalEntrance[]
  rooms: HadalRoom[]     // ❌
}
```

api.rs L502-510 `get_hadal_zone` 只返回 `{ entrances: ... }`，没有 `rooms` 字段。

### 12. `AvatarUpdate` — 字段名错误

文档 L654-661：
```typescript
export interface AvatarUpdate {
  level?: number
  rank?: number
  awake_id?: number
  skill_type_level?: Record<string, number>  // ❌
  skin_id?: number              // ❌
  dressed_weapon_uid?: number   // ❌
}
```

前端 `avatars.js` L263-270 实际发送：
```javascript
{
  level: ...,
  passive_skill_level: ...,     // 文档缺少
  unlocked_talent_num: ...,     // 文档缺少
  skill_type_level: [...],      // 是数组，不是 Record
  awake_id: ...,
  avatar_skin_id: ...,          // 不是 skin_id
  cur_weapon_uid: ...           // 不是 dressed_weapon_uid
}
```

### 13. `EquipCreate` — 字段不准确

文档 L676-681：
```typescript
export interface EquipCreate {
  suit_type: number      // ❌
  slot: number           // ❌
  properties: EquipProperty[]
  sub_properties: EquipProperty[]
}
```

前端 `equips.js` L554-558 实际发送：
```javascript
{
  id: equipId,           // 不是 suit_type + slot
  level: 15,
  star: 5,
  properties: [{ key: mainKey, base_value: mainBase, add_value: 0 }],
  sub_properties: subProps  // 数组元素可为 null
}
```

### 14. `EquipUpdate` — 缺少字段

文档 L669-674：
```typescript
export interface EquipUpdate {
  level?: number
  star?: number
  properties?: EquipProperty[]
  sub_properties?: EquipProperty[]
}
```

前端 `equips.js` L269-274 实际发送：
```javascript
{
  level: ...,
  star: ...,
  properties: [{ key: mainKey, base_value: mainBase, add_value: 0 }],
  sub_properties: subProps  // 数组元素可为 null
}
```

`properties` 和 `sub_properties` 不是可选的，每次保存都发送。`sub_properties` 数组元素可为 `null`（与 `EquipProperty` 类型不兼容）。

### 15. `Config` — 缺少 `version` 字段

文档 L552-557：
```typescript
export interface Config {
  configured: boolean
  config_exists: boolean
  state_dir?: string
  launch_config?: Record<string, string>
}
```

api.rs L66-72 返回 5 个字段：`configured`、`state_dir`、`version`、`config_exists`、`launch_config`。缺少 `version: string`。

### 16. `Templates.avatars` — 类型错误

文档 L634：`avatars: Record<number, AvatarTemplate>`

api.rs L173-182 返回的是数组 `Vec<Value>`，每个元素包含 `id`、`name`、`rarity`、`camp_id`、`camp_name`。不是 `Record<number, ...>`。

### 17. api.ts 中 `debugListDir` 和 `debugAvatarIds` 返回类型错误

文档 L357-358：
```typescript
debugListDir: (path: string) => invoke<string[]>('debug_list_dir', { path }),
debugAvatarIds: (uid: number) => invoke<number[]>('debug_avatar_ids', { uid }),
```

api.rs L122-134 `debug_list_dir` 返回 `{ path, exists, is_dir, entries: [{name, is_dir}] }`。
api.rs L136-166 `debug_avatar_ids` 返回 `{ count, first_result: {...} }`。

### 18. api.ts 中 `createEquip` 返回类型错误

文档 L382：
```typescript
createEquip: (uid: number, data: EquipCreate) => invoke<number>('create_equip', { uid, data }),
```

api.rs L484-486 返回 `{ ok: true, uid: number }` 或 `{ ok: false, error: string }`，不是 `number`。

### 19. 架构图命令数标注错误

文档 L93：`api.rs (33 cmds)`

`lib.rs` L47-74 注册了 26 个命令。新增 5 个快速启动命令后为 31 个。33 不正确。

### 20. 8.3 节 `safeInvoke` 注释与实际不符

文档 L446 注释：`// Rust 后端返回 { ok: true, data: T } 或 { ok: false, error: string }`

实际上 Rust 后端返回格式不统一：
- `get_player_basic` 返回 `{ nickname, level, ... }`（无 `ok` 字段）
- `update_player_basic` 返回 `{ ok: true }` 或 `{ ok: false, error: "..." }`
- `get_avatar` 找不到时返回 `null`
- `create_equip` 返回 `{ ok: true, uid: 123 }` 或 `{ ok: false, error: "..." }`

不存在 `{ ok: true, data: T }` 这种格式。

### 21. 8.2 节 shell 权限与快速启动方案矛盾

文档 8.2 节决定快速启动通过 Rust 后端新增命令（`std::process::Command` / `ShellExecuteW`）实现，不使用 `tauri-plugin-shell`。

但 9.2 节仍配置了 `shell:allow-execute`、`shell:allow-spawn` 权限，9.3 节仍将 `@tauri-apps/plugin-shell` 列为 dependency。

如果快速启动用后端命令，则前端不需要 shell 插件，也不需要 shell 权限。

### 22. 2 节技术栈表中 Shell 行与 8.2 节决策矛盾

文档 L27：`Shell | @tauri-apps/plugin-shell | 2.x | 前端直接执行 shell 命令（快速启动）`

但 8.2 节已决定快速启动不用 shell 插件，改用后端命令。此行描述与决策矛盾。

---

## 修正对照表

| 文档位置 | 文档内容 | 代码实际 | 修正 |
|---|---|---|---|
| L563 | `display_avatar_id` | `avatar_id` (api.rs L232) | 改名 |
| L565 | `disguise_avatar_id` | `control_guise_avatar_id` (api.rs L234) | 改名 |
| L568-574 | `Avatar` 含 rank, awake_id | 列表不含这两个，缺 6 个字段 (api.rs L264-277) | 重写类型 |
| L576-580 | `AvatarDetail.avatar: Avatar` | 详情对象有 18+ 字段，与列表项不同 (api.rs L288-313) | 新建详情类型 |
| L580 | `skill_type_level: Record` | `Array<{type, level}>` (api.rs L535-547) | 改类型 |
| L591-593 | `WeaponDetail: { weapon: Weapon }` | 扁平对象 (api.rs L368-391) | 去掉嵌套 |
| L582-589 | `Weapon` 缺字段 | 缺 en_name, profession, max_star, max_refine (api.rs L347-363) | 补充 |
| L612-614 | `EquipDetail: { equip: Equip }` | 扁平对象 (api.rs L435-458) | 去掉嵌套 |
| L595-604 | `Equip` 含 suit_type, properties | 列表不含，缺 suit_name 等 (api.rs L416-428) | 重写类型 |
| L606-610 | `EquipProperty` 含 property_id, value | key, key_name, base_value, add_value (api.rs L588-604) | 全部改名 |
| L616-619 | `HadalZone` 含 rooms | 只返回 entrances (api.rs L502-510) | 删除 rooms |
| L659 | `skin_id` | `avatar_skin_id` (avatars.js L269) | 改名 |
| L660 | `dressed_weapon_uid` | `cur_weapon_uid` (avatars.js L270) | 改名 |
| L676-681 | `EquipCreate` 含 suit_type, slot | 含 id, level, star (equips.js L554-558) | 重写类型 |
| L552-557 | `Config` 缺 version | 有 version (api.rs L69) | 补充 |
| L634 | `avatars: Record<number, ...>` | 数组 (api.rs L173-182) | 改类型 |
| L357-358 | debug 返回 `string[]` / `number[]` | 复杂对象 (api.rs L122-166) | 改类型 |
| L382 | `createEquip` 返回 `number` | `{ ok, uid?, error? }` (api.rs L484-486) | 改类型 |
| L93 | `33 cmds` | 26 现有 + 5 新增 = 31 | 改数字 |
| L446 | 注释说返回 `{ ok: true, data: T }` | 不存在此格式 | 改注释 |
| L27, L479-490, L509 | shell 插件/权限 | 8.2 节已决定不用 | 删除或标注 |
