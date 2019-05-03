extern crate ffmpeg_sys;

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> std::io::Result<()> {
  let dir = std::env::temp_dir().join("__ffmpeg_bin_tmp__eg_1");
  fs::create_dir_all(&dir)?;
  ffmpeg_sys::install_ffmpeg(&dir)?;
  let status = Command::new("ffmpeg")
    .current_dir(&dir)
    .arg("-version")
    .status();
  fs::remove_dir_all(&dir);
  Ok(())
}
