use std::collections::HashMap;

use tauri::api::process::CommandChild;

pub struct KaptState {
  pub active_recordings: [Option<FfmpegActiveRecording>; 2],

  // Used to prevent a thread from continuing a chunk recording when the
  // associated recording session has ended
  pub recording_session_id: Option<String>,

  // A map from a session ID to a Vec of FfmpegRecordings
  pub session_recordings: HashMap<String, Vec<FfmpegRecording>>,
}

impl KaptState {
  pub fn is_recording(&self) -> bool {
    self.active_recordings[0].is_some() || self.active_recordings[1].is_some()
  }

  pub fn add_recording(&mut self, session_id: String, recording: FfmpegRecording) {
    // Create a new HashMap entry if the session ID doesn't exist yet
    if !self.session_recordings.contains_key(&session_id) {
      self.session_recordings.insert(session_id.clone(), vec![]);
    }

    self
      .session_recordings
      .get_mut(&session_id)
      .expect("Session recording key not found")
      .push(recording);
  }

  pub fn new() -> Self {
    Self {
      active_recordings: [None, None],
      recording_session_id: None,
      session_recordings: HashMap::new(),
    }
  }
}

// A recording that's currently in process
pub struct FfmpegActiveRecording {
  pub path: String,
  pub start_time: Option<u128>,
  pub command_child: CommandChild,
}

// A recording that has already ended
pub struct FfmpegRecording {
  pub path: String,
  pub start_time: u128,
  pub end_time: u128,
}
