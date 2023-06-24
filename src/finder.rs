use std::path::PathBuf;

pub struct ReposFinder {
  pub repo_paths: Vec<PathBuf>,
}

pub struct Repo {
  pub path: PathBuf,
  pub name: String,
}

impl ReposFinder {
  pub fn new(repo_paths: Vec<PathBuf>) -> Self {
    ReposFinder { repo_paths }
  }

  pub fn search_by(&self, keyword: &str) -> Vec<Repo> {

    vec![]
  }

}
