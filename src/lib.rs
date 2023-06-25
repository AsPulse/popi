pub mod config;
pub mod filter;
pub mod finder;
pub mod terminal_util;

use std::{io::{stdout, Stdout}, error::Error};

use colored::Colorize;
use config::{LoadConfigError, LocalStorage};
use finder::ReposFinder;
use crossterm::{terminal::{enable_raw_mode, ClearType, disable_raw_mode}, execute};

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
      println!("");
    } else {
      std::process::exit(1);
    }
  }

  call_main_mode(storage, finder);
}

fn call_main_mode(storage: LocalStorage, finder: ReposFinder) {

  let mut stdout = stdout();
  execute!(
    stdout,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::cursor::Hide,
  ).unwrap();

  enable_raw_mode().unwrap();
  let main_mode_process = main_mode(storage, finder);
  disable_raw_mode().unwrap();

  execute!(
    stdout,
    crossterm::cursor::Show,
    crossterm::terminal::LeaveAlternateScreen,
  ).unwrap();

  main_mode_process.unwrap();
}

fn main_mode(storage: LocalStorage, finder: ReposFinder) -> Result<(), Box<dyn Error>> {
  let mut stdout = stdout();
  execute!(
    stdout,
    crossterm::terminal::Clear(ClearType::All),
    crossterm::cursor::MoveTo(0, 0),
    crossterm::style::Print("Hello World!"),
  )?;

  Ok(())
}

fn startup_message() {
  println!("\n {}\n", "◇ popi v0.1.0".bold().cyan(),)
}
