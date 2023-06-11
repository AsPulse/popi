mod config;

pub fn run() {
  // print config path
  println!("Config Path: {}", config::get_config_path());
}
