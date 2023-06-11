extern crate popi;
use popi::config::{Config, LoadConfigError};

#[test]
fn loading_no_paths_config_file() {
  let config = Config::new_from_config_path("tests/fixtures/config_0".into());
  assert_eq!(
    config.err().unwrap(),
    LoadConfigError::NoPathsConfigFileFound
  );
}

#[test]
fn loading_correct_paths() {
  let config = Config::new_from_config_path("tests/fixtures/config_1".into()).unwrap();
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
  let config = Config::new_from_config_path("tests/fixtures/config_2".into());
  assert_eq!(
    config.err().unwrap(),
    LoadConfigError::PathConfigInvalidYamlFormat
  );
}
