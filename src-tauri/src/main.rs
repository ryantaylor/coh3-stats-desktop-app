#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

extern crate machine_uid;
use coh3_stats_desktop_app::logfile::{parse_logfile_path, LogfileState};
use coh3_stats_desktop_app::parse_log_file;
use notify::event::ModifyKind::Data;
use notify::Watcher;
use notify::{Config, EventKind, FsEventWatcher, RecommendedWatcher, RecursiveMode};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use tauri::Manager;
use tauri_plugin_log::LogTarget;
use window_shadows::set_shadow;

#[derive(Clone, serde::Serialize)]
struct Payload {
    args: Vec<String>,
    cwd: String,
}

struct State {
    logfile: Arc<Mutex<LogfileState>>,
    watcher: Mutex<FsEventWatcher>,
}

impl State {
    pub fn new(log_path: PathBuf) -> State {
        let logfile = Arc::new(Mutex::new(LogfileState::default()));

        State {
            watcher: Mutex::new(watch_logfile(log_path, logfile.clone()).unwrap()),
            logfile,
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(State::new(PathBuf::from(
            "/Users/ryantaylor/Downloads/warnings.log".to_string(),
        )))
        .invoke_handler(tauri::generate_handler![
            get_default_log_file_path,
            check_log_file_exists,
            get_machine_id,
            parse_log_file::parse_log_file_reverse
        ])
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            let window = app.get_window("main").unwrap();
            window.set_focus().ok();
            window
                .request_user_attention(Some(tauri::UserAttentionType::Informational))
                .ok();

            //println!("{}, {argv:?}, {cwd}", app.package_info().name);

            app.emit_all("single-instance", Payload { args: argv, cwd })
                .unwrap();
        }))
        .plugin(
            tauri_plugin_log::Builder::default()
                .targets([LogTarget::LogDir, LogTarget::Stdout])
                .build(),
        )
        .plugin(tauri_plugin_fs_watch::init())
        // .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Add window shadows
            let window = app.get_window("main").unwrap();
            set_shadow(&window, true).expect("Unsupported platform!");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn watch_logfile(path: PathBuf, log: Arc<Mutex<LogfileState>>) -> notify::Result<FsEventWatcher> {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();

    watcher.watch(&path, RecursiveMode::Recursive).unwrap();

    spawn(move || {
        for res in rx {
            match res {
                Ok(event) => {
                    if let EventKind::Modify(Data(_)) = event.kind {
                        let log_state = parse_logfile_path(&path);

                        log.lock().unwrap().merge(log_state);
                    }
                }
                Err(error) => println!("Error: {error:?}"),
            }
        }
    });

    Ok(watcher)
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
