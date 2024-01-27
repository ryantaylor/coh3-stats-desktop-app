#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

extern crate machine_uid;
use tauri_plugin_single_instance;
use tauri_plugin_fs_watch;
use std::path::Path;
use tauri::{command, Manager, Window};
use window_shadows::set_shadow;
use tauri_plugin_log::{LogTarget};
use tauri_plugin_oauth::{start_with_config, OauthConfig};
mod parse_log_file;

#[derive(Clone, serde::Serialize)]
struct Payload {
  args: Vec<String>,
  cwd: String,
}

#[command]
async fn start_server(window: Window) -> Result<u16, String> {
    let mut config = OauthConfig::default();
    config.ports = Some(vec![6969]);
    start_with_config(config, move |url| {
        // Because of the unprotected localhost port, you must verify the URL here.
        // Preferably send back only the token, or nothing at all if you can handle everything else in Rust.
        println!("{}", url);
        let _ = window.emit("oauth://url", url);
    })
        .map_err(|err| err.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_default_log_file_path, check_log_file_exists, get_machine_id, parse_log_file::parse_log_file_reverse, start_server])
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            let window = app.get_window("main").unwrap();
            window.set_focus().ok();
            window.request_user_attention(Some(tauri::UserAttentionType::Informational)).ok();
            
            //println!("{}, {argv:?}, {cwd}", app.package_info().name);

            app.emit_all("single-instance", Payload { args: argv, cwd }).unwrap();
        }))
        .plugin(tauri_plugin_log::Builder::default().targets([
            LogTarget::LogDir,
            LogTarget::Stdout,
        ]).build())
        .plugin(tauri_plugin_fs_watch::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| { // Add window shadows
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// returns the default expected log file path
#[tauri::command]
fn get_default_log_file_path() -> String {
    let mut path = tauri::api::path::document_dir().unwrap();
    path.push("My Games");
    path.push("Company of Heroes 3");
    path.push("warnings.log");
    path.display().to_string()
}

/// checks if log file can be found on system
#[tauri::command]
fn check_log_file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// get the system machine id
#[tauri::command]
fn get_machine_id() -> String {
    machine_uid::get().unwrap()
}
