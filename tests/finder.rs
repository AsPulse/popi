extern crate popi;
use popi::config::LocalStorage;
use popi::finder::ReposFinder;



#[tokio::test]
async fn full_listup() {
  let config = LocalStorage::new_from_root_path("tests/fixtures/repo_search_1/config".into()).unwrap();
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
