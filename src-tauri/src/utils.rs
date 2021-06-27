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
