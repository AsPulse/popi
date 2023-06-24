pub mod config;
use colored::Colorize;
use config::{LocalStorage, LoadConfigError};

pub fn run() {
  startup_message();

  let _config = LocalStorage::new().unwrap_or_else(|err| {
    match err {
      LoadConfigError::NoConfigFileFound { root_path } => {
        let mut config_yaml_path = root_path.clone();
        config_yaml_path.push("config.yml");
        eprintln!(
          " {} {}\n\n Run following commands to edit:\n {}\n {}",
          " ✖ERROR ".on_red().white().bold(),
          "config.yml not found in your config directory.".red(),
          format!("$ mkdir -p \"{}\"", root_path.to_str().unwrap()).bold(),
          format!("$ vim \"{}\"", config_yaml_path.to_str().unwrap()).bold(),
        );
      }
      LoadConfigError::ConfigInvalidYamlFormat { config_yml_path } => {
        eprintln!(
          " {} {}\n Please check the file has 'repos' properties with valid yaml format.",
          " ✖ERROR ".on_red().white().bold(),
          format!("your config file, {} is invalid.", config_yml_path.bold()).red(),
        );
      }
    }
    std::process::exit(1);
  });
}

fn startup_message() {
  println!("\n {}\n", "◇ popi v0.1.0".bold().cyan(),)
}
