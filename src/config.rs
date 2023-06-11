use dirs::config_local_dir;
use std::path::{Path, PathBuf};
use thiserror::Error;
use yaml_rust::YamlLoader;

#[derive(Debug)]
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
  config_path: &Path,
  file_names: Vec<&str>,
) -> Result<(String, String), std::io::Error> {
  for file_name in file_names {
    let mut config_path_reading = config_path.to_path_buf();
    config_path_reading.push(file_name);
    match std::fs::read_to_string(&config_path_reading) {
      Ok(content) => {
        return Ok((content, config_path_reading.to_str().unwrap().to_string()));
      },
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
  NoPathsConfigFileFound {
    config_path: String,
  },
  #[error("Paths config has invalid yaml format")]
  PathConfigInvalidYamlFormat {
    paths_yml_path: String,
  },
}

fn load_config(config_path: PathBuf) -> Result<Config, LoadConfigError> {
  let (paths_yml, paths_yml_path) = read_file_with_priority(&config_path, vec!["paths.yml", "paths.yaml"])
    .map_err(|_| LoadConfigError::NoPathsConfigFileFound { config_path: config_path.to_str().unwrap().to_string() })?;

  let paths_payload = YamlLoader::load_from_str(&paths_yml)
    .map_err(|_| LoadConfigError::PathConfigInvalidYamlFormat { paths_yml_path: paths_yml_path.to_string() })?;

  let repos = &paths_payload[0]["repos"]
    .as_vec()
    .ok_or(LoadConfigError::PathConfigInvalidYamlFormat { paths_yml_path: paths_yml_path.to_string() })?;
  let repo_paths = &repos
    .iter()
    .map(|repo| {
      let repo_path = repo
        .as_str()
        .ok_or(LoadConfigError::PathConfigInvalidYamlFormat { paths_yml_path: paths_yml_path.to_string() })?;
      Ok(PathBuf::from(repo_path))
    })
    .collect::<Result<Vec<PathBuf>, LoadConfigError>>()?;

  Ok(Config {
    config_path,
    repo_paths: repo_paths.clone(),
  })
}
