use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

const FFMPEG_BIN: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ffmpeg"));
const FFPROBE_BIN: &'static [u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ffprobe"));

pub fn install_ffmpeg<T>(dir_path: T) -> std::io::Result<()>
  where T: AsRef<Path>
{
  let mut file = File::create(dir_path.as_ref().join("ffmpeg"))?;
  file.write_all(FFMPEG_BIN)?;
  Ok(())
}

pub fn install_ffprobe<T>(dir_path: T) -> std::io::Result<()>
  where T: AsRef<Path>
{
  let mut file = File::create(dir_path.as_ref().join("ffprobe"))?;
  file.write_all(FFPROBE_BIN)?;
  Ok(())
}
