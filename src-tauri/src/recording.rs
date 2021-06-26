use crate::recording;
use crate::state::FfmpegActiveRecording;
use crate::state::FfmpegRecording;
use crate::state::KaptState;
use lazy_static::lazy_static;
use nanoid::nanoid;
use std::env;
use std::path::Path;
use std::sync::RwLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tauri::api::process::Command;
use tauri::api::process::CommandEvent;

pub fn stop_recording_chunk(state_lock: &'static RwLock<KaptState>, recording_index: usize) {
  let mut state = state_lock.write().expect("Failed to acquire write lock");

  let mut active_recording = Option::take(&mut state.active_recordings[recording_index]);
  if let Some(active_recording) = &mut active_recording {
    println!("Stopping the recording chunk...");
    // Stopping the ffmpeg recording process
    active_recording
      .command_child
      .write(&[b'q'])
      .expect("Failed to stop ffmpeg process");

    println!(
      "Recording {} path: {}",
      recording_index, active_recording.path
    );
  }
}

pub fn start_recording_chunk(
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
  command = command.args(&["-i", ":0.0"]);

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
  state.active_recordings[recording_index] = Some(FfmpegActiveRecording {
    command_child,
    path,
    start_time: None,
  });

  let window_clone = window.clone();
  // Spawn a recording chunk for right now
  tauri::async_runtime::spawn(async move {
    let window = window_clone;
    let mut start_time: Option<u128> = None;

    while let Some(event) = rx.recv().await {
      println!("{:?}", event);
      // Ffmpeg logs to stderr
      if let CommandEvent::Stderr(line) = event {
        use regex::Regex;
        lazy_static! {
          static ref START_TIME_RE: Regex =
            Regex::new(r#"start: (\d+)\."#).expect("Failed to compile regex");
        };

        if let Some(cap) = START_TIME_RE.captures(&line) {
          if let Some(m) = cap.get(1) {
            let unix_timestamp = m
              .as_str()
              .to_string()
              .parse::<u128>()
              .expect("Failed to parse integer");
            start_time = Some(unix_timestamp);
          }

          println!("{}", line);
          window
            .emit("message", Some(format!("'{}'", line)))
            .expect("failed to emit event");
        }
      }
    }

    // Ffmpeg process ended
    let end_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis();

    let mut state = state_lock
      .write()
      .expect("Failed to acquire state read lock");

    let recording_session_id = state
      .recording_session_id
      .clone()
      .expect("Session ID not present");

    let path = state.active_recordings[recording_index]
      .as_ref()
      .unwrap()
      .path
      .clone();

    // Adding a past recording
    state.add_recording(
      recording_session_id.clone(),
      FfmpegRecording {
        end_time,
        path: path.to_string(),
        start_time: start_time.expect("Start time not present"),
      },
    );
  });

  use tokio::time::{sleep, Duration};
  // Spawn a recording chunk for 5 seconds in the future
  tauri::async_runtime::spawn(async move {
    sleep(Duration::from_secs(5)).await;
    start_recording_chunk(window, state_lock, if recording_index == 0 { 1 } else { 0 });
  });
}

#[tauri::command]
pub fn start_recording(
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

  recording::start_recording_chunk(window.clone(), *state_lock, 0);

  Ok(())
}

#[tauri::command]
pub fn stop_recording(state_lock: tauri::State<&'static RwLock<KaptState>>) {
  {
    let state = state_lock.read().expect("Failed to acquire write lock");

    if state.is_recording() {
      println!("There is no recording in process.");
      return;
    }
  }

  println!("Stopping the recording...");
  recording::stop_recording_chunk(*state_lock, 0);
  recording::stop_recording_chunk(*state_lock, 1);
}
