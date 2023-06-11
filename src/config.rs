use dirs::config_local_dir;
use std::path::PathBuf;
use thiserror::Error;
use yaml_rust::YamlLoader;

pub struct Config {
  pub config_path: PathBuf,
  pub repo_paths: Vec<PathBuf>,
}

impl Config {
  pub fn new() -> Result<Self, LoadConfigError> {
    Self::new_from_config_path(config_local_dir().unwrap())
  }
  pub fn new_from_config_path(config_path: PathBuf) -> Result<Self, LoadConfigError> {
    load_config(config_path)
  }
}

// Try to load files with given order, if all files are not found, return Err
fn read_file_with_priority(
  config_path: &PathBuf,
  file_names: Vec<&str>,
) -> Result<String, std::io::Error> {
  for file_name in file_names {
    let mut path = config_path.clone();
    path.push(file_name);
    match std::fs::read_to_string(path) {
      Ok(content) => return Ok(content),
      Err(_) => continue,
    }
  }
  Err(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "Could not find any files from given paths",
  ))
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LoadConfigError {
  #[error("Could not find any paths config file")]
  NoPathsConfigFileFound,
  #[error("Paths config has invalid yaml format")]
  PathConfigInvalidYamlFormat,
}

fn load_config(config_path: PathBuf) -> Result<Config, LoadConfigError> {
  let paths_yml: String = read_file_with_priority(&config_path, vec!["paths.yml", "paths.yaml"])
    .map_err(|_| LoadConfigError::NoPathsConfigFileFound)?;

  let paths_payload = YamlLoader::load_from_str(&paths_yml)
    .map_err(|_| LoadConfigError::PathConfigInvalidYamlFormat)?;

  let repos = &paths_payload[0]["repos"].as_vec().ok_or(LoadConfigError::PathConfigInvalidYamlFormat)?;
  let repo_paths = &repos.iter().map(|repo| {
    let repo_path = repo.as_str().ok_or(LoadConfigError::PathConfigInvalidYamlFormat)?;
    Ok(
      PathBuf::from(repo_path)
    )
  }).collect::<Result<Vec<PathBuf>, LoadConfigError>>()?;


  Ok(Config {
    config_path,
    repo_paths: repo_paths.clone(),
  })
}

