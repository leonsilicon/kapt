#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::sync::RwLock;

use tauri::api::process::{Command, CommandChild, CommandEvent};

struct KaptState {
  pub active_recordings: [Option<FfmpegActiveRecording>; 2],

  // Used to prevent a thread from continuing a chunk recording when the
  // associated recording session has ended
  pub recording_session_id: Option<String>,
}

impl KaptState {
  pub fn is_recording(&self) -> bool {
    self.active_recordings[0].is_some() || self.active_recordings[1].is_some()
  }
}

struct FfmpegActiveRecording {
  pub path: String,
  pub command_child: CommandChild,
}

use nanoid::nanoid;
use std::env;
use std::path::Path;

fn stop_recording_chunk(state_lock: &'static RwLock<KaptState>, recording_index: usize) {
  let mut state = state_lock.write().expect("Failed to acquire write lock");

  if let Some(mut recording_child) = Option::take(&mut state.active_recordings[recording_index]) {
    recording_child
      .command_child
      .write(&[b'q'])
      .expect("Failed to stop ffmpeg process");

    println!("Recording 0 path: {}", recording_child.path);
  }
}

fn start_recording_chunk(
  window: tauri::Window,
  state_lock: &'static RwLock<KaptState>,
  recording_index: usize,
) {
  let is_chunk_active = {
    let state = state_lock
      .read()
      .expect("Failed to acquire state read lock");
    state.active_recordings[recording_index].is_some()
  };

  if is_chunk_active {
    stop_recording_chunk(state_lock, recording_index)
  }

  // If the current recording index is taken, stop it

  let mut command =
    Command::new_sidecar("ffmpeg").expect("failed to create `ffmpeg` binary command");

  command = command.args(&["-video_size", "1024x768"]);
  command = command.args(&["-framerate", "25"]);
  command = command.args(&["-f", "x11grab"]);
  command = command.args(&["-i", ":0.0+100,200"]);

  let path = Path::new(&env::temp_dir())
    .join(format!("{}.mp4", nanoid!()))
    .to_string_lossy()
    .to_string();

  // Adding the .mp4 path to the command
  command = command.args(&[&path]);

  let (mut rx, command_child) = command.spawn().expect("Failed to spawn ffmpeg");

  println!("Ffmpeg process spawned...");

  let mut state = state_lock
    .write()
    .expect("Failed to acquire state write lock");
  state.active_recordings[0] = Some(FfmpegActiveRecording {
    command_child,
    path,
  });

  let window_clone = window.clone();
  // Spawn a recording chunk for right now
  tauri::async_runtime::spawn(async move {
    let window = window_clone;
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

  use tokio::time::{sleep, Duration};
  // Spawn a recording chunk for 5 seconds in the future
  tauri::async_runtime::spawn(async move {
    sleep(Duration::from_secs(5)).await;
    start_recording_chunk(window, state_lock, if recording_index == 0 { 1 } else { 0 });
  });
}

#[tauri::command]
fn start_recording(
  window: tauri::Window,
  state_lock: tauri::State<&'static RwLock<KaptState>>,
) -> tauri::Result<()> {
  {
    let state = state_lock.read().expect("Failed to acquire write lock");

    if state.is_recording() {
      println!("Recording has already been started.");
      return Ok(());
    }
  }

  println!("Starting the recording...");

  // Generating a recording session ID
  {
    let mut state = state_lock.write().expect("Failed to acquire write lock");
    state.recording_session_id = Some(nanoid!());
  }

  start_recording_chunk(window.clone(), *state_lock, 0);

  Ok(())
}

#[tauri::command]
fn stop_recording(state_lock: tauri::State<&'static RwLock<KaptState>>) {
  {
    let state = state_lock.read().expect("Failed to acquire write lock");

    if state.is_recording() {
      println!("There is no recording in process.");
      return;
    }
  }

  println!("Stopping the recording...");
  stop_recording_chunk(*state_lock, 0);
  stop_recording_chunk(*state_lock, 1);
}

use lazy_static::lazy_static;
lazy_static! {
  static ref KAPT_STATE: RwLock<KaptState> = RwLock::new(KaptState {
    recording_session_id: None,
    active_recordings: [None, None],
  });
}

fn main() {
  tauri::Builder::default()
    .manage(&*KAPT_STATE)
    .invoke_handler(tauri::generate_handler![start_recording, stop_recording])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
