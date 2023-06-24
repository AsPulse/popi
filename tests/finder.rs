extern crate popi;
use popi::config::LocalStorage;
use popi::finder::ReposFinder;

#[tokio::test]
async fn listup_from_one_directory() {
  let config =
    LocalStorage::new_from_root_path("tests/fixtures/repo_search_1/config".into()).unwrap();
  let mut finder: ReposFinder = ReposFinder::new(config.repo_paths);
  let status = finder.init().await;
  let repos = finder.search_by("");
  dbg!(&repos);
  dbg!(&status.paths_not_found);
  assert_eq!(status.paths_not_found.len(), 0);
  assert_eq!(repos.len(), 2);
  assert_eq!(repos[0].name, "a");
  assert_eq!(repos[1].name, "b");
}

#[tokio::test]
async fn listup_from_multi_directories() {
  let config =
    LocalStorage::new_from_root_path("tests/fixtures/repo_search_2/config".into()).unwrap();
  let mut finder: ReposFinder = ReposFinder::new(config.repo_paths);
  let status = finder.init().await;
  let repos = finder.search_by("");
  dbg!(&repos);
  dbg!(&status.paths_not_found);
  assert_eq!(status.paths_not_found.len(), 0);
  assert_eq!(repos.len(), 4);
  assert_eq!(repos[0].name, "a");
  assert_eq!(repos[1].name, "b");
  assert_eq!(repos[2].name, "c");
  assert_eq!(repos[3].name, "d");
}

#[tokio::test]
async fn listup_with_unexisting_directory() {
  let config =
    LocalStorage::new_from_root_path("tests/fixtures/repo_search_3/config".into()).unwrap();
  let mut finder: ReposFinder = ReposFinder::new(config.repo_paths);
  let status = finder.init().await;
  let repos = finder.search_by("");
  dbg!(&repos);
  dbg!(&status.paths_not_found);
  assert_eq!(status.paths_not_found.len(), 1);
  assert_eq!(
    status.paths_not_found[0].to_str().unwrap(),
    "tests/fixtures/repo_search_3/repos-B"
  );
  assert_eq!(repos.len(), 2);
  assert_eq!(repos[0].name, "a");
  assert_eq!(repos[1].name, "b");
}
