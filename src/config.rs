use dirs::config_local_dir;
use std::path::{Path, PathBuf};
use thiserror::Error;
use yaml_rust::YamlLoader;

#[derive(Debug)]
pub struct LocalStorage {
  pub root_path: PathBuf,
  pub repo_paths: Vec<PathBuf>,
}

impl LocalStorage {
  pub fn new() -> Result<Self, LoadConfigError> {
    let mut dir = config_local_dir().unwrap();
    dir.push("popi");
    Self::new_from_root_path(dir)
  }
  pub fn new_from_root_path(root_path: PathBuf) -> Result<Self, LoadConfigError> {
    load_localstorage(root_path)
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
      }
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
  #[error("Could not find any paths config.yml")]
  NoConfigFileFound { root_path: PathBuf },
  #[error("config.yml has invalid yaml format")]
  ConfigInvalidYamlFormat { config_yml_path: String },
}

fn load_localstorage(root_path: PathBuf) -> Result<LocalStorage, LoadConfigError> {
  let (config_yml, config_yml_path) =
    read_file_with_priority(&root_path, vec!["config.yml", "config.yaml"]).map_err(|_| {
      LoadConfigError::NoConfigFileFound {
        root_path: root_path.to_path_buf(),
      }
    })?;

  let config_payload = YamlLoader::load_from_str(&config_yml).map_err(|_| {
    LoadConfigError::ConfigInvalidYamlFormat {
      config_yml_path: config_yml_path.to_string(),
    }
  })?;

  let repos = &config_payload
    .get(0)
    .ok_or(LoadConfigError::ConfigInvalidYamlFormat {
      config_yml_path: config_yml_path.to_string(),
    })?["repos"]
    .as_vec()
    .ok_or(LoadConfigError::ConfigInvalidYamlFormat {
      config_yml_path: config_yml_path.to_string(),
    })?;
  let repo_paths = &repos
    .iter()
    .map(|repo| {
      let repo_path = repo
        .as_str()
        .ok_or(LoadConfigError::ConfigInvalidYamlFormat {
          config_yml_path: config_yml_path.to_string(),
        })?;
      Ok(PathBuf::from(repo_path))
    })
    .collect::<Result<Vec<PathBuf>, LoadConfigError>>()?;

  Ok(LocalStorage {
    root_path,
    repo_paths: repo_paths.clone(),
  })
}
