#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audio;
mod kapture;
mod recording;
mod state;
mod utils;

use audio::AudioSource;
use lazy_static::lazy_static;
use state::KaptState;
use std::{path::PathBuf, sync::RwLock};
lazy_static! {
  static ref KAPT_STATE: RwLock<KaptState> = RwLock::new(KaptState::new());
}

#[tauri::command]
async fn deactivate_kapt() {
  recording::deactivate_kapt(&*KAPT_STATE).await;
}

#[tauri::command]
async fn activate_kapt() {
  recording::activate_kapt(&*KAPT_STATE).await;
}

#[tauri::command]
// timestamp - Unix timestamp of when the user pressed the Kapture button (in seconds)
async fn create_kapture(timestamp: i64, seconds_to_capture: i64) -> String {
  kapture::create_kapture(&*KAPT_STATE, timestamp as u128, seconds_to_capture as u32).await
}

#[tauri::command]
fn set_audio_source(audio_source: usize) {
  let mut state = &mut *KAPT_STATE.write().expect("Failed to get write lock");
  state.audio_source = audio_source;
}

#[tauri::command]
fn set_video_folder(video_folder: String) {
  let mut state = &mut *KAPT_STATE.write().expect("Failed to get write lock");
  state.video_folder = Some(video_folder);
}

#[tauri::command]
fn get_audio_sources() -> Vec<AudioSource> {
  audio::get_audio_sources()
}

#[tauri::command]
fn select_video_folder() -> Option<PathBuf> {
  use tauri::api::dialog::FileDialogBuilder;

  FileDialogBuilder::new().pick_folder()
}

#[tauri::command]
fn set_max_seconds_cached(seconds: u32) {
  let mut state = &mut *KAPT_STATE.write().expect("Failed to get write lock");
  state.max_seconds_cached = seconds;
}

use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
fn create_system_tray() -> SystemTray<String> {
  let toggle_activate = CustomMenuItem::new("toggle_activate".to_string(), "Activate");
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");

  let tray_menu = SystemTrayMenu::new()
    .add_item(toggle_activate)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(quit);

  SystemTray::new().with_menu(tray_menu)
}

fn main() {
  let system_tray = create_system_tray();

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => {
        let item_handle = app.tray_handle().get_item(&id);

        match id.as_str() {
          "toggle_activate" => {
            let state = &*KAPT_STATE.read().expect("Failed to get read lock");

            // If Kapt is active, deactivate it
            if state.is_active() {
              tauri::async_runtime::spawn(async {
                recording::deactivate_kapt(&*KAPT_STATE).await;
              });

              item_handle
                .set_title("Activate")
                .expect("Failed to set menu title");
            } else {
              tauri::async_runtime::spawn(async {
                recording::activate_kapt(&*KAPT_STATE).await;
              });

              item_handle
                .set_title("Deactivate")
                .expect("Failed to set menu title");
            }
          }
          _ => {}
        }
      }
      _ => {}
    })
    .manage(&*KAPT_STATE)
    .invoke_handler(tauri::generate_handler![
      activate_kapt,
      deactivate_kapt,
      create_kapture,
      get_audio_sources,
      set_audio_source,
      select_video_folder,
      set_video_folder,
      set_max_seconds_cached
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
