pub mod config;
pub mod filter;
pub mod finder;
pub mod terminal;

use colored::Colorize;
use config::{LoadConfigError, LocalStorage};
use terminal::PopiTerminal;

use crate::{finder::ReposFinder, terminal::VERTICAL_LINE};

#[tokio::main]
pub async fn run() {
  startup_message();

  let config = LocalStorage::new().unwrap_or_else(|err| {
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
  let mut finder = ReposFinder::new(config.repo_paths);
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

    let warning_skip = !PopiTerminal::yes_or_no(format!(
      " {} {}",
      VERTICAL_LINE.yellow(),
      "Do you want to continue?".bold()
    ));

    if warning_skip {
      println!("");
    } else {
      std::process::exit(1);
    }
  }
}

fn startup_message() {
  println!("\n {}\n", "◇ popi v0.1.0".bold().cyan(),)
}
