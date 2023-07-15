use std::io::Stderr;

use crossterm::{cursor, queue};

use super::MainModeError;

pub fn safe_repeat(s: &str, n: isize) -> Result<String, MainModeError> {
  if n < 0 {
    return Err(MainModeError::NotEnoughtTerminalWidth);
  }
  if n == 0 {
    return Ok(String::new());
  }
  Ok(s.repeat(n as usize))
}

pub fn safe_move_to(
  stderr: &mut Stderr,
  x: i16,
  y: i16,
  width: i16,
  height: i16,
) -> Result<(), MainModeError> {
  if x >= width || x < 0 {
    return Err(MainModeError::NotEnoughtTerminalWidth);
  }
  if y >= height || y < 0 {
    return Err(MainModeError::NotEnoughtTerminalHeight);
  }
  queue!(stderr, cursor::MoveTo(x as u16, y as u16))
    .map_err(|_| MainModeError::StdoutWriteError)?;
  Ok(())
}
