use std::io::{stderr, stdin, Write};

pub const VERTICAL_LINE: &str = "│";
pub const HORIZONTAL_LINE: &str = "─";
pub const TOP_LEFT_CORNER: &str = "╭";
pub const TOP_RIGHT_CORNER: &str = "╮";
pub const BOTTOM_LEFT_CORNER: &str = "╰";
pub const BOTTOM_RIGHT_CORNER: &str = "╯";

pub fn yes_or_no(question: String) -> bool {
  let mut answer = String::new();
  loop {
    print!("{} [y/n]: ", question);
    stderr().flush().unwrap();
    stdin().read_line(&mut answer).unwrap();
    match answer.trim().to_lowercase().as_str() {
      "y" | "yes" => return true,
      "n" | "no" => return false,
      _ => {
        answer.clear();
      }
    }
  }
}
