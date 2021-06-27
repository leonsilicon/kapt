use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::RwLock;

use crate::recording;
use crate::state::KaptState;
use crate::utils::create_temp_path;
use nanoid::nanoid;

pub fn time_to_string(time: u128) -> String {
  let milliseconds = time % 1000;
  // 1000 milliseconds in a second
  let seconds = time / 1000;

  // 60 seconds in a minute
  let minutes = time / (1000 * 60);

  // 60 minutes in an hour
  let hours = time / (1000 * 60 * 60);

  let time_string = format!(
    "{:0>2}:{:0>2}:{:0>2}.{:0>3}",
    hours, minutes, seconds, milliseconds
  );

  println!("time: {}, time string: {}", time, time_string);

  time_string
}

// R_i = Recording at position `i`; main recordings have an even index `i`, secondary
// recordings have an odd index `i`
// S_i = Start time of R_i recording
// E_i = End time of R_i
// Returns path of the final recording
pub async fn process_kapture(state_lock: &'static RwLock<KaptState>, timestamp: u128) -> String {
  // Stop the recording first
  recording::stop_recording(state_lock).await;

  {
    let mut state = state_lock
      .write()
      .expect("Failed to acquire state write lock");

    // Sort the recordings by audio_start_time
    state
      .recordings
      .sort_by(|r1, r2| r1.audio_start_time.cmp(&r2.audio_start_time));

    println!("Sorted Recordings: {:?}", state.recordings);

    if state.recordings.len() > 2 {
      // Change the early_end_time of recording `i` such that it's never larger than the audio start time of recording `i + 2`
      for i in 0..(state.recordings.len() - 2) {
        let next_recording_audio_start_time = state.recordings[i + 2].audio_start_time;
        let cur_recording = &mut state.recordings[i];

        if cur_recording.early_end_time >= next_recording_audio_start_time {
          cur_recording.early_end_time = next_recording_audio_start_time - 1;
        }
      }
    }
  }

  let mut state = state_lock
    .write()
    .expect("Failed to acquire state read lock");
  #[derive(Debug)]
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
    println!("End index: {}", end_index);

    let mut video_chunks: Vec<VideoChunk> = vec![];
    let mut total_time_ms: u128 = 0;

    for cur_index in (0..=end_index).rev() {
      let cur_recording = &state.recordings[cur_index];
      // If it's a main recording, time is E_i - S_i
      let is_audio_early = cur_recording.audio_start_time < cur_recording.video_start_time;
      let audio_video_discrepancy = if is_audio_early {
        cur_recording.video_start_time - cur_recording.audio_start_time
      } else {
        cur_recording.audio_start_time - cur_recording.video_start_time
      };

      if cur_index % 2 == 0 {
        if is_audio_early {
          let audio_clip_time =
            cur_recording.early_end_time - cur_recording.audio_start_time - audio_video_discrepancy;
          let video_clip_time = cur_recording.early_end_time - cur_recording.video_start_time;
          total_time_ms += audio_clip_time;
          video_chunks.push(VideoChunk {
            clip_index: cur_index,
            video_offset: 0,
            audio_offset: audio_video_discrepancy,
            video_time: video_clip_time,
            audio_time: audio_clip_time,
          })
        } else {
          let audio_clip_time = cur_recording.early_end_time - cur_recording.audio_start_time;
          let video_clip_time =
            cur_recording.early_end_time - cur_recording.video_start_time - audio_video_discrepancy;
          total_time_ms += audio_clip_time;
          video_chunks.push(VideoChunk {
            clip_index: cur_index,
            video_offset: audio_video_discrepancy,
            audio_offset: 0,
            video_time: video_clip_time,
            audio_time: audio_clip_time,
          })
        }
      }
      // If it's a secondary recording, time is S_(i+1) - E_(i-1)
      else {
        let prev_recording = &state.recordings[cur_index - 1];
        if cur_index == end_index {
          let audio_clip_time = timestamp - prev_recording.early_end_time;
          let video_clip_time = timestamp - prev_recording.early_end_time;

          total_time_ms += audio_clip_time;
          if is_audio_early {
            let video_offset = prev_recording.early_end_time - cur_recording.video_start_time;
            let audio_offset = prev_recording.early_end_time
              - cur_recording.video_start_time
              - audio_video_discrepancy;

            video_chunks.push(VideoChunk {
              clip_index: cur_index,
              video_offset,
              audio_offset,
              audio_time: audio_clip_time,
              video_time: video_clip_time,
            })
          } else {
            let video_offset = prev_recording.early_end_time
              - cur_recording.video_start_time
              - audio_video_discrepancy;
            let audio_offset = prev_recording.early_end_time - cur_recording.video_start_time;

            video_chunks.push(VideoChunk {
              clip_index: cur_index,
              video_offset,
              audio_offset,
              audio_time: audio_clip_time,
              video_time: video_clip_time,
            })
          }
        } else {
          let next_recording = &state.recordings[cur_index + 1];
          let is_next_recording_audio_early =
            next_recording.audio_start_time < next_recording.video_start_time;

          if is_next_recording_audio_early {
            let video_clip_time = next_recording.video_start_time - prev_recording.early_end_time;
            let audio_clip_time = next_recording.video_start_time - prev_recording.early_end_time;

            total_time_ms += audio_clip_time;
            if is_audio_early {
              let video_offset = prev_recording.early_end_time - cur_recording.video_start_time;
              let audio_offset = prev_recording.early_end_time
                - cur_recording.video_start_time
                - audio_video_discrepancy;

              video_chunks.push(VideoChunk {
                clip_index: cur_index,
                video_offset,
                audio_offset,
                audio_time: audio_clip_time,
                video_time: video_clip_time,
              })
            } else {
              let video_offset = prev_recording.early_end_time
                - cur_recording.video_start_time
                - audio_video_discrepancy;
              let audio_offset = prev_recording.early_end_time - cur_recording.video_start_time;

              video_chunks.push(VideoChunk {
                clip_index: cur_index,
                video_offset,
                audio_offset,
                audio_time: audio_clip_time,
                video_time: video_clip_time,
              })
            }
          } else {
            let video_clip_time = next_recording.audio_start_time - prev_recording.early_end_time;
            let audio_clip_time = next_recording.audio_start_time - prev_recording.early_end_time;

            total_time_ms += audio_clip_time;
            if is_audio_early {
              let video_offset = prev_recording.early_end_time - cur_recording.video_start_time;
              let audio_offset = prev_recording.early_end_time
                - cur_recording.video_start_time
                - audio_video_discrepancy;

              video_chunks.push(VideoChunk {
                clip_index: cur_index,
                video_offset,
                audio_offset,
                audio_time: audio_clip_time,
                video_time: video_clip_time,
              })
            } else {
              let video_offset = prev_recording.early_end_time
                - cur_recording.video_start_time
                - audio_video_discrepancy;
              let audio_offset = prev_recording.early_end_time - cur_recording.video_start_time;

              video_chunks.push(VideoChunk {
                clip_index: cur_index,
                video_offset,
                audio_offset,
                audio_time: audio_clip_time,
                video_time: video_clip_time,
              })
            }
          }
        }
      }

      let seconds_to_capture = state.seconds_to_capture as u128;

      // 15 seconds hardcoded for now
      if total_time_ms >= seconds_to_capture * 1000 {
        // Adjust the last chunk's offset
        let n = video_chunks.len();
        video_chunks[n - 1].audio_offset += total_time_ms - seconds_to_capture * 1000;
        video_chunks[n - 1].video_offset += total_time_ms - seconds_to_capture * 1000;

        video_chunks.reverse();
        return video_chunks;
      }
    }

    // The clips combined don't exceed 15 seconds
    video_chunks.reverse();
    video_chunks
  };

  let concat_recordings = |recording_index| {
    println!("Recording index: {:?}", recording_index);
    let video_chunks = get_video_chunks(recording_index);
    println!("Video chunks: {:?}", video_chunks);

    let mut temp_video_paths: Vec<String> = vec![];
    for video_chunk in video_chunks {
      let VideoChunk {
        audio_offset,
        audio_time,
        video_offset,
        video_time,
        clip_index,
      } = video_chunk;

      let temp_video_path = create_temp_path(&format!("{}.mp4", nanoid!()));

      let clip = &state.recordings[clip_index];
      // Combining the audio and video of the clip and making a temporary clip
      let mut command = Command::new("ffmpeg");

      println!("Video chunk: {:?}", video_chunk);
      println!("Video offset: {}", time_to_string(video_offset));
      println!("Video time: {}", time_to_string(video_time));
      println!("Audio offset: {}", time_to_string(audio_offset));
      println!("Audio time: {}", time_to_string(audio_time));

      command
        .args(&["-ss", &time_to_string(video_offset)])
        .args(&["-t", &time_to_string(video_time)])
        .args(&["-i", &clip.video_path])
        .args(&["-ss", &time_to_string(audio_offset)])
        .args(&["-t", &time_to_string(audio_time)])
        .args(&["-i", &clip.audio_path])
        .args(&["-map", "0:v:0", "-map", "1:a:0"])
        .args(&["-y"])
        .args(&[&temp_video_path]);

      command
        .spawn()
        .expect("Failed to run ffmpeg command")
        .wait()
        .expect("Failed to wait for ffmpeg");

      temp_video_paths.push(temp_video_path);
    }

    println!("Video paths: {:?}", temp_video_paths);

    let mut video_path_list = String::new();
    for temp_video_path in temp_video_paths {
      video_path_list.push_str(&format!("file '{}'\n", temp_video_path));
    }

    let temp_video_list_path = create_temp_path(&format!("{}.txt", nanoid!()));
    fs::write(&temp_video_list_path, video_path_list).expect("Failed to write video list file");

    let video_dir_path = state
      .video_folder
      .as_ref()
      .expect("Video folder not provided.");

    let final_video_path = Path::new(video_dir_path)
      .join(format!("{}.mp4", nanoid!()))
      .to_string_lossy()
      .to_string();

    let mut command = Command::new("ffmpeg");

    command
      .args(&["-f", "concat"])
      .args(&["-safe", "0"])
      .args(&["-i", &temp_video_list_path])
      .args(&["-c", "copy"])
      .args(&[&final_video_path]);

    command
      .spawn()
      .expect("Failed to spawn video concat command")
      .wait()
      .expect("failed to wait for concat");

    println!("Final video path: {:?}", final_video_path);

    final_video_path
  };

  println!("Recordings: {:#?}", state.recordings);

  // Get the most recent main recording in the array
  let mut i = state.recordings.len() - 1;

  while state.recordings[i].audio_start_time > timestamp {
    i -= 1;
  }

  println!("i: {:?}, timestamp: {}", i, timestamp);

  let video_path = if i % 2 == 0 {
    // First case
    println!("First case");
    concat_recordings(i)
  } else {
    let recent_even_recording = &state.recordings[i - 1];
    // If timestamp is after the end time of the main recording, need to use it
    if timestamp > recent_even_recording.early_end_time {
      println!("Second case");
      concat_recordings(i)
    } else {
      println!("Third case");
      concat_recordings(i - 1)
    }
  };

  // Clear recordings now that we've processed it
  state.recordings = vec![];

  // Reactivate the recording so that the user can make more kaptures
  tauri::async_runtime::spawn(async move {
    recording::start_recording(state_lock).await;
  });

  video_path
}

// timestamp - Unix timestamp of when the user pressed the Kapture button (in seconds)
pub async fn create_kapture(state_lock: &'static RwLock<KaptState>, timestamp: u128) -> String {
  let kapture_path = process_kapture(state_lock, timestamp).await;

  kapture_path
}
