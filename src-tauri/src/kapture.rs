use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::RwLock;

use crate::recording;
use crate::state::KaptState;
use crate::utils::create_temp_path;
use nanoid::nanoid;

pub fn offset_to_string(offset: u128) -> String {
  let milliseconds = offset % 1000;
  // 1000 milliseconds in a second
  let seconds = offset / 1000;

  // 60 seconds in a minute
  let minutes = offset / (1000 * 60);

  // 60 minutes in an hour
  let hours = offset / (1000 * 60 * 60);

  return format!("{}:{}:{}.{}", hours, minutes, seconds, milliseconds);
}

// Converts a time in milliseconds to a string with time in seconds (and decimals)
pub fn time_to_string(time: u128) -> String {
  let seconds = (time as f64) / 1000.0;

  return seconds.to_string();
}

// R_i = Recording at position `i`; main recordings have an even index `i`, secondary
// recordings have an odd index `i`
// S_i = Start time of R_i recording
// E_i = End time of R_i
// Returns path of the final recording
pub fn process_kapture(state_lock: &'static RwLock<KaptState>, timestamp: u128) -> String {
  // Stop the recording first
  recording::stop_recording(state_lock);

  let state = state_lock
    .read()
    .expect("Failed to acquire state read lock");

  // Get the most recent main recording in the array
  let mut i = state.recordings.len() - 1;

  while state.recordings[i].audio_start_time > timestamp {
    i -= 1;
  }

  struct VideoChunk {
    pub clip_index: usize,
    pub video_offset: u128,
    pub audio_offset: u128,
    pub video_time: u128,
    pub audio_time: u128,
  }

  // Returns the start clip for a 15-second video and the offset of the clip to make
  // the video 15 seconds
  let get_video_chunks = |end_index| {
    let video_chunks: Vec<VideoChunk> = vec![];
    let total_time_ms: u128 = 0;

    for cur_index in (0..end_index - 1).rev() {
      let cur_recording = state.recordings[cur_index];
      // If it's a main recording, time is E_i - S_i
      if cur_index % 2 == 0 {
        let audio_clip_time = cur_recording.early_end_time - cur_recording.audio_start_time;
        let video_clip_time = cur_recording.early_end_time - cur_recording.video_start_time;
        total_time_ms += audio_clip_time;
        video_chunks.push(VideoChunk {
          clip_index: cur_index,
          video_offset: 0,
          audio_offset: 0,
          video_time: video_clip_time,
          audio_time: audio_clip_time,
        })
      }
      // If it's a secondary recording, time is S_(i+1) - E_(i-1)
      else {
        let prev_recording = state.recordings[cur_index - 1];
        let (audio_clip_time, video_clip_time) = if cur_index == end_index {
          (
            timestamp - prev_recording.early_end_time,
            timestamp - prev_recording.early_end_time,
          )
        } else {
          let next_recording = state.recordings[cur_index + 1];
          (
            next_recording.audio_start_time - prev_recording.early_end_time,
            next_recording.video_start_time - prev_recording.early_end_time,
          )
        };

        total_time_ms += audio_clip_time;
        video_chunks.push(VideoChunk {
          clip_index: cur_index,
          video_offset: prev_recording.early_end_time,
          audio_offset: prev_recording.early_end_time,
          audio_time: audio_clip_time,
          video_time: video_clip_time,
        })
      }

      // 15 seconds hardcoded for now
      if total_time_ms >= 15 * 1000 {
        // Adjust the last chunk's offset
        let n = video_chunks.len();
        video_chunks[n - 1].audio_offset += total_time_ms - 15 * 1000;
        video_chunks[n - 1].video_offset += total_time_ms - 15 * 1000;

        return video_chunks;
      }
    }

    // If the clips combined don't exceed 15 seconds
    return video_chunks;
  };

  let concat_recordings = |recording_index| {
    let video_chunks = get_video_chunks(recording_index);

    let temp_video_paths: Vec<String> = vec![];
    for video_chunk in video_chunks {
      let VideoChunk {
        audio_offset,
        audio_time,
        video_offset,
        video_time,
        clip_index,
      } = video_chunk;

      let temp_video_path = Path::new(&env::temp_dir())
        .join(format!("{}.mp4", nanoid!()))
        .to_string_lossy()
        .to_string();

      let clip = state.recordings[clip_index];
      // Combining the audio and video of the clip and making a temporary clip
      let command = Command::new("ffmpeg")
        .args(&["-ss", &offset_to_string(video_offset)])
        .args(&["-t", &time_to_string(video_time)])
        .args(&["-i", &clip.video_path])
        .args(&["-ss", &offset_to_string(audio_offset)])
        .args(&["-t", &time_to_string(audio_time)])
        .args(&["-i", &clip.audio_path])
        .args(&[clip.video_path])
        .args(&["-map", "0:v:0", "-map", "1:a:0"])
        .args(&["-y"])
        .args(&[&temp_video_path]);

      command
        .spawn()
        .expect("Failed to run ffmpeg command")
        .wait();

      temp_video_paths.push(temp_video_path);
    }

    let video_path_list = String::new();
    for temp_video_path in temp_video_paths {
      video_path_list.push_str(&format!("file '{}'\n", temp_video_path));
    }

    let temp_video_list_path = create_temp_path(&format!("{}.txt", nanoid!()));
    let final_video_path = create_temp_path(&format!("{}.mp4", nanoid!()));

    fs::write(temp_video_list_path, video_path_list).expect("Failed to write video list file");

    let command = Command::new("ffmpeg")
      .args(&["-f", "concat"])
      .args(&["-safe", "0"])
      .args(&["-i", &temp_video_list_path])
      .args(&["-c", "copy"])
      .args(&[final_video_path]);

    command
      .spawn()
      .expect("Failed to spawn video concat command")
      .wait();

    final_video_path
  };

  if i % 2 == 0 {
    // First case
    concat_recordings(i)
  } else {
    let recent_even_recording = state.recordings[i - 1];
    // If timestamp is after the end time of the main recording, need to use it
    if timestamp > recent_even_recording.early_end_time {
      concat_recordings(i)
    } else {
      concat_recordings(i - 1)
    }
  }
}

#[tauri::command]
// timestamp - Unix timestamp of when the user pressed the Kapture button (in seconds)
pub fn create_kapture(
  window: tauri::Window,
  state_lock: tauri::State<&'static RwLock<KaptState>>,
  timestamp: u128,
) -> String {
  let kapture_path = process_kapture(*state_lock, timestamp);

  kapture_path
}