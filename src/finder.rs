use async_fs::read_dir;
use futures::TryStreamExt;
use std::path::PathBuf;

use crate::filter::{MatchedResult, MatchedString, PopiFilter};

pub struct ReposFinder {
  pub repo_paths: Vec<PathBuf>,
  pub repos: Option<Vec<Repo>>,
}

#[derive(Clone, Debug)]
pub struct Repo {
  pub path: PathBuf,
  pub name: String,
}

#[derive(Clone, Debug)]
pub struct FoundRepo {
  pub repo: Repo,
  pub matched_string: MatchedString,
}

pub struct ReposStatus {
  pub paths_not_found: Vec<PathBuf>,
}

impl ReposFinder {
  pub fn new(repo_paths: Vec<PathBuf>) -> Self {
    ReposFinder {
      repo_paths,
      repos: None,
    }
  }

  pub async fn init(&mut self) -> ReposStatus {
    let mut repos: Vec<Repo> = vec![];
    let mut paths_not_found: Vec<PathBuf> = vec![];

    let repos_grep = self.repo_paths.iter().map(|path| {
      let target_dir = path.clone();
      tokio::spawn(async move { listup_repos(target_dir).await })
    });

    for repo_status in repos_grep {
      match repo_status.await.unwrap() {
        RepoStatus::NotFound(path) => paths_not_found.push(path),
        RepoStatus::Found(repo_int_path) => {
          repos.extend(repo_int_path);
        }
      }
    }
    self.repos = Some(repos);
    ReposStatus { paths_not_found }
  }

  pub fn listup_repos(&self) -> Vec<Repo> {
    self.repos.clone().unwrap()
  }

  pub fn search_by(&self, keyword: &str) -> Vec<FoundRepo> {
    let converted_keyword = convert_to_lower(keyword.to_string());
    let mut entries = self
      .repos
      .clone()
      .unwrap()
      .into_iter()
      .map(|repo| {
        let name = convert_to_lower(repo.name.clone());
        (repo, PopiFilter::fuzzy_match(&converted_keyword, &name))
      })
      .filter_map(|(repo, match_result)| match match_result {
        MatchedResult::Matched(result) => Some((repo, result)),
        MatchedResult::NotMatched() => None,
      })
      .collect::<Vec<(Repo, MatchedString)>>();

    entries.sort_by(|(_, a), (_, b)| a.distance.partial_cmp(&b.distance).unwrap());
    entries
      .into_iter()
      .map(|(repo, matched_string)| FoundRepo {
        repo,
        matched_string,
      })
      .collect::<Vec<FoundRepo>>()
  }
}

enum RepoStatus {
  NotFound(PathBuf),
  Found(Vec<Repo>),
}

async fn listup_repos(repo_path: PathBuf) -> RepoStatus {
  match read_dir(&repo_path).await {
    Ok(mut repos) => {
      let mut result: Vec<Repo> = vec![];
      while let Some(repo) = {
        loop {
          match repos.try_next().await {
            Ok(repo) => break repo,
            Err(_) => continue,
          }
        }
      } {
        let path = repo.path();
        let name = repo.file_name().to_str().unwrap().to_string();
        result.push(Repo { path, name });
      }
      RepoStatus::Found(result)
    }
    Err(_) => RepoStatus::NotFound(repo_path.to_path_buf()),
  }
}

fn convert_to_lower(from: String) -> String {
  from
    .to_lowercase()
    .replace("_", "-")
    .replace("+", "=")
}
