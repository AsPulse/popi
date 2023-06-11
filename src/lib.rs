mod config;
use crate::config::Config;

pub fn run() {
  let config = Config::new();
  println!("Config Path: {}", config.config_path);
}
