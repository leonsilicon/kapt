#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod audio;
mod recording;
mod state;

use lazy_static::lazy_static;
use state::KaptState;
use std::sync::RwLock;
lazy_static! {
  static ref KAPT_STATE: RwLock<KaptState> = RwLock::new(KaptState::new());
}

fn main() {
  tauri::Builder::default()
    .manage(&*KAPT_STATE)
    .invoke_handler(tauri::generate_handler![
      recording::start_recording,
      recording::stop_recording,
      audio::get_audio_sources
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
