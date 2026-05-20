fn main() {
    // Read version from tauri.conf.json and make it available at compile time
    let conf_path = std::path::Path::new("tauri.conf.json");
    if let Ok(content) = std::fs::read_to_string(conf_path) {
        if let Ok(conf) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(ver) = conf.get("version").and_then(|v| v.as_str()) {
                println!("cargo:rustc-env=APP_VERSION={}", ver);
            }
        }
    }
    tauri_build::build()
}
