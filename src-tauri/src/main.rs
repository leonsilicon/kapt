#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::RwLock;

use tauri::api::process::{Command, CommandEvent};

struct KaptState {
  pub is_recording: bool,
}

#[tauri::command]
fn start_recording(
  window: tauri::Window,
  state: tauri::State<RwLock<KaptState>>,
) -> tauri::Result<()> {
  let mut state = state.write().expect("Failed to acquire write lock");

  if state.is_recording {
    println!("Recording has already been started.");
    return Ok(());
  }

  println!("Starting the recording...");
  state.is_recording = true;

  let (mut rx, _child) = Command::new_sidecar("ffmpeg")
    .expect("failed to create `ffmpeg` binary command")
    .args(&["-version"])
    .spawn()
    .expect("Failed to spawn sidecar");

  tauri::async_runtime::spawn(async move {
    // read events such as stdout
    while let Some(event) = rx.recv().await {
      if let CommandEvent::Stdout(line) = event {
        window
          .emit("message", Some(format!("'{}'", line)))
          .expect("failed to emit event");
      }
    }
  });

  Ok(())
}

#[tauri::command]
fn stop_recording(state: tauri::State<RwLock<KaptState>>) {
  let mut state = state.write().expect("Failed to acquire write lock");

  if state.is_recording == false {
    println!("There is no recording in process.");
    return;
  }

  println!("Stopping the recording...");
  state.is_recording = false
}

fn main() {
  let kapt_state = KaptState {
    is_recording: false
  };

  tauri::Builder::default()
    .manage(RwLock::new(kapt_state))
    .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
