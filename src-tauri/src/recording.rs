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
      .video_command_child
      .write(&[b'q'])
      .expect("Failed to stop ffmpeg video process");

    active_recording
      .audio_command_child
      .write(&[b'q'])
      .expect("Failed to stop ffmpeg audio process");

    println!(
      "Recording {}; Video path: {}; Audio path: {}",
      recording_index, active_recording.video_path, active_recording.audio_path
    );
  }
}

pub fn start_recording_chunk(
  window: tauri::Window,
  state_lock: &'static RwLock<KaptState>,
  recording_index: usize,
) {
  let recording_session_id = {
    let state = state_lock
      .read()
      .expect("Failed to acquire state write lock");

    state.recording_session_id.clone()
  };

  let recording_session_id = match recording_session_id {
    Some(session_id) => session_id,
    None => return,
  };

  let is_chunk_active = {
    let state = state_lock
      .read()
      .expect("Failed to acquire state read lock");
    state.active_recordings[recording_index].is_some()
  };

  // If the current recording index is taken, stop it
  if is_chunk_active {
    stop_recording_chunk(state_lock, recording_index)
  }

  let video_path = Path::new(&env::temp_dir())
    .join(format!("{}.mp4", nanoid!()))
    .to_string_lossy()
    .to_string();

  // Recording the video
  let (mut video_rx, video_command_child) = {
    let mut command = Command::new("ffmpeg");

    // Video
    command = command.args(&["-video_size", "2560x1440"]);
    command = command.args(&["-framerate", "25"]);
    command = command.args(&["-f", "x11grab"]);
    command = command.args(&["-i", ":0.0"]);

    // Adding the .mp4 path to the command
    command = command.args(&[&video_path]);

    command.spawn().expect("Failed to spawn ffmpeg")
  };

  let audio_path = Path::new(&env::temp_dir())
    .join(format!("{}.wav", nanoid!()))
    .to_string_lossy()
    .to_string();

  // Recording the audio
  let (mut audio_rx, audio_command_child) = {
    let mut command = Command::new("ffmpeg");

    // Video
    command = command.args(&["-f", "pulse"]);
    command = command.args(&["-i", "4"]);
    command = command.args(&["-fflags", "+genpts"]);
    command = command.args(&["-async", "1"]);

    // Adding the .mp4 path to the command
    command = command.args(&[&audio_path]);

    command.spawn().expect("Failed to spawn ffmpeg")
  };

  println!("Ffmpeg process spawned...");

  {
    let mut state = state_lock
      .write()
      .expect("Failed to acquire state write lock");
    state.active_recordings[recording_index] = Some(FfmpegActiveRecording {
      video_command_child,
      video_path: video_path.clone(),
      audio_command_child,
      audio_path: audio_path.clone(),
      video_start_time: None,
      audio_start_time: None,
    });
  };

  {
    let recording_session_id = recording_session_id.clone();
    // Spawn a recording chunk for right now
    tauri::async_runtime::spawn(async move {
      let mut video_start_time: Option<u128> = None;
      let mut audio_start_time: Option<u128> = None;

      let parse_start_time = |line: String| {
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

            return Some(unix_timestamp);
          }
        }

        None
      };

      while let Some(event) = video_rx.recv().await {
        println!("{:?}", event);
        // Ffmpeg logs to stderr
        if let CommandEvent::Stderr(line) = event {
          if let Some(unix_timestamp) = parse_start_time(line) {
            video_start_time = Some(unix_timestamp);
          }
        }
      }

      while let Some(event) = audio_rx.recv().await {
        // Ffmpeg logs to stderr
        println!("{:?}", event);
        if let CommandEvent::Stderr(line) = event {
          if let Some(unix_timestamp) = parse_start_time(line) {
            audio_start_time = Some(unix_timestamp);
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

      state.add_recording(
        recording_session_id,
        FfmpegRecording {
          audio_path,
          audio_start_time: audio_start_time.expect("Audio start time not found."),
          video_path,
          video_start_time: video_start_time.expect("Video start not not found."),
          end_time,
        },
      );
    });
  }

  use tokio::time::{sleep, Duration};
  // Spawn a recording chunk for 5 seconds in the future
  tauri::async_runtime::spawn(async move {
    sleep(Duration::from_secs(5)).await;
    // Check if the session ID is most recent
    let current_recording_session_id = {
      let state = state_lock
        .read()
        .expect("Failed to acquire state read lock");

      state.recording_session_id.clone()
    };

    if current_recording_session_id == Some(recording_session_id) {
      start_recording_chunk(window, state_lock, if recording_index == 0 { 1 } else { 0 });
    }
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

    if !state.is_recording() {
      println!("There is no recording in process.");
      return;
    }
  }

  {
    let mut state = state_lock.write().expect("Failed to acquire write lock");
    state.recording_session_id = None;
  }

  println!("Stopping the recording...");
  recording::stop_recording_chunk(*state_lock, 0);
  recording::stop_recording_chunk(*state_lock, 1);
}
