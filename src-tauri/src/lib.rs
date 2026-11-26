pub mod fmod;

use fmod::{
    discover_compiler, execute_bake_bank, inspect_audio_file, load_bankbake_project,
    save_bankbake_project, AppConfig, AudioFileInfo, BakeBankRequest, BakeBankResult,
    BakeEventItem, CompilerInfo, LoadedProjectData,
};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct ConfigState {
    pub config: Mutex<AppConfig>,
    pub config_path: PathBuf,
}

fn resolve_config_path() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        let root = if cwd.ends_with("src-tauri") {
            cwd.parent().unwrap_or(&cwd).to_path_buf()
        } else {
            cwd
        };
        if root.join("src-tauri").exists() {
            return root.join("bankbake.config.json");
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            return exe_dir.join("bankbake.config.json");
        }
    }
    PathBuf::from("bankbake.config.json")
}

fn load_config_from_disk(path: &PathBuf) -> AppConfig {
    let bundled_exists = fmod::find_bundled_compiler().is_some();
    if let Ok(raw) = fs::read_to_string(path) {
        if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&raw) {
            cfg.mode = fmod::BakeMode::Standalone;
            cfg.is_debug_build = cfg!(debug_assertions);
            if !cfg!(debug_assertions) {
                cfg.keep_temp_project = false;
                if bundled_exists {
                    cfg.fmod_compiler_path = None;
                }
            }
            return cfg;
        }
    }
    let mut cfg = AppConfig::default();
    if let Ok(cwd) = std::env::current_dir() {
        let root = if cwd.ends_with("src-tauri") {
            cwd.parent().unwrap_or(&cwd).to_path_buf()
        } else {
            cwd
        };
        cfg.output_directory = Some(root.join("BakedBanks").to_string_lossy().to_string());
    }
    cfg
}

#[tauri::command]
fn get_app_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?.clone();
    let bundled_exists = fmod::find_bundled_compiler().is_some();
    cfg.is_debug_build = cfg!(debug_assertions);
    if !cfg!(debug_assertions) {
        cfg.keep_temp_project = false;
        if bundled_exists {
            cfg.fmod_compiler_path = None;
        }
    }
    Ok(cfg)
}

#[tauri::command]
fn save_app_config(mut new_config: AppConfig, state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    let bundled_exists = fmod::find_bundled_compiler().is_some();
    new_config.is_debug_build = cfg!(debug_assertions);
    if !cfg!(debug_assertions) {
        new_config.keep_temp_project = false;
        if bundled_exists {
            new_config.fmod_compiler_path = None;
        }
    }
    let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
    *cfg = new_config.clone();
    if let Ok(serialized) = serde_json::to_string_pretty(&new_config) {
        let _ = fs::write(&state.config_path, serialized);
    }
    Ok(new_config)
}

#[tauri::command]
fn check_fmod_compiler(
    custom_path: Option<String>,
    state: State<'_, ConfigState>,
) -> Result<CompilerInfo, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    let effective_path = custom_path
        .as_deref()
        .or(cfg.fmod_compiler_path.as_deref());
    Ok(discover_compiler(effective_path))
}

#[tauri::command]
fn inspect_audio_files(paths: Vec<String>) -> Result<Vec<AudioFileInfo>, String> {
    let mut infos = Vec::new();
    let allowed = ["wav", "ogg", "mp3", "flac", "aif", "aiff", "wma"];

    for p_str in paths {
        let path = PathBuf::from(&p_str);
        if path.is_dir() {
            for entry in walkdir::WalkDir::new(&path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let ep = entry.path();
                if ep.is_file() {
                    if let Some(ext) = ep.extension().and_then(|e| e.to_str()) {
                        if allowed.contains(&ext.to_lowercase().as_str()) {
                            if let Ok(info) = inspect_audio_file(&ep.to_string_lossy()) {
                                infos.push(info);
                            }
                        }
                    }
                }
            }
        } else if path.is_file() {
            match inspect_audio_file(&p_str) {
                Ok(info) => infos.push(info),
                Err(e) => return Err(e),
            }
        }
    }
    Ok(infos)
}

#[tauri::command]
fn read_audio_bytes(path: String) -> Result<Vec<u8>, String> {
    let p = PathBuf::from(&path);
    if !p.exists() || !p.is_file() {
        return Err(format!("Audio file not found: {}", path));
    }
    fs::read(&p).map_err(|e| format!("Failed to read audio file: {}", e))
}

#[tauri::command]
async fn bake_bank(
    request: BakeBankRequest,
    state: State<'_, ConfigState>,
) -> Result<BakeBankResult, String> {
    let cfg = {
        let guard = state.config.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    tauri::async_runtime::spawn_blocking(move || execute_bake_bank(&request, &cfg))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn save_project_file(
    bank_name: String,
    events: Vec<BakeEventItem>,
    destination_path: String,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let dest = PathBuf::from(&destination_path);
        save_bankbake_project(&bank_name, &events, &dest)?;
        Ok(dest.to_string_lossy().to_string())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
async fn load_project_file(path: String) -> Result<LoadedProjectData, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let p = PathBuf::from(&path);
        load_bankbake_project(&p)
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
fn list_sample_audio_files() -> Result<Vec<AudioFileInfo>, String> {
    let mut samples = Vec::new();
    let mut roots = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            roots.push(exe_dir.join("samples"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.join("samples"));
        if let Some(p) = cwd.parent() {
            roots.push(p.join("samples"));
        }
    }
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        if let Some(parent) = PathBuf::from(manifest_dir).parent() {
            roots.push(parent.join("samples"));
        }
    }

    let filenames = [
        "ui_cancel.ogg",
        "ui_okay.ogg",
        "weapons_explosion_small.ogg",
        "amb_bird_call_01.ogg",
    ];

    for root in roots {
        if root.exists() && root.is_dir() {
            for name in filenames {
                let candidate = root.join(name);
                if let Ok(info) = inspect_audio_file(&candidate.to_string_lossy()) {
                    samples.push(info);
                }
            }
            if !samples.is_empty() {
                break;
            }
        }
    }
    Ok(samples)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config_path = resolve_config_path();
    let initial_config = load_config_from_disk(&config_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(ConfigState {
            config: Mutex::new(initial_config),
            config_path,
        })
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            save_app_config,
            check_fmod_compiler,
            inspect_audio_files,
            read_audio_bytes,
            bake_bank,
            save_project_file,
            load_project_file,
            list_sample_audio_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
