#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audio;
mod kapture;
mod recording;
mod state;
mod utils;

use lazy_static::lazy_static;
use state::KaptState;
use std::sync::RwLock;
lazy_static! {
  static ref KAPT_STATE: RwLock<KaptState> = RwLock::new(KaptState::new());
}

#[tauri::command]
fn stop_recording(state_lock: tauri::State<&'static RwLock<KaptState>>) {
  recording::stop_recording(*state_lock);
}

#[tauri::command]
fn start_recording(state_lock: tauri::State<&'static RwLock<KaptState>>) {
  recording::start_recording(*state_lock);
}

fn main() {
  tauri::Builder::default()
    .manage(&*KAPT_STATE)
    .invoke_handler(tauri::generate_handler![
      start_recording,
      stop_recording,
      audio::get_audio_sources
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
