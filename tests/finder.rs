extern crate popi;
use popi::config::LocalStorage;
use popi::finder::ReposFinder;

#[tokio::test]
async fn listup_from_one_directory() {
  let config =
    LocalStorage::new_from_root_path("tests/fixtures/repo_search_1/config".into()).unwrap();
  let mut finder: ReposFinder = ReposFinder::new(config.repo_paths);
  let status = finder.init().await;
  let mut repos = finder.listup_repos();
  dbg!(&repos);
  dbg!(&status.paths_not_found);
  assert_eq!(status.paths_not_found.len(), 0);
  repos.sort_by(|a, b| a.name.cmp(&b.name));
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
  let mut repos = finder.listup_repos();
  dbg!(&repos);
  dbg!(&status.paths_not_found);
  assert_eq!(status.paths_not_found.len(), 0);
  repos.sort_by(|a, b| a.name.cmp(&b.name));
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
  let mut repos = finder.listup_repos();
  dbg!(&repos);
  dbg!(&status.paths_not_found);
  assert_eq!(status.paths_not_found.len(), 1);
  assert_eq!(
    status.paths_not_found[0].to_str().unwrap(),
    "tests/fixtures/repo_search_3/repos-B"
  );
  repos.sort_by(|a, b| a.name.cmp(&b.name));
  assert_eq!(repos.len(), 2);
  assert_eq!(repos[0].name, "a");
  assert_eq!(repos[1].name, "b");
}

#[tokio::test]
async fn search_by_1() {
  let config = LocalStorage::new_from_root_path("tests/fixtures/filter_1/config".into()).unwrap();
  let mut finder: ReposFinder = ReposFinder::new(config.repo_paths);
  let status = finder.init().await;

  let mut repos = finder.search_by("apple");
  dbg!(&repos);
  assert_eq!(status.paths_not_found.len(), 0);
  repos.sort_by(|a, b| a.repo.name.cmp(&b.repo.name));
  assert_eq!(repos.len(), 4);
  assert_eq!(repos[0].repo.name, "appde");
  assert_eq!(repos[1].repo.name, "apple");
  assert_eq!(repos[2].repo.name, "banana");
  assert_eq!(repos[3].repo.name, "sapporo");
}
