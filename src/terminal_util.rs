use std::io::{stdin, stdout, Write};

pub const VERTICAL_LINE: &str = "│";

pub fn yes_or_no(question: String) -> bool {
  let mut answer = String::new();
  loop {
    print!("{} [y/n]: ", question);
    stdout().flush().unwrap();
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
