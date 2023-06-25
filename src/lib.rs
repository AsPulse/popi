pub mod config;
pub mod filter;
pub mod finder;
pub mod main_mode;
pub mod strings;
pub mod terminal_util;

use colored::Colorize;

use crate::config::{LoadConfigError, LocalStorage};
use crate::finder::ReposFinder;
use crate::main_mode::call_main_mode;
use crate::strings::POPI_HEADER;
use crate::terminal_util::VERTICAL_LINE;

#[tokio::main]
pub async fn run() {
  startup_message();

  let storage = LocalStorage::new().unwrap_or_else(|err| {
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

  println!("{}", "Loading Repositories...".bright_black());
  let mut finder = ReposFinder::new(storage.repo_paths.to_vec());
  let repos_status = finder.init().await;
  println!("{}\n", "Finished!".bright_black());

  if !repos_status.paths_not_found.is_empty() {
    println!(
      " {} {}",
      "WARNING".yellow().bold(),
      "Following paths are not found:".red()
    );
    for path in repos_status.paths_not_found {
      println!(
        " {} - {}",
        VERTICAL_LINE.yellow(),
        path.to_str().unwrap_or("(Unknown Path)")
      );
    }
    println!(" {}", VERTICAL_LINE.yellow());

    let warning_skip = !terminal_util::yes_or_no(format!(
      " {} {}",
      VERTICAL_LINE.yellow(),
      "Do you want to continue?".bold()
    ));

    if warning_skip {
      println!();
    } else {
      std::process::exit(1);
    }
  }

  call_main_mode(storage, finder);
}

fn startup_message() {
  println!("\n {}\n", POPI_HEADER.bold().cyan());
}
