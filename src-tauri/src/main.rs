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
  recording::start_recording(&*KAPT_STATE).await;
}

#[tauri::command]
// timestamp - Unix timestamp of when the user pressed the Kapture button (in seconds)
async fn create_kapture(timestamp: i64) -> String {
  kapture::create_kapture(&*KAPT_STATE, timestamp as u128).await
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
fn set_seconds_to_capture(seconds: u32) {
  let mut state = &mut *KAPT_STATE.write().expect("Failed to get write lock");
  state.seconds_to_capture = seconds;
}

fn main() {
  tauri::Builder::default()
    .manage(&*KAPT_STATE)
    .invoke_handler(tauri::generate_handler![
      activate_kapt,
      deactivate_kapt,
      create_kapture,
      get_audio_sources,
      set_audio_source,
      select_video_folder,
      set_video_folder,
      set_seconds_to_capture
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
