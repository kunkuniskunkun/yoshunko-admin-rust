mod api;
mod data_manager;
mod template_loader;
mod zon;

use api::AppState;
use data_manager::DataManager;
use std::fs;
use template_loader::TemplateLoader;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Determine data/config paths
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    // Config: use exe_dir in release, src-tauri/ in dev
    let config_path = if exe_dir.join("config.json").exists() || !cfg!(debug_assertions) {
        exe_dir.join("config.json").to_string_lossy().to_string()
    } else {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("config.json").to_string_lossy().to_string()
    };

    // Data dir: try exe_dir/data first, fall back to src-tauri/data (for cargo tauri dev)
    let exe_data = exe_dir.join("data");
    let data_dir = if exe_data.is_dir() {
        exe_data
    } else {
        // During development, data is next to Cargo.toml
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("data")
    };

    // Try to load existing config
    let dm = if let Ok(content) = fs::read_to_string(&config_path) {
        if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&content) {
            cfg.get("state_dir")
                .and_then(|v| v.as_str())
                .filter(|p| !p.is_empty())
                .and_then(|p| {
                    let player_dir = std::path::Path::new(p).join("player");
                    if player_dir.is_dir() { Some(DataManager::new(p)) } else { None }
                })
        } else { None }
    } else { None };

    let tl = TemplateLoader::new(data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Set high-resolution window icon for clear taskbar display
            let icon_bytes = include_bytes!("../icons/256x256.png");
            if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon);
                }
            }
            Ok(())
        })
        .manage(AppState {
            data_manager: std::sync::Mutex::new(dm),
            template_loader: tl,
            config_path,
            cached_templates: std::sync::OnceLock::new(),
        })
        .invoke_handler(tauri::generate_handler![
            api::get_config,
            api::get_version,
            api::set_state_dir,
            api::auto_detect_paths,
            api::debug_list_dir,
            api::debug_avatar_ids,
            api::get_templates,
            api::window_minimize,
            api::window_toggle_max,
            api::window_close,
            api::get_player_list,
            api::get_player_basic,
            api::update_player_basic,
            api::get_avatars,
            api::get_avatar,
            api::update_avatar,
            api::get_weapons,
            api::get_weapon,
            api::update_weapon,
            api::get_equips,
            api::get_equip,
            api::update_equip,
            api::create_equip,
            api::delete_equip,
            api::get_hadal_zone,
            api::update_hadal_zone,
            api::get_launch_config,
            api::set_launch_path,
            api::launch_program,
            api::launch_program_admin,
            api::launch_yoshunko,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
