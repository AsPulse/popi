extern crate popi;
use std::path::PathBuf;

use popi::config::{LoadConfigError, LocalStorage};

#[test]
fn loading_no_paths_config_file() {
  let err = LocalStorage::new_from_root_path("tests/fixtures/config_0".into()).unwrap_err();
  assert_eq!(
    err,
    LoadConfigError::NoConfigFileFound {
      root_path: PathBuf::from("tests/fixtures/config_0")
    }
  );
}

#[test]
fn loading_correct_paths() {
  let config = LocalStorage::new_from_root_path("tests/fixtures/config_1".into()).unwrap();
  assert_eq!(config.repo_paths.len(), 2);
  assert_eq!(
    config.repo_paths[0].to_str().unwrap(),
    "/Users/aspulse/repositories"
  );
  assert_eq!(
    config.repo_paths[1].to_str().unwrap(),
    "/Users/aspulse/github"
  );
}

#[test]
fn loading_broken_config() {
  let err = LocalStorage::new_from_root_path("tests/fixtures/config_2".into()).unwrap_err();
  assert_eq!(
    err,
    LoadConfigError::ConfigInvalidYamlFormat {
      config_yml_path: "tests/fixtures/config_2/config.yml".to_string()
    }
  );
}

#[test]
fn loading_empty_config() {
  let err = LocalStorage::new_from_root_path("tests/fixtures/config_3".into()).unwrap_err();
  assert_eq!(
    err,
    LoadConfigError::ConfigInvalidYamlFormat {
      config_yml_path: "tests/fixtures/config_3/config.yml".to_string()
    }
  );
}

#[test]
fn loading_config_with_no_paths() {
  let config = LocalStorage::new_from_root_path("tests/fixtures/config_4".into()).unwrap();
  assert_eq!(config.repo_paths.len(), 0);
}
