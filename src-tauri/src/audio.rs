use serde::Serialize;

#[derive(Serialize)]
pub struct AudioSource {
  description: String,
  id: usize,
}

use std::process::{Command, Stdio};
pub fn get_audio_sources() -> Vec<AudioSource> {
  let sources_descriptions: Vec<String> = {
    let list_sources_child = Command::new("pactl")
      .args(&["list", "sources"])
      .stdout(Stdio::piped())
      .spawn();
    let sources_descriptions = String::from_utf8(if list_sources_child.is_ok() {
      Command::new("grep")
        .args(&["-e", "device.description"])
        .stdin(list_sources_child.unwrap().stdout.take().unwrap())
        .output()
        .unwrap()
        .stdout
    } else {
      Vec::new()
    })
    .unwrap();
    sources_descriptions
      .split("\n")
      .map(|s| {
        s.trim()
          .replace("device.description = ", "")
          .replace("\"", "")
      })
      .filter(|s| s != "")
      .collect()
  };

  let sources_ids: Vec<usize> = {
    let list_sources_child = Command::new("pactl")
      .args(&["list", "sources"])
      .stdout(Stdio::piped())
      .spawn();
    let sources_ids = String::from_utf8(if list_sources_child.is_ok() {
      Command::new("grep")
        .args(&["-e", "Source #"])
        .stdin(list_sources_child.unwrap().stdout.take().unwrap())
        .output()
        .unwrap()
        .stdout
    } else {
      Vec::new()
    })
    .unwrap();
    sources_ids
      .split("\n")
      .map(|s| s.trim().replace("Source #", ""))
      .filter(|s| s != "")
      .map(|s| s.parse::<usize>().expect("Failed to parse source id"))
      .collect()
  };

  let mut audio_devices: Vec<AudioSource> = vec![];
  for (pos, description) in sources_descriptions.into_iter().enumerate() {
    audio_devices.push(AudioSource {
      description,
      id: sources_ids[pos],
    })
  }

  audio_devices
}
