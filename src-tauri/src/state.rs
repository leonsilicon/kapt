use std::sync::RwLock;
use tokio::sync::mpsc::Receiver;

use tauri::api::process::{CommandChild, CommandEvent};

pub struct KaptState {
  pub active_recordings: [Option<FfmpegActiveRecording>; 2],

  // Used to prevent a thread from continuing a chunk recording when the
  // associated recording session has ended
  pub recording_session_id: Option<String>,

  // A map from a session ID to a Vec of FfmpegRecordings
  pub recordings: Vec<FfmpegRecording>,

  // The currently selected audio source for recording
  pub audio_source: usize,
}

impl KaptState {
  pub fn is_recording(&self) -> bool {
    self.active_recordings[0].is_some() || self.active_recordings[1].is_some()
  }

  pub fn new() -> Self {
    Self {
      active_recordings: [None, None],
      recording_session_id: None,
      recordings: vec![],
      audio_source: 0,
    }
  }
}

// A recording that's currently in process
pub struct FfmpegActiveRecording {
  pub video_command_child: CommandChild,
  pub video_path: String,
  pub video_rx: Receiver<CommandEvent>,
  pub video_start_time: Option<u128>,
  pub audio_command_child: CommandChild,
  pub audio_path: String,
  pub audio_rx: Receiver<CommandEvent>,
  pub audio_start_time: Option<u128>,
}

use lazy_static::lazy_static;

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

impl FfmpegActiveRecording {
  // Wait until the commands
  pub async fn stop(&mut self, state_lock: &'static RwLock<KaptState>) {
    if let Err(e) = self.video_command_child.write(&[b'q']) {
      log::error!("Failed to stop ffmpeg video process: {}", e);
    }

    if let Err(e) = self.audio_command_child.write(&[b'q']) {
      log::error!("Failed to stop ffmpeg audio process: {}", e)
    }

    let mut video_start_time: Option<u128> = None;
    let mut audio_start_time: Option<u128> = None;

    let parse_start_time = |line: String| {
      use regex::Regex;
      lazy_static! {
        static ref START_TIME_RE: Regex =
          Regex::new(r#"start: (\d+)\.(\d+)"#).expect("Failed to compile regex");
      };

      if let Some(cap) = START_TIME_RE.captures(&line) {
        if let Some(seconds) = cap.get(1) {
          if let Some(milliseconds) = cap.get(2) {
            let unix_timestamp_seconds = seconds
              .as_str()
              .to_string()
              .parse::<u128>()
              .expect("Failed to parse integer");

            let unix_timestamp_milliseconds = milliseconds
              .as_str()
              .to_string()
              .parse::<u128>()
              .expect("Failed to parse integer")
              / 1000;

            let unix_timestamp = unix_timestamp_seconds * 1000 + unix_timestamp_milliseconds;

            return Some(unix_timestamp);
          }
        }
      }

      None
    };

    while let Some(event) = self.video_rx.recv().await {
      // Ffmpeg logs to stderr
      if let CommandEvent::Stderr(line) = event {
        if let Some(unix_timestamp) = parse_start_time(line) {
          video_start_time = Some(unix_timestamp);
        }
      }
    }

    while let Some(event) = self.audio_rx.recv().await {
      // Ffmpeg logs to stderr
      println!("{:?}", event);
      if let CommandEvent::Stderr(line) = event {
        if let Some(unix_timestamp) = parse_start_time(line) {
          audio_start_time = Some(unix_timestamp);
        }
      }
    }

    // Ffmpeg process ended
    let early_end_time = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("Time went backwards")
      .as_millis();
    //

    let mut state = state_lock
      .write()
      .expect("Failed to acquire state read lock");

    let recording = FfmpegRecording {
      audio_path: self.audio_path.clone(),
      audio_start_time: audio_start_time.expect("Audio start time not found."),
      video_path: self.video_path.clone(),
      video_start_time: video_start_time.expect("Video start not not found."),
      early_end_time,
    };

    println!("Recording: {:?}", recording);

    state.recordings.push(recording);
  }
}

// A recording that has already ended
#[derive(Debug, Clone)]
pub struct FfmpegRecording {
  pub video_path: String,
  pub video_start_time: u128,
  pub audio_path: String,
  pub audio_start_time: u128,
  // The audio/video is guaranteed to have ended **after** this time
  pub early_end_time: u128,
}

impl Drop for FfmpegRecording {
  // Removing the temporary video and audio files when the recording is cleaned up
  fn drop(&mut self) {
    use std::fs;
    if let Err(e) = fs::remove_file(&self.audio_path) {
      log::error!("{}", e);
    }

    if let Err(e) = fs::remove_file(&self.video_path) {
      log::error!("{}", e);
    }
  }
}
