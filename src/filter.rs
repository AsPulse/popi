pub struct PopiFilter {}

#[derive(Debug, PartialEq)]
pub struct MatchedString {
  pub matched_start: u32,
  pub matched_length: u32,
  pub distance: u32,
}

#[derive(Debug, PartialEq)]
pub enum MatchedResult {
  Matched(MatchedString),
  NotMatched(),
}

impl PopiFilter {
  pub fn fuzzy_match(keyword: &str, target: &str) -> MatchedResult {
    MatchedResult::NotMatched()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn complete_matches() {
    assert_eq!(
      PopiFilter::fuzzy_match("abc", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 3,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("ab", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 2,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("bc", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 1,
        matched_length: 2,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("a", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 1,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("c", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 2,
        matched_length: 1,
        distance: 0,
      })
    );
  }

  #[test]
  fn not_matched() {
    assert_eq!(
      PopiFilter::fuzzy_match("de", "abc"),
      MatchedResult::NotMatched()
    );
  }

  #[test]
  fn matched_with_distance() {
    assert_eq!(
      PopiFilter::fuzzy_match("ac", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 3,
        distance: 1,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("abcse", "abcdef"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 5,
        distance: 1,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("abcdde", "abcdef"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 5,
        distance: 1,
      })
    );
  }
}
