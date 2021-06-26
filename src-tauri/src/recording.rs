use crate::recording;
use crate::state::FfmpegActiveRecording;
use crate::state::KaptState;
use crate::utils::create_temp_path;
use nanoid::nanoid;
use std::sync::RwLock;
use tauri::api::process::Command;

pub async fn stop_recording_chunk(state_lock: &'static RwLock<KaptState>, recording_index: usize) {
  let mut active_recording = {
    let mut state = state_lock.write().expect("Failed to acquire write lock");

    Option::take(&mut state.active_recordings[recording_index])
  };

  if let Some(active_recording) = &mut active_recording {
    println!("Stopping the recording chunk...");
    active_recording.stop(state_lock).await;

    println!(
      "Recording {}; Video path: {}; Audio path: {}",
      recording_index, active_recording.video_path, active_recording.audio_path
    );
  }
}

pub async fn start_recording_chunk(state_lock: &'static RwLock<KaptState>, recording_index: usize) {
  let is_chunk_active = {
    let state = state_lock
      .read()
      .expect("Failed to acquire state read lock");
    state.active_recordings[recording_index].is_some()
  };

  // If the current recording index is taken, stop it
  if is_chunk_active {
    stop_recording_chunk(state_lock, recording_index).await;
  }

  let video_path = create_temp_path(&format!("{}.mp4", nanoid!()));

  // Recording the video
  let (video_rx, video_command_child) = {
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

  let audio_path = create_temp_path(&format!("{}.wav", nanoid!()));

  // Recording the audio
  let (audio_rx, audio_command_child) = {
    let mut command = Command::new("ffmpeg");

    // Video
    command = command.args(&["-f", "pulse"]);
    command = command.args(&["-i", "6"]);
    command = command.args(&["-fflags", "+genpts"]);
    command = command.args(&["-async", "1"]);

    // Adding the .wav path to the command
    command = command.args(&[&audio_path]);
    command = command.args(&["-y"]);

    command.spawn().expect("Failed to spawn ffmpeg")
  };

  println!("Ffmpeg process spawned...");

  {
    let mut state = state_lock
      .write()
      .expect("Failed to acquire state write lock");
    state.active_recordings[recording_index] = Some(FfmpegActiveRecording {
      video_command_child,
      video_rx,
      video_path: video_path.clone(),
      video_start_time: None,
      audio_command_child,
      audio_rx,
      audio_path: audio_path.clone(),
      audio_start_time: None,
    });
  };
}

pub async fn start_recording(state_lock: &'static RwLock<KaptState>) {
  {
    let state = state_lock.read().expect("Failed to acquire write lock");

    if state.is_recording() {
      println!("Recording has already been started.");
    }
  }

  println!("Starting the recording...");
  let recording_session_id = nanoid!();

  // Generating a recording session ID
  {
    let mut state = state_lock.write().expect("Failed to acquire write lock");
    state.recording_session_id = Some(recording_session_id.clone());
  }

  let mut recording_index = 0;
  recording::start_recording_chunk(state_lock, recording_index).await;
  use tokio::time::{sleep, Duration};

  // Spawn a recording chunk for 5 seconds in the future
  tauri::async_runtime::spawn(async move {
    loop {
      sleep(Duration::from_secs(5)).await;
      recording_index = if recording_index == 0 { 1 } else { 0 };
      // Check if the session ID is most recent
      let current_recording_session_id = {
        let state = state_lock
          .read()
          .expect("Failed to acquire state read lock");

        state.recording_session_id.clone()
      };

      if let Some(current_recording_session_id) = current_recording_session_id {
        if current_recording_session_id == recording_session_id {
          start_recording_chunk(state_lock, recording_index).await;
        }
      }
    }
  });
}

pub async fn stop_recording(state_lock: &'static RwLock<KaptState>) {
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
  recording::stop_recording_chunk(state_lock, 0).await;
  recording::stop_recording_chunk(state_lock, 1).await;
}
