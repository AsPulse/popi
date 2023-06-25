pub struct PopiFilter {}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchedString {
  pub matched_start: usize,
  pub matched_length: usize,
  pub distance: usize,
}

#[derive(Debug, PartialEq)]
pub enum MatchedResult {
  Matched(MatchedString),
  NotMatched(),
}

impl PopiFilter {
  pub fn fuzzy_match(keyword: &str, target: &str) -> MatchedResult {
    let keyword_len = keyword.chars().count();

    let mut keyword_filter: (usize, usize) = (0, 0);
    loop {
      if keyword_filter.0 + keyword_filter.1 >= keyword_len {
        return MatchedResult::NotMatched();
      }
      let clipped_keyword = &keyword[keyword.char_indices().nth(keyword_filter.0).unwrap().0
        ..keyword
          .char_indices()
          .nth(keyword_len - keyword_filter.1)
          .map_or(keyword_len, |v| v.0)];

      let start = target
        .char_indices()
        .find(|c| c.1 == clipped_keyword.chars().next().unwrap())
        .map(|v| v.0);
      let end = target
        .char_indices()
        .rev()
        .find(|c| c.1 == clipped_keyword.chars().last().unwrap())
        .map(|v| v.0);
      if let (Some(start), Some(end)) = (start, end) {
        if start <= end {
          return MatchedResult::Matched(Self::get_matched_string(
            (start, end),
            target,
            clipped_keyword,
            keyword_filter.0 + keyword_filter.1,
          ));
        }
      }
      keyword_filter = Self::next_start_and_end(keyword_filter);
    }
  }

  pub(super) fn get_matched_string(
    (start, end): (usize, usize),
    target: &str,
    clipped_keyword: &str,
    keyword_filter_penalty: usize,
  ) -> MatchedString {
    let clipped_target = &target[start..end + 1];
    MatchedString {
      matched_start: start,
      matched_length: end - start + 1,
      distance: usize::try_from(stringmetrics::levenshtein(clipped_target, clipped_keyword))
        .unwrap()
        + keyword_filter_penalty,
    }
  }

  pub(super) fn next_start_and_end(current: (usize, usize)) -> (usize, usize) {
    let (start, end) = current;
    if start == 0 {
      (end + 1, 0)
    } else {
      (start - 1, end + 1)
    }
  }
}

#[cfg(test)]
mod test_next_start_and_end {
  use super::*;

  #[test]
  fn concurrent() {
    assert_eq!(PopiFilter::next_start_and_end((0, 0)), (1, 0));
    assert_eq!(PopiFilter::next_start_and_end((1, 0)), (0, 1));
    assert_eq!(PopiFilter::next_start_and_end((0, 1)), (2, 0));
    assert_eq!(PopiFilter::next_start_and_end((2, 0)), (1, 1));
    assert_eq!(PopiFilter::next_start_and_end((1, 1)), (0, 2));
    assert_eq!(PopiFilter::next_start_and_end((0, 2)), (3, 0));
    assert_eq!(PopiFilter::next_start_and_end((3, 0)), (2, 1));
    assert_eq!(PopiFilter::next_start_and_end((2, 1)), (1, 2));
    assert_eq!(PopiFilter::next_start_and_end((1, 2)), (0, 3));
    assert_eq!(PopiFilter::next_start_and_end((0, 3)), (4, 0));
  }
}

#[cfg(test)]
mod test_fuzzy_match {
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
    assert_eq!(
      PopiFilter::fuzzy_match("bdf", "abcdefg"),
      MatchedResult::Matched(MatchedString {
        matched_start: 1,
        matched_length: 5,
        distance: 2,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("abcd", "abc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 3,
        distance: 1,
      })
    );
  }

  #[test]
  fn matched_with_distance_and_duplicates() {
    assert_eq!(
      PopiFilter::fuzzy_match("atisiie", "satisfied"),
      MatchedResult::Matched(MatchedString {
        matched_start: 1,
        matched_length: 7,
        distance: 1,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("aspulse", "aspulse-k8s-manifests"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 7,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("zen", "zenn"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 3,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("sugar", "sugarform"),
      MatchedResult::Matched(MatchedString {
        matched_start: 0,
        matched_length: 5,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("deno", "discordeno-"),
      MatchedResult::Matched(MatchedString {
        matched_start: 6,
        matched_length: 4,
        distance: 0,
      })
    );
    assert_eq!(
      PopiFilter::fuzzy_match("o", "abcoxxxoabc"),
      MatchedResult::Matched(MatchedString {
        matched_start: 4,
        matched_length: 1,
        distance: 0,
      })
    );
  }

  #[test]
  fn no_matched_with_empty_keyword() {
    assert_eq!(
      PopiFilter::fuzzy_match("", "abc"),
      MatchedResult::NotMatched()
    );
  }
}
