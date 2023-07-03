use crate::filter::MatchedString;

pub fn split_by_matched<'a>(s: &'a str, meta: &MatchedString) -> (&'a str, &'a str, &'a str) {
  (
    &s[..meta.matched_start],
    &s[meta.matched_start..meta.matched_start + meta.matched_length],
    &s[meta.matched_start + meta.matched_length..],
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_split_by_matched() {
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 0,
          matched_length: 1,
          distance: 0,
        }
      ),
      ("", "h", "ello")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 1,
          distance: 0,
        }
      ),
      ("h", "e", "llo")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 2,
          distance: 0,
        }
      ),
      ("h", "el", "lo")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 3,
          distance: 0,
        }
      ),
      ("h", "ell", "o")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 4,
          distance: 0,
        }
      ),
      ("h", "ello", "")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 0,
          matched_length: 5,
          distance: 0,
        }
      ),
      ("", "hello", "")
    );
  }
}
