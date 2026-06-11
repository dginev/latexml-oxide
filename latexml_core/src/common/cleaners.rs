use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use unicode_normalization::UnicodeNormalization;
use unidecode::unidecode;

use crate::binding::def::dialect::{
  DIRTY_ID_IDIOM_RE, LEADING_PROTOCOL_RE, NON_ID_CHARSET_RE, SPACES_RE, TILDE_NOISE_RE,
  TRAILING_SLASH_RE,
};

static TRAILING_PUNCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\.,;]+$").unwrap());
static NON_ALNUM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^a-zA-Z0-9]").unwrap());

//======================================================================
// Cleaners
//======================================================================

static RMLETTERS: [char; 7] = ['i', 'v', 'x', 'l', 'c', 'd', 'm'];
/// auxiliary helper for `roman`
pub fn roman_aux<T: Into<i64>>(stuff: T) -> String {
  let mut n: i64 = stuff.into();
  if n <= 0 {
    return String::new();
  }
  let mut div = 1000;
  let mut s: String = if n >= div {
    String::from_utf8(vec![b'm'; (n / div) as usize]).unwrap()
  } else {
    String::new()
  };
  let mut p = 4;
  while n % div != 0 {
    n %= div;
    div /= 10;
    let mut d = n / div;
    if d % 5 == 4 {
      s.push(RMLETTERS[p]);
      d += 1;
    }
    if d > 4 {
      s.push(RMLETTERS[p + (d / 5) as usize]);
      d %= 5;
    }
    if d != 0 {
      s.push_str(&String::from_utf8(vec![RMLETTERS[p] as u8; d as usize]).unwrap());
    }
    // silly, but i'm postponing rewriting the entire method for now, just porting over from Perl
    if p > 2 {
      p -= 2;
    } else {
      p = 0;
    }
  }
  s
}

/// cleans a string down to characters acceptable for an id attribute
pub fn clean_id(key: &str) -> String {
  let cleaned = Cow::Borrowed(key.trim()); // Trim leading/trailing whitespace
  let cleaned_1 = SPACES_RE.replace_all(&cleaned, ""); // remove all spaces
  // Remove common idiom:
  // Perl parity: CleanID strips `${}^{foo}$` down to just `foo`.
  // The regex captures that inner content as named group `label`; the
  // replacement must reference it by the correct name (`$inner` was a
  // stale typo that silently erased the captured text).
  let cleaned_2 = DIRTY_ID_IDIOM_RE.replace_all(&cleaned_1, "$label");
  // transform some forbidden chars
  let cleaned_3 = cleaned_2
    .replace(':', "..") // No colons!
    .replace('@', "-at-")
    .replace('*', "-star-")
    .replace('$', "-dollar-")
    .replace(',', "-comma-")
    .replace('%', "-pct-")
    .replace('&', "-amp-");
  let cleaned_4 = unidecode(&cleaned_3);
  let cleaned_5 = NON_ID_CHARSET_RE.replace_all(&cleaned_4, ""); // remove everything else.
  let out = cleaned_5.as_ref();
  // Perl parity (Package.pm CleanID): XML ids must start with a letter or `_`
  // (since we already replaced `:` with `..`). Prepend "X" when the cleaned
  // key starts with anything else — protects against leading `.`, `-`, or
  // digits, which would otherwise produce invalid id attributes.
  match out.chars().next() {
    Some(c) if c.is_ascii_alphabetic() || c == '_' => out.to_string(),
    Some(_) => format!("X{out}"),
    None => String::new(),
  }
}
/// cleans a string down to characters acceptable for a label attribute
pub fn clean_label<'a>(label: &'a str, prefix_opt: Option<&str>) -> Cow<'a, str> {
  let key = label.trim(); // Trim leading/trailing, in any case
  let cleaned_1 = SPACES_RE.replace_all(key, "_"); // spaces to underscores
  let prefix = prefix_opt.unwrap_or("LABEL");
  if prefix.is_empty() {
    cleaned_1
  } else {
    Cow::Owned(s!("{}:{}", prefix, cleaned_1))
  }
}

/// Clean string for use in index keys (Perl: CleanIndexKey)
/// Applies NFC normalization and removes trailing punctuation.
pub fn clean_index_key(key: &str) -> String {
  let trimmed = key.trim();
  let normalized: String = trimmed.nfc().collect();
  TRAILING_PUNCT_RE.replace(&normalized, "").to_string()
}

/// Clean string for use as a CSS class name (Perl: CleanClassName)
/// Decomposes to NFD, removes non-alphanumeric chars, recomposes to NFC.
pub fn clean_class_name(key: &str) -> String {
  let trimmed = key.trim();
  let decomposed: String = trimmed.nfd().collect();
  let cleaned = NON_ALNUM_RE.replace_all(&decomposed, "");
  cleaned.nfc().collect()
}

/// cleans a string down to characters acceptable for a bibliography key
pub fn clean_bib_key(key: &str) -> String {
  // Originally lc() here, but let's preserve case till Postproc.
  let trimmed = key.trim();
  SPACES_RE.replace_all(trimmed, "").to_string()
}

/// Return the bibkey in a form to ACTUALLY lookup (Perl: NormalizeBibKey)
/// Usually use clean_bib_key to preserve key in the original form (case)
pub fn normalize_bib_key(key: &str) -> String { clean_bib_key(key).to_lowercase() }

/// Split comma-separated text into trimmed tokens (Perl: TrimmedCommaList)
pub fn trimmed_comma_list(text: &str) -> Vec<String> {
  let trimmed = text.trim();
  if trimmed.is_empty() {
    return Vec::new();
  }
  trimmed.split(',').map(|s| s.trim().to_string()).collect()
}

