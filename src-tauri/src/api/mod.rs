// Tauri command handlers — JS Bridge API
// Returns data directly (no wrapper), matching Python api.py behavior

use crate::data_manager::DataManager;
use crate::template_loader::TemplateLoader;
use crate::zon::ZonValue;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Mutex;
use tauri::State;

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct AppState {
    pub data_manager: Mutex<Option<DataManager>>,
    pub template_loader: TemplateLoader,
    pub config_path: String,
    pub cached_templates: std::sync::OnceLock<Value>,
    pub log_manager: crate::log_manager::LogManager,
    pub running_processes: std::sync::Arc<Mutex<std::collections::HashMap<String, u32>>>,
}

// Submodules
mod helpers;
mod config;
mod window;
mod image;
#[cfg(debug_assertions)]
mod debug;
mod templates;
mod players;
mod avatars;
mod weapons;
mod equips;
mod hadal;
mod launch;
mod logs;

// Re-export all public items
pub use helpers::*;
pub use config::*;
pub use window::*;
pub use image::*;
#[cfg(debug_assertions)]
#[allow(unused_imports)]
pub use debug::*;
pub use templates::*;
pub use players::*;
pub use avatars::*;
pub use weapons::*;
pub use equips::*;
pub use hadal::*;
pub use launch::*;
pub use logs::*;
