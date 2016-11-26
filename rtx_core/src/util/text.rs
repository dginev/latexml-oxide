pub enum Delimiter {
  Parenthesis,
  Brace,
  Bracket
}
impl Delimiter {
  fn open(&self) -> char {
    use self::Delimiter::*;
    match *self {
      Parenthesis => '(',
      Brace => '{',
      Bracket => '['
    }
  }
  fn close(&self) -> char {
    use self::Delimiter::*;
    match *self {
      Parenthesis => ')',
      Brace => '}',
      Bracket => ']'
    }
  }
}

pub fn extract_bracketed(mut text: &mut String, delimiter: Option<Delimiter>) -> String {
  let open_delim = match delimiter {
    None => '(',
    Some(ref d) => d.open()
  };
  let close_delim = match delimiter {
    None => ')',
    Some(ref d) => d.close()
  };

  // println_stderr!("-- eb before: {:?}", text);
  let mut extracted = String::new();
  let mut level = 0;
  while !text.is_empty() {
    let c = text.remove(0);

    // termination clause goes first
    if c == close_delim {
      level -= 1;
      if level < 1 {
        break;
      }
    }
    // if we are inside the parens, record the char
    if level > 0 {
      extracted.push(c)
    }

    if c.is_whitespace() { // whitespaces are neutral
      continue
    } else if c == open_delim { // level up on open paren
      level += 1;
    } else {
      // regular chars out of () body should terminate the expression
      if level < 1 {
        *text = c.to_string() + text;
        break;
      }
    }
  }
  // println_stderr!("-- eb after: {:?}", text);
  extracted
}