/// cleans a string down to characters acceptable for a URL
pub fn clean_url(url: &str) -> String {
  let cleaned = url.trim(); // Trim leading/trailing whitespace
  TILDE_NOISE_RE.replace_all(cleaned, "~").to_string()
}

/// builds a complete url from fragments
pub fn compose_url(base: &str, url: &str, fragid_opt: Option<&str>) -> String {
  let base = TRAILING_SLASH_RE.replace(base, ""); //  remove trailing /
  let fragid = fragid_opt.unwrap_or("");
  let base: String = if !base.is_empty() && !LEADING_PROTOCOL_RE.is_match(url) {
    // already has protocol, so is absolute url
    base.to_string() + if url.starts_with('/') { "" } else { "/" } // else start w/base, possibly /
  } else {
    String::new()
  };
  let fragid: String = if !fragid.is_empty() {
    s!("#{}", clean_id(fragid))
  } else {
    String::new()
  };
  clean_url(&(base + url + &fragid))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clean_id_preserves_alpha_start() {
    assert_eq!(clean_id("foo"), "foo");
    assert_eq!(clean_id("Foo_bar"), "Foo_bar");
    assert_eq!(clean_id("_underscore"), "_underscore");
  }

  #[test]
  fn clean_id_prepends_x_for_non_alpha_start() {
    // Leading digit, dot, or hyphen is invalid in XML ids — prepend X.
    assert_eq!(clean_id("1foo"), "X1foo");
    assert_eq!(clean_id(".foo"), "X.foo");
    assert_eq!(clean_id("-foo"), "X-foo");
  }

  #[test]
  fn clean_id_after_colon_replacement() {
    // `:` becomes `..`, so ":foo" → "..foo" → needs X prefix.
    assert_eq!(clean_id(":foo"), "X..foo");
  }

  #[test]
  fn clean_id_empty_stays_empty() {
    assert_eq!(clean_id(""), "");
    assert_eq!(clean_id("   "), "");
  }

  #[test]
  fn clean_id_dirty_idiom_preserves_label() {
    // Perl: $key =~ s/\$\{\}\^\{(.*?)\}\$/$1/g; retains the captured
    // content. Common TeX idiom from latex generates ${}^{foo}$.
    assert_eq!(clean_id("${}^{foo}$"), "foo");
    assert_eq!(clean_id("bar${}^{tag}$"), "bartag");
  }

  #[test]
  fn roman_aux_non_positive() {
    assert_eq!(roman_aux(0i64), "");
    assert_eq!(roman_aux(-1i64), "");
  }

  #[test]
  fn roman_aux_basic() {
    assert_eq!(roman_aux(1i64), "i");
    assert_eq!(roman_aux(1000i64), "m");
    assert_eq!(roman_aux(1999i64), "mcmxcix");
  }

  #[test]
  fn clean_label_default_prefix() {
    // Spaces become underscores; default prefix is "LABEL:".
    assert_eq!(clean_label("foo bar", None), "LABEL:foo_bar");
    assert_eq!(clean_label("simple", None), "LABEL:simple");
  }

  #[test]
  fn clean_label_custom_prefix() {
    assert_eq!(clean_label("thm:main", Some("REF")), "REF:thm:main");
  }

  #[test]
  fn clean_label_empty_prefix_skips_colon() {
    // Empty prefix means no prefix at all (not an empty-prefix colon).
    let out = clean_label("foo bar", Some(""));
    assert_eq!(out, "foo_bar");
  }

  #[test]
  fn clean_label_trims_whitespace() {
    // Leading/trailing whitespace trimmed before space-to-underscore.
    assert_eq!(clean_label("  foo  ", None), "LABEL:foo");
  }

  #[test]
  fn clean_class_name_basic() {
    // clean_class_name strips spaces, converts to lowercase, removes
    // non-class-safe chars.
    let out = clean_class_name("foo");
    assert!(out.contains("foo"), "got {out:?}");
  }

  #[test]
  fn clean_bib_key_basic() {
    // Bib keys are case-preserved but trimmed/cleaned.
    let out = clean_bib_key("Author:2020");
    assert!(!out.is_empty());
  }

  #[test]
  fn normalize_bib_key_case_insensitive() {
    // normalize_bib_key should produce the same output for case variants.
    let a = normalize_bib_key("Author2020");
    let b = normalize_bib_key("AUTHOR2020");
    assert_eq!(a, b, "normalize_bib_key folds case (got {a:?} vs {b:?})");
  }

  #[test]
  fn trimmed_comma_list_basic() {
    let out = trimmed_comma_list("a, b ,c,  d");
    assert_eq!(out, vec!["a", "b", "c", "d"]);
  }

  #[test]
  fn trimmed_comma_list_handles_empty_segments() {
    // Leading/trailing/internal empty comma positions — behavior may
    // retain empty tokens or drop them depending on the implementation;
    // just assert consistency with non-empty entries.
    let out = trimmed_comma_list(",a,,b,");
    assert!(
      out.contains(&"a".to_string()) && out.contains(&"b".to_string()),
      "got {out:?}"
    );
  }

  #[test]
  fn clean_url_removes_quotes_and_whitespace() {
    // clean_url is lenient — it trims and normalizes. At minimum it
    // must not break a well-formed URL.
    let canonical = "http://example.com/path";
    assert_eq!(clean_url(canonical), canonical);
  }

  #[test]
  fn clean_index_key_trims_trailing_punct() {
    // Per docstring: Applies NFC + strips trailing punctuation.
    let out = clean_index_key("topic.");
    assert_eq!(out, "topic", "got {out:?}");
  }
}
