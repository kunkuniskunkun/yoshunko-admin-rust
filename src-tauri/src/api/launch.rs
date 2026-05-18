use super::*;

#[tauri::command]
pub fn get_launch_config(state: State<AppState>) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    json!({"config": config.get("launch").cloned().unwrap_or(json!({}))})
}

#[tauri::command]
pub fn set_launch_path(state: State<AppState>, key: String, path: String) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let mut config_map = config.as_object().cloned().unwrap_or_default();
    let mut launch = config_map.get("launch").cloned().unwrap_or(json!({}));
    if let Some(launch_obj) = launch.as_object_mut() {
        launch_obj.insert(key, json!(path));
    }
    config_map.insert("launch".to_string(), launch);
    if let Err(e) = atomic_write_config(&state.config_path, &config_map) {
        return json!({"ok": false, "error": e});
    }
    json!({"ok": true})
}

#[tauri::command]
pub fn launch_program(state: State<AppState>, key: String) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let path = config.get("launch")
        .and_then(|l| l.get(&key))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if path.is_empty() {
        return json!({"ok": false, "error": format!("路径未配置: {}", key)});
    }
    let p = std::path::Path::new(path);
    if !p.exists() {
        return json!({"ok": false, "error": format!("文件不存在: {}", path)});
    }
    let lm = &state.log_manager;
    let (log_file, _log_name) = lm.create_log(&key);
    let cwd = p.parent().map(|d| d.to_path_buf());
    let mut cmd = std::process::Command::new(path);
    if let Some(dir) = cwd { cmd.current_dir(dir); }
    if let Ok(log_file2) = log_file.try_clone() {
        use std::process::Stdio;
        cmd.stdout(Stdio::from(log_file));
        cmd.stderr(Stdio::from(log_file2));
    }
    #[cfg(windows)]
    { cmd.creation_flags(CREATE_NO_WINDOW); }
    match cmd.spawn() {
        Ok(child) => {
            if let Ok(mut procs) = state.running_processes.lock() {
                procs.insert(key, child.id());
            }
            json!({"ok": true})
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

#[tauri::command]
pub fn launch_program_admin(state: State<AppState>, path: String, key: String) -> Value {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return json!({"ok": false, "error": format!("文件不存在: {}", path)});
    }
    #[cfg(windows)]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::Shell::ShellExecuteW;
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let file_w: Vec<u16> = OsStr::new(&path).encode_wide().chain(Some(0)).collect();
        let verb_w: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
        let cwd = p.parent().map(|d| {
            let w: Vec<u16> = OsStr::new(d).encode_wide().chain(Some(0)).collect();
            w
        });
        let cwd_ptr = cwd.as_ref().map(|w| w.as_ptr()).unwrap_or(std::ptr::null());

        let result = unsafe {
            ShellExecuteW(std::ptr::null_mut(), verb_w.as_ptr(), file_w.as_ptr(), std::ptr::null(), cwd_ptr, SW_SHOWNORMAL)
        };
        // ShellExecuteW returns a value > 32 on success
        let result_val = result as isize;
        if result_val > 32 {
            // Mark as running immediately (admin process may not be visible to us)
            if let Ok(mut procs) = state.running_processes.lock() {
                procs.insert(key.clone(), 1);
            }
            json!({"ok": true})
        } else {
            json!({"ok": false, "error": format!("ShellExecuteW failed: code {}", result_val)})
        }
    }
    #[cfg(not(windows))]
    {
        match std::process::Command::new("pkexec").arg(&path).spawn() {
            Ok(_) => json!({"ok": true}),
            Err(e) => json!({"ok": false, "error": e.to_string()}),
        }
    }
}

