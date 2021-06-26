#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::RwLock;

use tauri::api::process::{Command, CommandChild, CommandEvent};

struct KaptState {
  pub is_recording: bool,
  pub recording_child: Option<CommandChild>,
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

  let mut command =
    Command::new_sidecar("ffmpeg").expect("failed to create `ffmpeg` binary command");

  command = command.args(&["-video_size", "1024x768"]);
  command = command.args(&["-framerate", "25"]);
  command = command.args(&["-f", "x11grab"]);
  command = command.args(&["-i", ":0.0+100,200"]);
  command = command.args(&["/home/leonzalion/output.mp4"]);

  let (mut rx, recording_child) = command.spawn().expect("Failed to spawn ffmpeg");

  println!("Ffmpeg process spawned...");

  state.recording_child = Some(recording_child);

  tauri::async_runtime::spawn(async move {
    // read events such as stdout
    while let Some(event) = rx.recv().await {
      println!("{:?}", event);
      if let CommandEvent::Stdout(line) = event {
        println!("{}", line);
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

  let recording_child = Option::take(&mut state.recording_child);

  recording_child
    .unwrap()
    .write(&[b'q'])
    .expect("Failed to kill ffmpeg process");

  state.is_recording = false;
}

fn main() {
  let kapt_state = KaptState {
    is_recording: false,
    recording_child: None,
  };

  tauri::Builder::default()
    .manage(RwLock::new(kapt_state))
    .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
