pub mod config;
use config::{Config, LoadConfigError};
use colored::Colorize;

pub fn run() {

  startup_message();

  let _config = Config::new().unwrap_or_else(|err| {
    match err {
      LoadConfigError::NoPathsConfigFileFound { config_path } => {
        let mut config_file_path = config_path.clone();
        config_file_path.push("paths.yml");
        eprintln!(
          "{} {}\nRun this to edit: {}",
          " ✖ERROR ".on_red().white().bold(),
          "paths.yml not found in your config directory.",
          format!("$ vim \"{}\"", config_file_path.to_str().unwrap()).bold(),
        );
      },
      LoadConfigError::PathConfigInvalidYamlFormat { paths_yml_path } => {
        eprintln!(
          "{} {}\nPlease check the file has 'repos' properties with valid yaml format.",
          " ✖ERROR ".on_red().white().bold(),
          format!("your config file, {} is invalid.", paths_yml_path.bold()).red(),
        );
      },
    }
    std::process::exit(1);
  });
}

fn startup_message() {
  println!(
    "\n {}\n",
    "◇ popi v0.1.0".bold().cyan(),
  )
}
