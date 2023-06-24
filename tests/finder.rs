extern crate popi;
use popi::config::LocalStorage;
use popi::finder::ReposFinder;



#[test]
fn full_listup() {
  let config = LocalStorage::new_from_root_path("tests/fixtures/repo_search_1/config".into()).unwrap();
  let finder = ReposFinder::new(config.repo_paths);
  let repos = finder.search_by("");
  assert_eq!(repos.len(), 3);
  assert_eq!(repos[0].name, "a");
  assert_eq!(repos[1].name, "b");
  assert_eq!(repos[2].name, "c");
}
