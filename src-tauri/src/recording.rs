use crate::recording;
use crate::state::FfmpegActiveRecording;
use crate::state::KaptState;
use crate::utils::create_temp_path;
use crate::utils::get_current_time;
use nanoid::nanoid;
use std::collections::VecDeque;
use std::sync::RwLock;
use tauri::api::process::Command;

pub async fn stop_recording_chunk(state_lock: &'static RwLock<KaptState>, recording_index: usize) {
  let mut active_recording = {
    let mut state = state_lock.write().expect("Failed to acquire write lock");

    Option::take(&mut state.active_recordings[recording_index])
  };

  if let Some(active_recording) = &mut active_recording {
    active_recording.stop(state_lock).await;

    println!(
      "Recording {}; Video path: {}; Audio path: {}",
      recording_index, active_recording.video_path, active_recording.audio_path
    );
  }
}

pub async fn start_recording_chunk(state_lock: &'static RwLock<KaptState>, recording_index: usize) {
  let (is_chunk_active, audio_source) = {
    let state = state_lock
      .read()
      .expect("Failed to acquire state read lock");
    (
      state.active_recordings[recording_index].is_some(),
      state.audio_source,
    )
  };

  // If the current recording index is taken, stop it
  if is_chunk_active {
    stop_recording_chunk(state_lock, recording_index).await;
  }

  {
    // Check if the oldest recording chunk has an early end time that's less than the max minute history,
    // and remove it if so
    let is_oldest_chunk_expired = {
      let state = state_lock
        .read()
        .expect("Failed to acquire state read lock");

      let oldest_chunk = state.recordings.back();

      if let Some(oldest_chunk) = oldest_chunk {
        get_current_time() - oldest_chunk.early_end_time > state.max_seconds_cached as u128
      } else {
        false
      }
    };

    if is_oldest_chunk_expired {
      let mut state = state_lock
        .write()
        .expect("Failed to acquire state read lock");

      state.recordings.pop_front();
    }
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
    command = command.args(&["-i", &audio_source.to_string()]);
    command = command.args(&["-fflags", "+genpts"]);
    command = command.args(&["-async", "1"]);
    command = command.args(&["-vsync", "1"]);

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

pub async fn activate_kapt(state_lock: &'static RwLock<KaptState>) {
  {
    let state = state_lock.read().expect("Failed to acquire write lock");

    if state.is_active() {
      println!("Kapt has already been activated.");
      return;
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

  tauri::async_runtime::spawn(async move {
    loop {
      // Spawn a recording chunk for 15 seconds in the future
      sleep(Duration::from_secs(15)).await;
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

pub async fn deactivate_kapt(state_lock: &'static RwLock<KaptState>) {
  {
    let state = state_lock.read().expect("Failed to acquire write lock");

    if !state.is_active() {
      println!("Kapt isn't currently active.");
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

  let mut state = state_lock.write().expect("Failed to acquire write lock");
  state.recordings = VecDeque::new();
  state.recording_session_id = None;
  state.active_recordings = [None, None];
}