#[tauri::command]
pub fn launch_yoshunko(state: State<AppState>) -> Value {
    let config: serde_json::Value = std::fs::read_to_string(&state.config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(json!({}));
    let state_dir = config.get("state_dir").and_then(|v| v.as_str()).unwrap_or("");

    if state_dir.is_empty() {
        return json!({"ok": false, "error": "未配置状态目录"});
    }

    // Extract distro and WSL path from \\wsl.localhost\<Distro>\<path>\state
    let normalized = state_dir.replace('\\', "/");
    let parts: Vec<&str> = normalized.split('/').filter(|s| !s.is_empty()).collect();
    let distro = parts.iter().position(|&s| s == "wsl.localhost")
        .and_then(|i| parts.get(i + 1))
        .copied()
        .unwrap_or("Ubuntu");
    // Build WSL path: join parts after distro and strip trailing /state
    let wsl_path = if let Some(wsl_idx) = parts.iter().position(|&s| s == "wsl.localhost") {
        if wsl_idx + 2 < parts.len() {
            let joined = format!("/{}", parts[(wsl_idx + 2)..].join("/"));
            joined.strip_suffix("/state").unwrap_or(&joined).to_string()
        } else {
            "/root/yoshunko".to_string()
        }
    } else {
        "/root/yoshunko".to_string()
    };

    let cmd = format!(
        "cd {} && (zig build run-dpsv &) && sleep 2 && zig build run-gamesv", wsl_path
    );

    let lm = &state.log_manager;
    let (log_file, _log_name) = lm.create_log("yoshunko");

    let mut cmd_proc = std::process::Command::new("wsl");
    cmd_proc.args(&["-u", "root", "-d", distro, "-e", "bash", "-c", &cmd]);
    if let Ok(log_file2) = log_file.try_clone() {
        use std::process::Stdio;
        cmd_proc.stdout(Stdio::from(log_file));
        cmd_proc.stderr(Stdio::from(log_file2));
    }
    #[cfg(windows)]
    { cmd_proc.creation_flags(CREATE_NO_WINDOW); }
    match cmd_proc.spawn() {
        Ok(child) => {
            if let Ok(mut procs) = state.running_processes.lock() {
                procs.insert("yoshunko".to_string(), child.id());
            }
            json!({"ok": true, "distro": distro})
        }
        Err(e) => json!({"ok": false, "error": e.to_string()}),
    }
}

/// Get all currently running PIDs in a single tasklist call.
#[cfg(windows)]
fn get_all_running_pids() -> std::collections::HashSet<u32> {
    std::process::Command::new("tasklist")
        .args(&["/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
        .map(|o| {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout.lines()
                .filter_map(|line| {
                    // CSV format: "processname","pid",...
                    let parts: Vec<&str> = line.split(',').collect();
                    parts.get(1)
                        .and_then(|s| s.trim_matches('"').parse::<u32>().ok())
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(not(windows))]
fn get_all_running_pids() -> std::collections::HashSet<u32> {
    // On non-Windows, assume all tracked PIDs are alive (simplification)
    std::collections::HashSet::new()
}

// ─── Process Management ─────────────────────────────────

#[tauri::command]
pub fn get_running_processes(state: State<AppState>) -> Value {
    let mut procs = state.running_processes.lock().unwrap_or_else(|e| e.into_inner());
    // Batch-verify all PIDs with a single tasklist call
    let running_pids = get_all_running_pids();
    procs.retain(|key, pid| {
        if *pid == 1 {
            // Client sentinel: assume running until explicitly stopped
            key == "client"
        } else {
            running_pids.contains(pid)
        }
    });
    let map: serde_json::Map<String, Value> = procs.iter()
        .map(|(k, &pid)| (k.clone(), json!(pid)))
        .collect();
    json!({"processes": map})
}

#[tauri::command]
pub fn stop_process(state: State<AppState>, key: String) -> Value {
    let pid = {
        let procs = state.running_processes.lock().unwrap_or_else(|e| e.into_inner());
        procs.get(&key).copied()
    };
    #[cfg(windows)]
    {
        if key == "client" || key == "gale" || key == "velina" {
            // Game runs as admin — use ShellExecuteW(runas) to kill it
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::UI::Shell::ShellExecuteW;
            use windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE;

            let file_w: Vec<u16> = OsStr::new("taskkill").encode_wide().chain(Some(0)).collect();
            let verb_w: Vec<u16> = OsStr::new("runas").encode_wide().chain(Some(0)).collect();
            let args_w: Vec<u16> = OsStr::new("/IM ZenlessZoneZeroBeta.exe /F /T").encode_wide().chain(Some(0)).collect();
            unsafe {
                ShellExecuteW(std::ptr::null_mut(), verb_w.as_ptr(), file_w.as_ptr(), args_w.as_ptr(), std::ptr::null(), SW_HIDE);
            }
        } else if let Some(pid) = pid {
            let _ = std::process::Command::new("taskkill")
                .args(&["/PID", &pid.to_string(), "/F", "/T"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(pid) = pid {
            let _ = std::process::Command::new("kill").arg(pid.to_string()).spawn();
        }
    }
    if let Ok(mut procs) = state.running_processes.lock() {
        procs.remove(&key);
    }
    json!({"ok": true})
}
