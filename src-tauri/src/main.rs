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

use tauri::{
  CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
  SystemTraySubmenu,
};

use crate::utils::get_current_time;

fn main() {
  let toggle_activate = CustomMenuItem::new("toggle_activate".to_string(), "Activate");
  let quit = CustomMenuItem::new("quit".to_string(), "Quit");

  let seconds_options = vec![5, 10, 15, 30, 60];
  let mut kapture_menu = SystemTrayMenu::new();
  for seconds_option in &seconds_options {
    let menu_item = CustomMenuItem::new(
      format!("kapture_seconds_{}", seconds_option),
      format!("{} Seconds", seconds_option),
    );

    kapture_menu = kapture_menu.add_item(menu_item.disabled());
  }

  let toggle_kapture_menu_activation = move |app: &tauri::AppHandle, enabled: bool| {
    for seconds_option in &seconds_options {
      app
        .tray_handle()
        .get_item(&format!("kapture_seconds_{}", seconds_option))
        .set_enabled(enabled)
        .expect("Failed to set enabled");
    }
  };

  let kapture_submenu = SystemTraySubmenu::new("Kapture", kapture_menu);

  let tray_menu = SystemTrayMenu::new()
    .add_item(toggle_activate)
    .add_submenu(kapture_submenu)
    .add_native_item(SystemTrayMenuItem::Separator)
    .add_item(quit);

  let system_tray = SystemTray::new().with_menu(tray_menu);

  tauri::Builder::default()
    .system_tray(system_tray)
    .on_system_tray_event(move |app, event| {
      match event {
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

                app
                  .emit_all("kapt_activation_toggled", false)
                  .expect("Failed to emit event");

                item_handle
                  .set_title("Activate")
                  .expect("Failed to set menu title");

                toggle_kapture_menu_activation(app, false);
              } else {
                tauri::async_runtime::spawn(async {
                  recording::activate_kapt(&*KAPT_STATE).await;
                });

                app
                  .emit_all("kapt_activation_toggled", true)
                  .expect("Failed to emit event");

                item_handle
                  .set_title("Deactivate")
                  .expect("Failed to set menu title");

                toggle_kapture_menu_activation(app, true);
              }
            }
            "quit" => {
              std::process::exit(0);
            }
            id => {
              if id.starts_with("kapture_seconds") {
                let seconds = id
                  .replace("kapture_seconds_", "")
                  .parse::<u32>()
                  .expect("Failed to parse");

                let timestamp = get_current_time();
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                  let video_path = kapture::create_kapture(&*KAPT_STATE, timestamp, seconds).await;
                  println!("caputer created");
                  app_handle
                    .emit_all("kapture_created", video_path)
                    .expect("Failed to emit event");
                })
              }
            }
          }
        }
        _ => {}
      }
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
