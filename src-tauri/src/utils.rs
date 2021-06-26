use std::{env, path::Path};

pub fn create_temp_path(file_name: &str) -> String {
  Path::new(&env::temp_dir())
    .join(file_name)
    .to_string_lossy()
    .to_string()
}
