use chrono::{Datelike, Timelike};
use std::{env, path::Path};

pub fn create_temp_path(file_name: &str) -> String {
  Path::new(&env::temp_dir())
    .join(file_name)
    .to_string_lossy()
    .to_string()
}

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub fn get_current_time() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards")
    .as_millis()
}

pub fn get_file_date_string() -> String {
  let cur_date_time = chrono::offset::Local::now();
  let year = cur_date_time.year();
  let month = cur_date_time.month();
  let day = cur_date_time.day();
  let hour = cur_date_time.hour();
  let minute = cur_date_time.minute();
  let second = cur_date_time.second();

  format!(
    "{}-{:0>2}-{:0>2}--{:0>2}-{:0>2}-{:0>2}",
    year, month, day, hour, minute, second
  )
}
