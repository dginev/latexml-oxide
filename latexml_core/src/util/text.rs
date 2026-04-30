/// kinds of delimiters used by the binding replacement strings
pub enum Delimiter {
  /// for ()
  Parenthesis,
  /// for {}
  Brace,
  /// for []
  Bracket,
}
impl Delimiter {
  fn open(&self) -> char {
    use self::Delimiter::*;
    match *self {
      Parenthesis => '(',
      Brace => '{',
      Bracket => '[',
    }
  }
  fn close(&self) -> char {
    use self::Delimiter::*;
    match *self {
      Parenthesis => ')',
      Brace => '}',
      Bracket => ']',
    }
  }
}

/// Extract a bracketed subexpression at the start of a larger string. defaults to () delimiters.
pub fn extract_bracketed(text: &mut String, delimiter: Option<&Delimiter>) -> Option<String> {
  let open_delim = match delimiter {
    None => '(',
    Some(d) => d.open(),
  };
  let close_delim = match delimiter {
    None => ')',
    Some(d) => d.close(),
  };
  let mut has_open = false;
  let mut has_close = false;

  let mut extracted = String::new();
  let mut level = 0;
  while !text.is_empty() {
    match text.remove(0) {
      c if c == close_delim => {
        has_close = true;
        level -= 1;
        if level < 1 {
          break;
        } else {
          extracted.push(c);
        }
      },
      c if c == open_delim => {
        has_open = true;
        // level up on open paren
        level += 1;
        if level > 1 {
          extracted.push(c);
        }
      },
      c => {
        if level > 0 {
          // if we are inside the parens, record the char
          extracted.push(c)
        } else if !c.is_whitespace() {
          // whitespaces are neutral even outside of the delim blocks
          // regular chars out of () body should terminate the expression
          *text = c.to_string() + text;
          break;
        }
      },
    }
  }

  if has_open && has_close && level == 0 {
    Some(extracted)
  } else {
    // signal malformed
    None
  }
}

// Nice tip from https://users.rust-lang.org/t/trim-string-in-place/15809/18
pub fn trim_end_in_place(s: &mut String) {
  let trimmed = s.trim_end();
  s.truncate(trimmed.len());
}

// Nice tip from https://users.rust-lang.org/t/trim-string-in-place/15809/18
pub fn trim_start_in_place(s: &mut String) {
  let trimmed = s.trim_start();
  s.replace_range(..(s.len() - trimmed.len()), "");
}

// Nice tip from https://users.rust-lang.org/t/trim-string-in-place/15809/18
pub fn trim_in_place(s: &mut String) {
  trim_end_in_place(s);
  trim_start_in_place(s);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extract_bracketed_parens_default() {
    let mut s = "(abc)rest".to_string();
    let out = extract_bracketed(&mut s, None).expect("well-formed parens");
    assert_eq!(out, "abc");
    assert_eq!(s, "rest");
  }

  #[test]
  fn extract_bracketed_braces() {
    let mut s = "{foo}tail".to_string();
    let out = extract_bracketed(&mut s, Some(&Delimiter::Brace)).expect("braces");
    assert_eq!(out, "foo");
    assert_eq!(s, "tail");
  }

  #[test]
  fn extract_bracketed_brackets() {
    let mut s = "[opt]body".to_string();
    let out = extract_bracketed(&mut s, Some(&Delimiter::Bracket)).expect("brackets");
    assert_eq!(out, "opt");
    assert_eq!(s, "body");
  }

  #[test]
  fn extract_bracketed_nested() {
    let mut s = "(a(b)c)rest".to_string();
    let out = extract_bracketed(&mut s, None).expect("nested parens");
    assert_eq!(out, "a(b)c");
    assert_eq!(s, "rest");
  }

  #[test]
  fn extract_bracketed_malformed_returns_none() {
    let mut s = "(unclosed".to_string();
    let out = extract_bracketed(&mut s, None);
    assert!(out.is_none(), "unclosed bracket returns None");
  }

  #[test]
  fn trim_end_in_place_strips_trailing_space() {
    let mut s = "foo   ".to_string();
    trim_end_in_place(&mut s);
    assert_eq!(s, "foo");
  }

  #[test]
  fn trim_end_in_place_preserves_leading() {
    let mut s = "  foo  ".to_string();
    trim_end_in_place(&mut s);
    assert_eq!(s, "  foo");
  }

  #[test]
  fn trim_start_in_place_strips_leading_space() {
    let mut s = "   foo".to_string();
    trim_start_in_place(&mut s);
    assert_eq!(s, "foo");
  }

  #[test]
  fn trim_start_in_place_preserves_trailing() {
    let mut s = "  foo  ".to_string();
    trim_start_in_place(&mut s);
    assert_eq!(s, "foo  ");
  }

  #[test]
  fn trim_in_place_strips_both_ends() {
    let mut s = "  foo bar  ".to_string();
    trim_in_place(&mut s);
    assert_eq!(s, "foo bar");
  }

  #[test]
  fn trim_in_place_empty_input() {
    let mut s = "   ".to_string();
    trim_in_place(&mut s);
    assert_eq!(s, "");
  }

  #[test]
  fn trim_in_place_no_whitespace() {
    let mut s = "already_clean".to_string();
    trim_in_place(&mut s);
    assert_eq!(s, "already_clean");
  }
}
