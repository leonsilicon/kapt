use tauri::api::process::CommandChild;

use std::collections::HashMap;

pub struct KaptState {
  pub active_recordings: [Option<FfmpegActiveRecording>; 2],

  // Used to prevent a thread from continuing a chunk recording when the
  // associated recording session has ended
  pub recording_session_id: Option<String>,

  // A map from a session ID to a Vec of FfmpegRecordings
  pub recordings: Vec<FfmpegRecording>,
}

impl KaptState {
  pub fn is_recording(&self) -> bool {
    self.active_recordings[0].is_some() || self.active_recordings[1].is_some()
  }
}

// A recording that's currently in process
pub struct FfmpegActiveRecording {
  pub video_command_child: CommandChild,
  pub video_path: String,
  pub video_start_time: Option<u128>,
  pub audio_command_child: CommandChild,
  pub audio_path: String,
  pub audio_start_time: Option<u128>,
}

// A recording that has already ended
pub struct FfmpegRecording {
  pub video_path: String,
  pub video_start_time: u128,
  pub audio_path: String,
  pub audio_start_time: u128,
  // The audio/video is guaranteed to have ended **after** this time
  pub early_end_time: u128,
}
