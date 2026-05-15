# 运行日志记录功能设计

> 日期：2026-05-15 · 当前版本：V0.630

## 目标

为快速启动面板的 4 个进程（HoyoSDK、Yoshunko Server、KCPSHIM、Client）添加日志记录功能，输出持久化到文件，可在 Settings 面板回溯查看。

## 约束

- 去掉 `CREATE_NEW_CONSOLE`，stdout/stderr 重定向到日志文件
- 日志存储在 `exe_dir/logs/` 目录
- 按大小轮转：单文件上限 5MB，保留最近 3 个备份
- 每行带时间戳：`[2026-05-15 17:30:01] message`
- 在 Settings 面板查看日志，进入时加载一次 + 手动刷新

## 后端设计

### 新增 `log_manager.rs`

```
LogManager {
    log_dir: PathBuf,   // exe_dir/logs/
}

LogManager::new(exe_dir) -> Self
    - 创建 logs/ 目录（如不存在）

LogManager::init_log(key: &str) -> File
    - 打开 logs/{key}.log，追加模式
    - 返回 File handle

LogManager::rotate_log(key: &str)
    - 检查 logs/{key}.log 大小
    - 超过 5MB 时：{key}.log.2 删除 → .1 重命名为 .2 → 当前文件重命名为 .1
    - best-effort，失败不阻塞

LogManager::read_log(key: &str, offset: u64) -> (String, u64)
    - 从 offset 位置读取文件新增内容
    - 返回 (内容, 新 offset)
    - 文件不存在返回 ("", 0)

LogManager::log_path(key: &str) -> PathBuf
    - 返回日志文件完整路径
```

### 修改 `api.rs`

**修改 `launch_program`：**
- 去掉 `CREATE_NEW_CONSOLE`
- 调用 `log_manager.rotate_log(key)` 检查轮转
- 用 `log_manager.init_log(key)` 获取 File
- `Command::new(path).stdout(file).stderr(file)` 重定向输出
- Yoshunko 特殊处理：`wsl` 命令的 stdout 重定向到 `yoshunko.log`

**新增 Tauri 命令：**
- `read_log(key: String, offset: u64)` → `{content: String, offset: u64}`
- `get_log_dir()` → `{path: String}`（供"打开文件夹"按钮用）

### 修改 `lib.rs`

- AppState 新增 `log_manager: LogManager` 字段
- 注册 `read_log`、`get_log_dir` 命令

## 前端设计

### 修改 `SettingsPanel.vue`

新增"运行日志"区域：

- 4 个标签页：HoyoSDK / Yoshunko / KCPSHIM / Client
- 每个标签页：
  - `<pre>` 日志内容区，等宽字体，max-height + overflow-y: auto
  - 底部按钮栏："刷新" + "打开日志文件夹"
- `onActivated` 时加载当前标签的日志
- 切换标签时加载对应日志
- 刷新时用 offset 增量读取，追加到现有内容

### 修改 `api.ts`

新增：
- `readLog(key: string, offset: number)` → `{content: string, offset: number}`
- `getLogDir()` → `{path: string}`

## 日志文件

| 进程 | key | 文件名 |
|------|-----|--------|
| HoyoSDK | `hoyosdk` | `logs/hoyosdk.log` |
| Yoshunko Server | `yoshunko` | `logs/yoshunko.log` |
| KCPSHIM | `kcpshim` | `logs/kcpshim.log` |
| Client | `client` | `logs/client.log` |

轮转文件：`{key}.log.1`、`{key}.log.2`、`{key}.log.3`

## 错误处理

- 日志目录不存在 → 自动创建
- 进程未启动时读日志 → 返回空内容
- 日志文件被外部删除 → 返回空内容
- 轮转失败 → best-effort，不阻塞写入
- WSL 命令管道读取失败 → 记录错误到日志文件本身
