use crate::prelude::*;
use base64::Engine as _;

/// Helper to build an invocation without requiring `?` context.
fn invoke(cs: Token, args: Vec<Tokens>) -> Vec<Token> {
  let result: Result<Tokens> = build_invocation(cs, args.into_iter().map(Into::into).collect());
  result.map(|t| t.unlist()).unwrap_or_default()
}

//======================================================================
// Region 1: Preamble (Perl lines 1-33)
// RequireResource, RequirePackage, initial setup
//======================================================================

//======================================================================
// Region 4: Low-level string stuff (Perl lines 276-360)
// Character mapping, lstRescan, readRawString/Lines/File
//======================================================================

/// Perl: $lst_charmapping — maps special chars to TeX control sequences.
fn lst_char_mapping(ch: &str, upquote: bool) -> Option<&'static str> {
  match ch {
    "#" => Some("\\#"),
    "$" => Some("\\textdollar"),
    "&" => Some("\\&"),
    "'" => Some(if upquote {
      "\\textquotesingle"
    } else {
      "\\textquoteright"
    }),
    "*" => Some("\\textasteriskcentered"),
    "<" => Some("\\textless"),
    ">" => Some("\\textgreater"),
    "\\" => Some("\\textbackslash"),
    "^" => Some("\\textasciicircum"),
    "_" => Some("\\textunderscore"),
    "`" => Some(if upquote {
      "\\textasciigrave"
    } else {
      "\\textquoteleft"
    }),
    "{" => Some("\\textbraceleft"),
    "}" => Some("\\textbraceright"),
    "%" => Some("\\%"),
    "|" => Some("\\textbar"),
    "~" => Some("\\textasciitilde"),
    _ => None,
  }
}

/// Perl: lstRescan — remap special chars via character mapping.
fn lst_rescan(tokens: Option<Tokens>) -> Option<Tokens> {
  let upquote = lst_get_boolean("upquote");
  tokens.map(|toks| {
    let remapped: Vec<Token> = toks
      .unlist()
      .iter()
      .flat_map(|t| {
        if t.get_catcode() == Catcode::OTHER {
          let ch = t.to_string();
          if let Some(cs) = lst_char_mapping(&ch, upquote) {
            return vec![T_CS!(cs)];
          }
        }
        vec![*t]
      })
      .collect();
    Tokens::new(remapped)
  })
}

/// Perl: listingsReadRawLines — read raw lines until \end{$environment}
pub fn listings_read_raw_lines(environment: &str) -> String {
  let mut lines = Vec::new();
  gullet::read_raw_line(); // Ignore 1st line (following \begin{...})
  let end_re = Regex::new(&format!(
    "^\\s*\\\\end\\{{{}\\}}(.*?)$",
    regex::escape(environment)
  ))
  .unwrap();
  while let Some(line) = gullet::read_raw_line() {
    if let Some(caps) = end_re.captures(&line) {
      let rest = caps.get(1).map_or("", |m| m.as_str()).to_string();
      if !rest.is_empty() {
        gullet::unread(Tokenize!(&rest));
        gullet::unread(Tokens::new(vec![T_CR!()]));
      }
      break;
    }
    lines.push(line);
  }
  lines.join("\n")
}

/// Read a balanced group from a token slice, starting at tokens[*pos].
/// *pos should point to a T_BEGIN token. Returns content tokens (without outer braces).
/// Advances *pos past the closing T_END.
fn read_balanced_group(tokens: &[Token], pos: &mut usize) -> Vec<Token> {
  let mut result = Vec::new();
  if *pos >= tokens.len() || tokens[*pos].get_catcode() != Catcode::BEGIN {
    return result;
  }
  *pos += 1; // skip T_BEGIN
  let mut level = 1i32;
  while *pos < tokens.len() && level > 0 {
    match tokens[*pos].get_catcode() {
      Catcode::BEGIN => {
        level += 1;
        result.push(tokens[*pos]);
      },
      Catcode::END => {
        level -= 1;
        if level > 0 {
          result.push(tokens[*pos]);
        }
      },
      _ => {
        result.push(tokens[*pos]);
      },
    }
    *pos += 1;
  }
  result
}

/// Perl: TokenizeBalanced — tokenize a string with current catcodes, then balance groups.
fn tokenize_balanced(text: &str) -> Vec<Token> {
  let tokens = latexml_core::mouth::tokenize(text);
  let mut toks: Vec<Token> = tokens.unlist();
  let mut level: i32 = 0;
  for t in &toks {
    match t.get_catcode() {
      Catcode::BEGIN => level += 1,
      Catcode::END => level -= 1,
      _ => {},
    }
  }
  while level > 0 {
    toks.push(T_END!());
    level -= 1;
  }
  while level < 0 {
    toks.insert(0, T_BEGIN!());
    level += 1;
  }
  toks
}

/// Perl: listingsReadRawString — read until closing delimiter token.
/// Handles mathescape: within $...$, content is read with normal catcodes
/// and preserved as TeX (backslashes intact). Outside math, CS tokens have \ stripped.
/// Returns UnTeX'd string representation.
fn listings_read_raw_string(until: Option<&Token>) -> String {
  let mathescape = lst_get_boolean("mathescape");
  let mut inmath = false;
  let mut tokens: Vec<Token> = Vec::new();

  while let Ok(Some(token)) = gullet::read_token() {
    if let Some(until_tok) = until {
      // Perl `listings.sty.ltxml:291` matches by string only —
      // `last if $until and $token->getString eq $until->getString;`.
      // The verbatim catcode switch makes the `until` delimiter's
      // catcode (e.g. END for `}`) differ from the body's reading
      // catcode (OTHER), so a strict (text, code) match would never
      // trigger and the body greedily consumes input. Match on
      // interned text identity alone, mirroring Perl.
      if token.text == until_tok.text {
        break;
      }
    }
    // Check for mathescape $ toggle
    if mathescape && token.text == pin!("$") {
      inmath = !inmath;
      tokens.push(T_OTHER!("$"));
      continue;
    }
    let cc = token.get_catcode();
    if inmath && cc == Catcode::BEGIN {
      // In math mode with {, read balanced group and preserve
      tokens.push(T_BEGIN!());
      if let Ok(balanced) = gullet::read_balanced(ExpansionLevel::Off, false, false) {
        tokens.extend(balanced.unlist());
      }
      tokens.push(T_END!());
    } else if !inmath && cc.is_active_or_cs() {
      // Outside math: convert CS tokens to plain text, preserving the backslash
      // In verbatim listing context, \end should appear as literal characters \ e n d
      let name = token.to_string();
      for c in name.chars() {
        tokens.push(Token {
          text: arena::pin_char(c),
          code: Catcode::OTHER,
        });
      }
    } else {
      tokens.push(token);
    }
  }
  // Remove trailing spaces
  while tokens
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    tokens.pop();
  }
  // UnTeX: convert tokens to string representation
  let mut result = String::new();
  for t in &tokens {
    let cc = t.get_catcode();
    if cc.is_active_or_cs() {
      // Preserve CS with backslash
      let s = t.to_string();
      result.push_str(&s);
      result.push(' '); // CS names need trailing space for re-tokenization
    } else {
      result.push_str(&t.to_string());
    }
  }
  result.truncate(result.trim_end().len());
  result
}

/// Perl: listingsReadRawFile — read entire file contents as string.
fn listings_read_raw_file(file: &str) -> Option<String> {
  let filename = file.to_string();
  if let Some(path) = find_file(&filename, None) {
    std::fs::read_to_string(&path).ok()
  } else {
    log::warn!("Can't read listings file '{}'", filename);
    None
  }
}

//======================================================================
// Region 5: KeyVal management helpers (Perl lines 362-537)
// lstActivate, class/word management
//======================================================================

/// Perl: lstUnGroup — strip outer {} if there's only a single group.
fn lst_un_group(tokens: Option<Tokens>) -> Option<Tokens> {
  tokens.map(|toks| {
    let mut t = toks.unlist();
    if t.len() >= 2
      && t
        .first()
        .is_some_and(|tok| tok.get_catcode() == Catcode::BEGIN)
      && t
        .last()
        .is_some_and(|tok| tok.get_catcode() == Catcode::END)
    {
      let mut groups = 0;
      let mut level = 0;
      for tok in t.iter() {
        let cc = tok.get_catcode();
        if cc == Catcode::END {
          level -= 1;
        } else if cc == Catcode::BEGIN {
          if level == 0 {
            groups += 1;
          }
          level += 1;
        }
      }
      if groups == 1 {
        t.pop();
        t.remove(0);
      }
    }
    Tokens::new(t)
  })
}

/// Perl: lstSplit — split comma-separated, strip whitespace and TeX comments.
fn lst_split(stuff: &Option<Tokens>) -> Vec<String> {
  match stuff {
    None => vec![],
    Some(toks) => {
      let mut s = lst_un_group(Some(toks.clone()))
        .map(|t| t.to_string())
        .unwrap_or_default();
      // Strip TeX comments: %...\n
      let comment_re = Regex::new(r"%.*?\n\s*").unwrap();
      s = comment_re.replace_all(&s, "").to_string();
      // Strip whitespace
      s = s.replace(char::is_whitespace, "");
      s.split(',')
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
    },
  }
}

/// Perl: lstDeslash — strip TeX's quoting (leading backslash).
fn lst_deslash(string: &str) -> String {
  let s = string.to_string();
  if let Some(stripped) = s.strip_prefix('\\') {
    stripped.to_string()
  } else {
    s
  }
}

/// Perl: lstRegexp — convert a string of TeX chars to a Perl-compatible regexp.
fn lst_regexp(chars: &str) -> String { regex::escape(&lst_deslash(chars)) }

/// Perl: lst_splitDelimiters — split a delimiter token list into (open, close) parts.
/// The format is `{open}{close}` where the two TeX groups contain the delimiters.
/// First group = open delimiter, second group = close delimiter.
fn lst_split_delimiters(delims: &Tokens) -> (String, String) {
  // Perl listings.sty.ltxml:629 lst_splitDelimiters
  //   my @t  = grep { !Equals($_, T_BEGIN) } $delims->unlist;
  //   my @t1 = ();
  //   if (scalar(@t) == 2) { @t1 = ($t[0]); @t = ($t[1]); }
  //   else {
  //     while (@t && !Equals($t[0], T_END)) { push(@t1, shift(@t)); }
  //     @t = grep { !Equals($_, T_END) } @t; }
  //   return (Tokens(@t1), Tokens(@t));
  // Two-token short-circuit is what handles bare `<>` (open='<', close='>'),
  // which is how the XML language registers its `tag=**[s]<>` delimiter.
  let toks: Vec<Token> = delims
    .unlist_ref()
    .iter()
    .filter(|t| **t != T_BEGIN!())
    .copied()
    .collect();
  let (open_toks, close_toks): (Vec<Token>, Vec<Token>) = if toks.len() == 2 {
    (vec![toks[0]], vec![toks[1]])
  } else {
    let mut it = toks.into_iter();
    let mut open = Vec::new();
    let mut close = Vec::new();
    let mut saw_end = false;
    for t in it.by_ref() {
      if t == T_END!() {
        saw_end = true;
        break;
      }
      open.push(t);
    }
    if saw_end {
      for t in it {
        if t == T_END!() {
          continue;
        }
        close.push(t);
      }
    }
    (open, close)
  };
  let open_str = lst_deslash(&Tokens::new(open_toks).to_string());
  let close_str = lst_deslash(&Tokens::new(close_toks).to_string());
  (open_str, close_str)
}

/// Perl: lstGetLiteral — get string value from LST@key state.
fn lst_get_literal(value: &str) -> String {
  let key = s!("LST@{value}");
  let v = state::with_value(&key, |s| s.map(|s| s.to_string()).unwrap_or_default());
  // Strip outer {} if present
  if v.starts_with('{') && v.ends_with('}') {
    v[1..v.len() - 1].to_string()
  } else {
    v
  }
}

/// Perl: lstGetBoolean — get boolean from LST@key state.
fn lst_get_boolean(value: &str) -> bool { lst_get_literal(value) == "true" }

/// Perl: lstGetNumber — get numeric value from LST@key state.
fn lst_get_number(value: &str) -> i64 {
  let key = s!("LST@{value}");
  state::with_value(&key, |v| match v {
    Some(Stored::Number(n)) => n.value_of(),
    Some(Stored::Tokens(t)) => t.to_string().parse().unwrap_or(0),
    Some(v) => v.to_string().parse().unwrap_or(0),
    None => 0,
  })
}

/// Perl: lstGetTokens — get tokens from LST@key state.
fn lst_get_tokens(value: &str) -> Tokens {
  let key = s!("LST@{value}");
  match state::lookup_value(&key) {
    Some(Stored::Tokens(t)) => lst_un_group(Some(t)).unwrap_or(Tokens!()),
    _ => Tokens!(),
  }
}

/// Perl: lstClassName — build numbered class name like "keywords2".
fn lst_class_name(class: &str, n: Option<i64>) -> String {
  let mut n = n.unwrap_or(1);
  n += lst_get_number("classoffset");
  if n <= 1 {
    class.to_string()
  } else {
    format!("{class}{n}")
  }
}

/// Perl: lstPushValueLocally — like PushValue but local (not global).
fn lst_push_value_locally(list: &str, values: Vec<Token>) {
  let key = list;
  let prev = match state::lookup_value(key) {
    Some(Stored::Tokens(t)) => t.unlist(),
    _ => vec![],
  };
  let mut combined = prev;
  combined.extend(values);
  state::assign_value(key, Stored::Tokens(Tokens::new(combined)), None);
}

/// Perl: lstSetClassStyle — define properties of a styling class.
fn lst_set_class_style(class: &str, style: Option<Tokens>, props: Vec<(&str, &str)>) {
  let map_key = "LST_CLASSES";
  // Get or create the class entry
  if let Some(style_toks) = &style {
    let stylestring = style_toks.to_string().trim().to_string();
    if stylestring.ends_with("style") || {
      let re = Regex::new(r"style\d*$").unwrap();
      re.is_match(&stylestring)
    } {
      // If names a style, convert into indirect class reference
      let classref = Regex::new(r"style(\d*)$")
        .unwrap()
        .replace(&stylestring, "s$1")
        .to_string();
      // remove explicit styling (begin)
      let begin_key = s!("{map_key}@{class}@begin");
      state::assign_value(&begin_key, Stored::None, None);
      // add indirect to class
      let class_key = s!("{map_key}@{class}@class");
      state::assign_value(&class_key, Stored::String(arena::pin(&classref)), None);
    } else {
      // Otherwise, it's presumably TeX styling
      let class_key = s!("{map_key}@{class}@class");
      state::assign_value(&class_key, Stored::None, None);
      let begin_key = s!("{map_key}@{class}@begin");
      state::assign_value(&begin_key, Stored::Tokens(style_toks.clone()), None);
    }
  }
  // Set cssclass based on class name
  let cssclass = class.strip_suffix('s').unwrap_or(class).to_string();
  let css_key = s!("{map_key}@{class}@cssclass");
  state::assign_value(&css_key, Stored::String(arena::pin(&cssclass)), None);

  // Apply extra properties
  for (k, v) in props {
    let prop_key = s!("{map_key}@{class}@{k}");
    state::assign_value(&prop_key, Stored::String(arena::pin(v)), None);
  }
}

/// Perl listings.sty.ltxml:506-513 lstSetClassWords
///   First delete existing words assigned to $class, then add the new set.
fn lst_set_class_words(class: &str, words: &Option<Tokens>, prefix: Option<&str>) {
  lst_clear_class_words(class, false);
  lst_add_class_words(class, words, prefix);
}

/// Mirrors Perl listings.sty.ltxml:531-537 lstDeleteClass
///   foreach my $word (keys %$wordslist) {
///     delete $$wordslist{$word}{class}
///       if (($$wordslist{$word}{class} || '') eq $class)
///       || (($$wordslist{$word}{class} || '') =~ /^\Q$class\E\d/); }
/// `digit_suffix=true` enables the trailing-digit match (e.g. `class='keywords'`
/// also matches 'keywords1', 'keywords2', …); lstSetClassWords uses only the
/// exact-equals branch (digit_suffix=false).
fn lst_clear_class_words(class: &str, digit_suffix: bool) {
  let words: Vec<String> = match state::lookup_value("LST_WORD_LIST") {
    Some(Stored::Strings(list)) => list.iter().map(|sym| arena::to_string(*sym)).collect(),
    _ => return,
  };
  for word in &words {
    let key = s!("LST_WORDS@{word}@class");
    let assigned = state::with_value(&key, |v| v.map(|s| s.to_string()).unwrap_or_default());
    let assigned_str = assigned.as_str();
    let matches = assigned_str == class
      || (digit_suffix
        && assigned_str.starts_with(class)
        && assigned_str[class.len()..]
          .chars()
          .next()
          .is_some_and(|c| c.is_ascii_digit()));
    if matches {
      state::assign_value(&key, Stored::None, None);
    }
  }
}

/// Perl: lstAddClassWords — add words to a class.
fn lst_add_class_words(class: &str, words: &Option<Tokens>, prefix: Option<&str>) {
  let word_list = lst_split(words);
  for mut word in word_list {
    if let Some(pfx) = prefix {
      word = format!("{pfx}{word}");
    }
    let key = s!("LST_WORDS@{word}@class");
    // Only set if not already in a class
    if state::with_value(&key, |v| v.is_none()) {
      state::assign_value(&key, Stored::String(arena::pin(class)), None);
      // Track the word in the word list for case-insensitive duplication
      let list_key = "LST_WORD_LIST";
      let mut list = state::with_value(list_key, |v| match v {
        Some(Stored::Strings(v)) => v.to_vec(),
        _ => Vec::new(),
      });
      list.push(arena::pin(&word));
      state::assign_value(list_key, Stored::Strings(list.into()), None);
    }
  }
}

/// Perl: lstDeleteClassWords — remove words from a class.
fn lst_delete_class_words(class: &str, words: &Option<Tokens>, prefix: Option<&str>) {
  let word_list = lst_split(words);
  for mut word in word_list {
    if let Some(pfx) = prefix {
      word = format!("{pfx}{word}");
    }
    let key = s!("LST_WORDS@{word}@class");
    let matches_class = state::with_value(&key, |v| v.map(|s| s.eq_text(class)).unwrap_or(false));
    if matches_class {
      state::assign_value(&key, Stored::None, None);
    }
  }
}

/// Perl listings.sty.ltxml:531-537 lstDeleteClass — delete words belonging to
/// `class` OR `class\d` (e.g. 'keywords' clears 'keywords1', 'keywords2', …).
/// Without this `lstClearLanguage` (called on every `language=` switch) leaves
/// prior keyword sets attached, so later listings highlight stale keywords from
/// previously selected languages.
fn lst_delete_class(class: &str) {
  lst_clear_class_words(class, true);
}

/// Perl L617-622: lstDeleteDelimiterKind — delete delimiters whose class starts with `kind`.
fn lst_delete_delimiter_kind(kind: &str) {
  // Snapshot the keys-string via with_value so the subsequent per-key
  // class lookups don't race against the outer lookup holding a clone.
  let keys_str = state::with_value("LST_DELIM_KEYS", |v| {
    v.map(|s| s.to_string()).unwrap_or_default()
  });
  if !keys_str.is_empty() {
    for open_key in keys_str.split_whitespace() {
      let open_key = open_key.trim();
      if open_key.is_empty() {
        continue;
      }
      let class_key = s!("LST_DELIM@{}@class", open_key);
      let class_starts = state::with_value(&class_key, |v| {
        v.map(|s| s.starts_with_text(kind)).unwrap_or(false)
      });
      if class_starts {
        // Remove delimiter entries
        state::assign_value(&class_key, Stored::default(), None);
        state::assign_value(&s!("LST_DELIM@{}@open", open_key), Stored::default(), None);
        state::assign_value(&s!("LST_DELIM@{}@close", open_key), Stored::default(), None);
        state::assign_value(
          &s!("LST_DELIM@{}@recursive", open_key),
          Stored::default(),
          None,
        );
        state::assign_value(
          &s!("LST_DELIM@{}@invisible", open_key),
          Stored::default(),
          None,
        );
      }
    }
  }
}

/// Perl: lstClearLanguage — clear keyword/comment/string definitions before activating a new
/// language. Note: Perl clears 'textcs' (not 'texcss'), so texcs words survive the clear.
fn lst_clear_language() {
  lst_delete_class("keywords");
  lst_delete_class("otherkeywords");
  lst_delete_class("endkeywords");
  lst_delete_class("directives");
  lst_delete_class("textcs"); // Note: Perl typo? texcs uses class 'texcss', not 'textcs'
  lst_delete_delimiter_kind("comment");
  lst_delete_delimiter_kind("string");
}

//======================================================================
// Region 6: Delimiter parsing (Perl lines 539-650)
//======================================================================

/// Perl: lstAddDelimiter — add a delimiter (comment, string, etc) definition.
fn lst_add_delimiter(
  kind: &str,
  type_str: &str,
  style: &str,
  delims: Option<Tokens>,
  recursive: bool,
) {
  let type_str = type_str.to_string();
  // Perl: $invisible = ($type =~ /^(?:bd|b|d|l|s|n)i$/) || ($type =~ /^i(?:bd|b|d|l|s|n)$/);
  // Only strip 'i' from specific invisible marker patterns, not from type names like "directive"
  let base_types = ["bd", "b", "d", "l", "s", "n"];
  let invisible = base_types
    .iter()
    .any(|bt| type_str == format!("{bt}i") || type_str == format!("i{bt}"));
  let type_clean = if invisible {
    // Remove first 'i' occurrence
    let mut s = type_str.clone();
    if let Some(pos) = s.find('i') {
      s.remove(pos);
    }
    s
  } else {
    type_str.clone()
  };

  let delim_str = delims
    .as_ref()
    .map(|d| lst_un_group(Some(d.clone())).unwrap().to_string())
    .unwrap_or_default();

  let mut kind_override: Option<String> = None;
  // Compute open, close, close_re, and quoted pattern.
  // NOTE: Rust's `regex` crate does NOT support lookbehinds (?<!...).
  // Instead we use `quoted` patterns (matching Perl's approach) to consume escaped
  // delimiters before the close_re can match them.
  let (open_str, close_str, close_re, quoted) = match type_clean.as_str() {
    "l" => {
      // Line: close is till end of line
      // Perl: $closere = "(?=\n)" — lookahead not supported by regex crate.
      // Use sentinel "__NEWLINE__" for special zero-width handling.
      let open = lst_deslash(&delim_str);
      (
        open,
        String::new(),
        "__NEWLINE__".to_string(),
        String::new(),
      )
    },
    "s" | "n" => {
      // String/Nested: different open & close delimiters
      // Use Perl-like token-level splitting for proper brace handling.
      // For type='n' Perl listings.sty.ltxml:583 also sets `$keys{nested} = 1`
      // — registered below as LST_DELIM@<open>@nested so lst_process_internal
      // can re-allow the same open inside a non-recursive nested span.
      if let Some(delim_toks) = &delims {
        let (open, close) = lst_split_delimiters(delim_toks);
        let close_re = regex::escape(&close);
        (open, close, close_re, String::new())
      } else {
        let open = lst_deslash(&delim_str);
        let close_re = regex::escape(&open);
        (open.clone(), open.clone(), close_re, String::new())
      }
    },
    "b" => {
      // Balanced: same delim open & close; but not when slashed
      // Perl: $closere = "(?<!\\)$openre"; $quoted = "\\$openre"
      // Rust: no lookbehind; close_re is just the open regex;
      //       quoted handles \<delim> so it's consumed before end_re
      let open = lst_deslash(&delim_str);
      let open_re = lst_regexp(&delim_str);
      let quoted = format!("\\\\{open_re}");
      (open.clone(), open, open_re, quoted)
    },
    "d" => {
      // Doubled: same delim; not when doubled
      // Perl: $closere = "(?<!$openre)$openre(?!$openre)"; $quoted = $openre.$openre
      // Rust: no lookahead/lookbehind; use simple close_re. Doubled delims are consumed
      // by the quoted_re before close_re can match, so simple match is correct.
      let open = lst_deslash(&delim_str);
      let open_re = lst_regexp(&delim_str);
      let quoted = format!("{open_re}{open_re}");
      (open.clone(), open, open_re, quoted)
    },
    "directive" => {
      // Perl: $kind = $type . 's' = "directives"
      kind_override = Some("directives".to_string());
      let open = lst_deslash(&delim_str);
      // Perl: $closere = "(?=\W)" — lookahead not supported by regex crate.
      // Use sentinel "__NONWORD__" for special zero-width handling in lst_process_internal.
      (
        open,
        String::new(),
        "__NONWORD__".to_string(),
        String::new(),
      )
    },
    _ => {
      let open = lst_deslash(&delim_str);
      let open_re = lst_regexp(&delim_str);
      (open.clone(), open, open_re, String::new())
    },
  };

  if !open_str.is_empty() {
    // Perl: $kind can be overridden by type (e.g. "directive" → "directives")
    let kind = kind_override.as_deref().unwrap_or(kind);

    // Perl: process $style parameter to determine base class name
    // "commentstyle" → "comments", "stringstyle" → "strings", etc.
    let style_re = Regex::new(r"style(\d*)$").unwrap();
    let base_class = if style_re.is_match(style) {
      style_re.replace(style, "s$1").to_string()
    } else {
      kind.to_string()
    };

    // Perl: $class = $class . ToString($open) . ToString($close)
    let class = format!("{base_class}{open_str}{close_str}");
    // eprintln!("lst_add_delimiter: kind={kind:?} type={type_clean:?} open={open_str:?}
    // close={close_str:?} close_re={close_re:?} class={class:?} base_class={base_class:?}");
    // Store delimiter info in state
    let key_open = s!("LST_DELIM@{open_str}@open");
    let key_close = s!("LST_DELIM@{open_str}@close");
    let key_class = s!("LST_DELIM@{open_str}@class");
    let key_recursive = s!("LST_DELIM@{open_str}@recursive");
    let key_invisible = s!("LST_DELIM@{open_str}@invisible");
    state::assign_value(
      &key_open,
      Stored::String(arena::pin(regex::escape(&open_str))),
      None,
    );
    state::assign_value(&key_close, Stored::String(arena::pin(&close_re)), None);
    state::assign_value(&key_class, Stored::String(arena::pin(&class)), None);
    state::assign_value(&key_recursive, Stored::Bool(recursive), None);
    if invisible {
      state::assign_value(&key_invisible, Stored::Bool(true), None);
    }
    if type_clean == "n" {
      // Perl listings.sty.ltxml:583 sets `$keys{nested} = 1` for type='n'.
      let key_nested = s!("LST_DELIM@{open_str}@nested");
      state::assign_value(&key_nested, Stored::Bool(true), None);
    }
    if !quoted.is_empty() {
      let key_quoted = s!("LST_DELIM@{open_str}@quoted");
      state::assign_value(&key_quoted, Stored::String(arena::pin(&quoted)), None);
    }
    // Register this delimiter in the delimiter list
    lst_push_value_locally("LST_DELIM_KEYS", vec![T_OTHER!(&open_str)]);

    // Perl L593-607: lstSetClassStyle with openTeX/closeTeX.
    // If style is TeX markup (not a style name), prepend it to open tokens.
    let style_is_markup =
      !style_re.is_match(style) && !style.is_empty() && style != "None" && style.contains('\\'); // TeX markup contains backslash
    let style_tokens = if style_is_markup {
      mouth::tokenize_internal(style)
    } else {
      Tokens!()
    };
    let open_tex = if invisible {
      if style_is_markup {
        style_tokens
      } else {
        Tokens!()
      }
    } else if style_is_markup {
      // Perl: Tokens($styleTeX->unlist, $open)
      let mut toks = style_tokens.unlist();
      toks.push(T_OTHER!(&open_str));
      Tokens::new(toks)
    } else {
      Tokens::new(vec![T_OTHER!(&open_str)])
    };
    let close_tex = if invisible || close_str.is_empty() {
      Tokens!()
    } else {
      Tokens::new(vec![T_OTHER!(&close_str)])
    };

    // Set parent class (Perl: class => $oldclass)
    let class_key = s!("LST_CLASSES@{class}@class");
    state::assign_value(&class_key, Stored::String(arena::pin(&base_class)), None);
    // Don't set cssclass on artificial class — Perl's regex /^(\w+?)s?$/ won't match
    // class names containing delimiter chars like "comments{}". Let parent chain provide it.
    if !open_tex.is_empty() {
      let begin_key = s!("LST_CLASSES@{class}@begin");
      state::assign_value(&begin_key, Stored::Tokens(open_tex), None);
    }
    if !close_tex.is_empty() {
      let end_key = s!("LST_CLASSES@{class}@end");
      state::assign_value(&end_key, Stored::Tokens(close_tex), None);
    }
  }
}

/// Perl: lstSetCharacterClass — set characters as letter/digit/other.
fn lst_set_character_class(class: &str, chars: &Tokens) {
  for ch in chars.unlist_ref() {
    let ch_re = ch.with_str(|ch_str| regex::escape(&lst_deslash(ch_str)));
    // Remove from all classes, then add to target
    for cls in &["letter", "digit", "other"] {
      let key = s!("LST_CHAR@{cls}@{ch_re}");
      state::assign_value(&key, Stored::None, None);
    }
    let key = s!("LST_CHAR@{class}@{ch_re}");
    state::assign_value(&key, Stored::Bool(true), None);
  }
}

/// Build literate substitution entries from state.
/// Returns (pattern_string, replacement_tokens, protected_flag) triples.
fn build_literate_entries() -> Vec<(String, Tokens, bool)> {
  let keys: Vec<String> = match state::lookup_value("LST_LITERATE_KEYS") {
    Some(Stored::Tokens(t)) => t.unlist_ref().iter().map(|tok| tok.to_string()).collect(),
    _ => return Vec::new(),
  };
  let mut entries = Vec::new();
  for pattern in &keys {
    let repl_key = s!("LST_LIT@{pattern}");
    let prot_key = s!("LST_LIT@{pattern}@protected");
    if let Some(Stored::Tokens(replacement)) = state::lookup_value(&repl_key) {
      let protected = state::with_value(&prot_key, |v| matches!(v, Some(Stored::Bool(true))));
      entries.push((pattern.clone(), replacement, protected));
    }
  }
  entries
}

/// Build a regex that matches any literate pattern.
/// If `inner_only` is true, only include non-protected patterns.
fn build_literate_re(inner_only: bool) -> Option<Regex> {
  let entries = build_literate_entries();
  let patterns: Vec<String> = entries
    .iter()
    .filter(|(_, _, protected)| !inner_only || !protected)
    .map(|(pat, ..)| regex::escape(pat))
    .collect();
  if patterns.is_empty() {
    None
  } else {
    Regex::new(&format!("^({})", patterns.join("|"))).ok()
  }
}

/// Build a regex character class string from the character table in state.
/// Enumerates all printable ASCII chars and checks if they belong to `class`
/// (letter, digit, or other) via individual `LST_CHAR@{class}@{escaped}` keys.
/// Perl equivalent: `join('', sort keys %{$$characters{$class}})`
fn build_char_class(class: &str) -> String {
  let mut result = String::new();
  // Check all printable ASCII chars (space through tilde)
  for b in 0x20u8..=0x7Eu8 {
    let c = b as char;
    let escaped = regex::escape(&c.to_string());
    let key = s!("LST_CHAR@{class}@{escaped}");
    if state::with_value(&key, |v| matches!(v, Some(Stored::Bool(true)))) {
      // Escape chars that are special inside regex character classes
      match c {
        ']' | '\\' | '^' | '-' => {
          result.push('\\');
          result.push(c);
        },
        _ => result.push(c),
      }
    }
  }
  // Also check extended chars (128-255) for extendedchars support
  for code in 128u32..=255 {
    if let Some(c) = char::from_u32(code) {
      let escaped = regex::escape(&c.to_string());
      let key = s!("LST_CHAR@{class}@{escaped}");
      if state::with_value(&key, |v| matches!(v, Some(Stored::Bool(true)))) {
        result.push(c);
      }
    }
  }
  result
}

//======================================================================
// Region 9: The listing parser (Perl lines 1234-1559)
// lstProcess, lstProcess_internal, class begin/end, line constructors
//======================================================================

/// Perl: lstClassBegin — generate opening tokens for a styled class.
/// Collects delimiter chars and styling tokens separately. Delimiter chars
/// come first (in default font), then styling is scoped in a group so
/// it doesn't affect the close delimiter chars from lstClassEnd.
fn lst_class_begin(classname: &str) -> Vec<Token> {
  let mut delim_tokens = Vec::new();
  let mut style_tokens = Vec::new();
  let mut css_classes = Vec::new();

  if classname == "spaces" {
    css_classes.push("space".to_string());
  }

  let mut current_class = Some(classname.to_string());
  let mut is_leaf = true;
  while let Some(ref cname) = current_class {
    // Look up cssclass
    let css_key = s!("LST_CLASSES@{cname}@cssclass");
    if let Some(css) = state::lookup_value(&css_key) {
      let css_str = css.to_string();
      if !css_str.is_empty() {
        css_classes.push(css_str);
      }
    }
    // Look up begin tokens
    let begin_key = s!("LST_CLASSES@{cname}@begin");
    if let Some(Stored::Tokens(begin)) = state::lookup_value(&begin_key) {
      if let Some(rescanned) = lst_rescan(Some(begin)) {
        if is_leaf {
          // Leaf class tokens are delimiter chars — emit before styling
          delim_tokens.extend(rescanned.unlist());
        } else {
          // Parent class tokens are styling (e.g. \itshape) — scope in a group
          style_tokens.extend(rescanned.unlist());
        }
      }
    }
    // Follow class chain
    let class_key = s!("LST_CLASSES@{cname}@class");
    current_class = state::lookup_value(&class_key)
      .map(|v| v.to_string())
      .filter(|s| !s.is_empty());
    is_leaf = false;
  }

  // Deduplicate and sort CSS classes (matching Perl's addSSValues sort behavior)
  let mut seen = rustc_hash::FxHashSet::default();
  let mut deduped: Vec<String> = css_classes
    .iter()
    .filter(|c| seen.insert((*c).clone()))
    .map(|c| format!("ltx_lst_{c}"))
    .collect();
  deduped.sort();
  let css_string = deduped.join(" ");

  let mut result = vec![T_BEGIN!(), T_CS!("\\@listingGroup"), T_BEGIN!()];
  result.extend(ExplodeText!(&css_string));
  result.push(T_END!());
  result.push(T_BEGIN!());
  // Perl: @open built with unshift — parent style before leaf delimiters.
  // Style tokens (e.g. \itshape, \color{green}) apply to delimiters and content.
  result.extend(style_tokens);
  result.extend(delim_tokens);
  result
}

/// Perl: lstClassEnd — generate closing tokens for a styled class.
/// Mirrors lstClassBegin: close the style group first, then emit delimiter chars.
fn lst_class_end(classname: &str) -> Vec<Token> {
  let mut delim_tokens = Vec::new();
  let mut current_class = Some(classname.to_string());
  let mut is_leaf = true;
  while let Some(ref cname) = current_class {
    let end_key = s!("LST_CLASSES@{cname}@end");
    if let Some(Stored::Tokens(end)) = state::lookup_value(&end_key) {
      if let Some(rescanned) = lst_rescan(Some(end)) {
        if is_leaf {
          delim_tokens.extend(rescanned.unlist());
        }
      }
    }
    let class_key = s!("LST_CLASSES@{cname}@class");
    current_class = state::lookup_value(&class_key)
      .map(|v| v.to_string())
      .filter(|s| !s.is_empty());
    is_leaf = false;
  }
  let mut result = Vec::new();
  // No separate style group to close — style applied at group level
  result.extend(delim_tokens);
  result.push(T_END!());
  result.push(T_END!());
  result
}

/// Perl: lstClassProperty — recursive property lookup on class chain.
fn lst_class_property(classname: &str, property: &str) -> Option<String> {
  let prop_key = s!("LST_CLASSES@{classname}@{property}");
  if let Some(val) = state::lookup_value(&prop_key) {
    let s = val.to_string();
    if !s.is_empty() {
      return Some(s);
    }
  }
  let class_key = s!("LST_CLASSES@{classname}@class");
  if let Some(parent) = state::lookup_value(&class_key) {
    let parent_str = parent.to_string();
    if !parent_str.is_empty() {
      return lst_class_property(&parent_str, property);
    }
  }
  None
}

/// Main listing parser context
struct LstContext {
  listing:        String,
  linenum:        i64,
  colnum:         i64,
  mode:           String,
  linestart:      Option<usize>,
  emptyfrom:      Option<usize>,
  lsttokens:      Vec<Token>,
  // Regexes for current scope
  id_re:          Option<Regex>,
  delim_re:       Option<Regex>,
  escape_re:      Option<Regex>,
  quoted_re:      Regex,
  space_token:    Token,
  case_sensitive: bool,
  // Perl: lsthk@SelectCharTable — literate substitution patterns (TODO: implement processing)
  #[allow(dead_code)]
  literate:       Vec<(String, Tokens, bool)>,
  #[allow(dead_code)]
  literate_re:    Option<Regex>,
  firstline:      i64,
  lastline:       i64,
}

/// Perl: linetest closure — checks if a line number should be included based on firstline/lastline.
fn lst_linetest(ctx: &LstContext) -> bool {
  ctx.firstline <= ctx.linenum && ctx.linenum <= ctx.lastline
}

/// Perl: lstProcess — main entry point for processing listing text.
fn lst_process(mode: &str, text: &str) -> Tokens {
  if text.is_empty() || !lst_get_boolean("print") {
    return Tokens!();
  }

  let mut text = text.to_string();
  // Strip trailing whitespace if !showlines
  if !lst_get_boolean("showlines") {
    text = text.trim_end().to_string();
  }

  // Establish line numbering parameters
  let firstnumber = lst_get_literal("firstnumber");
  let line0: i64 = match firstnumber.as_str() {
    "last" => state::lookup_value("LISTINGS_LAST_NUMBER")
      .map(|v| v.to_string().parse().unwrap_or(1))
      .unwrap_or(1),
    "auto" => {
      let name = lst_get_literal("name");
      if !name.is_empty() {
        let key = s!("LISTINGS_LAST_NUMBER_{name}");
        state::lookup_value(&key)
          .map(|v| v.to_string().parse().unwrap_or(1))
          .unwrap_or(1)
      } else {
        1
      }
    },
    _ => firstnumber.parse().unwrap_or(1),
  };

  let stepnumber = lst_get_number("stepnumber");
  let _numpos = if stepnumber == 0 {
    "none".to_string()
  } else {
    lst_get_literal("numbers")
  };

  // Build ID regex dynamically from character table in state
  // Perl: join('', sort keys %{$$characters{letter}})
  let letter_chars = build_char_class("letter");
  let digit_chars = build_char_class("digit");
  // Perl: LookupValue('LST@TEXCS') — set when texcs or moretexcs has been used
  let has_texcs = matches!(state::lookup_value("LST@TEXCS"), Some(Stored::Bool(true)));
  let id_re = if letter_chars.is_empty() {
    None
  } else {
    let id_pattern = if has_texcs {
      format!("\\\\?[{letter_chars}][{letter_chars}{digit_chars}]*")
    } else {
      format!("[{letter_chars}][{letter_chars}{digit_chars}]*")
    };
    Regex::new(&id_pattern).ok()
  };

  // Build delimiter regexes from LST_DELIM_KEYS (Perl: $LaTeXML::DELIM_RE, $LaTeXML::ESCAPE_RE)
  let delim_keys: Vec<String> = match state::lookup_value("LST_DELIM_KEYS") {
    Some(Stored::Tokens(t)) => t.unlist_ref().iter().map(|tok| tok.to_string()).collect(),
    _ => vec![],
  };
  // Also check mathescape delimiter ($)
  let mut all_delim_keys = delim_keys;
  if let Some(Stored::String(_)) = state::lookup_value("LST_DELIM@$@open") {
    if !all_delim_keys.iter().any(|k| k == "$") {
      all_delim_keys.push("$".to_string());
    }
  }
  // Perl listings.sty.ltxml:1283
  //   local $LaTeXML::DELIM_RE = join('|', map { $$delimiters{$_}{open} } sort keys %$delimiters);
  // The lexical sort matters: leftmost-first regex alternation means short
  // prefixes that sort before longer ones win, e.g. `<` (XML tag) is matched
  // before `<!--` (XML comment) so a stray `<!--` is classified as a tag.
  all_delim_keys.sort();

  let mut delim_opens: Vec<String> = Vec::new();
  let mut escape_opens: Vec<String> = Vec::new();
  for key in &all_delim_keys {
    let open_key = s!("LST_DELIM@{key}@open");
    if let Some(Stored::String(open_sym)) = state::lookup_value(&open_key) {
      let open_re = arena::with(open_sym, |s| s.to_string());
      delim_opens.push(open_re.clone());
      let escape_key = s!("LST_DELIM@{key}@escape");
      if matches!(state::lookup_value(&escape_key), Some(Stored::Bool(true))) {
        escape_opens.push(open_re);
      }
    }
  }
  let delim_re = if delim_opens.is_empty() {
    None
  } else {
    Regex::new(&format!("^({})", delim_opens.join("|"))).ok()
  };
  let escape_re = if escape_opens.is_empty() {
    None
  } else {
    Regex::new(&format!("^({})", escape_opens.join("|"))).ok()
  };

  let space_token = if lst_get_boolean("showspaces") {
    T_CS!("\\@lst@visible@space")
  } else {
    T_CS!(" ")
  };

  let case_sensitive = lst_get_boolean("sensitive");
  // Perl: if (!$CASE_SENSITIVE) { foreach word (keys %$words) { $$words{uc($word)} =
  // $$words{$word}; } }
  if !case_sensitive {
    if let Some(Stored::Strings(word_list)) = state::lookup_value("LST_WORD_LIST") {
      for word_sym in word_list.iter() {
        let word = arena::to_string(*word_sym);
        let upper = word.to_uppercase();
        if upper != word {
          let src_class_key = s!("LST_WORDS@{word}@class");
          let dst_class_key = s!("LST_WORDS@{upper}@class");
          if let Some(class_val) = state::lookup_value(&src_class_key) {
            state::assign_value(&dst_class_key, class_val, None);
          }
          let src_index_key = s!("LST_WORDS@{word}@index");
          let dst_index_key = s!("LST_WORDS@{upper}@index");
          if let Some(index_val) = state::lookup_value(&src_index_key) {
            state::assign_value(&dst_index_key, index_val, None);
          }
        }
      }
    }
  }

  let mut ctx = LstContext {
    listing: text,
    linenum: line0,
    colnum: 0,
    mode: mode.to_string(),
    linestart: None,
    emptyfrom: None,
    lsttokens: vec![T_BEGIN!()],
    id_re,
    delim_re,
    escape_re,
    quoted_re: Regex::new(r"^\\\\").unwrap(),
    space_token,
    case_sensitive,
    literate: build_literate_entries(),
    literate_re: build_literate_re(false),
    firstline: lst_get_number("firstline"),
    lastline: lst_get_number("lastline"),
  };

  // Add preamble tokens
  if let Some(Stored::Tokens(preamble)) = state::lookup_value("LISTINGS_PREAMBLE") {
    ctx.lsttokens.extend(preamble.unlist());
  }
  let basicstyle = lst_get_tokens("basicstyle");
  if !basicstyle.is_empty() {
    ctx.lsttokens.extend(basicstyle.unlist());
  }

  // Perl: while ($listing && !&$linetest($linenum)) { skip lines before firstline }
  while !ctx.listing.is_empty() && !lst_linetest(&ctx) {
    if let Some(pos) = ctx.listing.find('\n') {
      ctx.listing = ctx.listing[pos + 1..].to_string();
    } else {
      ctx.listing = String::new();
    }
    ctx.linenum += 1;
  }

  if mode != "inline" {
    ctx.lsttokens.extend(invoke(T_CS!("\\setcounter"), vec![
      Tokens!(T_OTHER!("lstnumber")),
      Tokens::new(ExplodeText!(&ctx.linenum.to_string())),
    ]));
    lst_process_start_line(&mut ctx);
  }

  lst_process_internal(&mut ctx, None, None);

  if mode != "inline" {
    lst_process_end_line(&mut ctx);
  }

  // Save line number for later use
  let name = lst_get_literal("name");
  state::assign_value(
    "LISTINGS_LAST_NUMBER",
    Stored::Int(ctx.linenum),
    Some(Scope::Global),
  );
  if !name.is_empty() {
    let key = s!("LISTINGS_LAST_NUMBER_{name}");
    state::assign_value(&key, Stored::Int(ctx.linenum), Some(Scope::Global));
  }

  // Remove trailing empty lines
  if let Some(from) = ctx.emptyfrom {
    ctx.lsttokens.truncate(from);
  }

  ctx.lsttokens.push(T_END!());
  Tokens::new(ctx.lsttokens)
}

fn lst_process_start_line(ctx: &mut LstContext) {
  let stepnumber = lst_get_number("stepnumber");
  let numpos = if stepnumber == 0 {
    "none".to_string()
  } else {
    lst_get_literal("numbers")
  };
  ctx.linestart = Some(ctx.lsttokens.len());
  ctx.lsttokens.push(T_CS!("\\@lst@startline"));
  ctx.lsttokens.push(T_BEGIN!());
  if numpos != "none" {
    let is_empty = ctx.listing.starts_with('\n') || ctx.listing.is_empty();
    let number_tokens = lst_do_number(ctx, is_empty);
    ctx.lsttokens.extend(number_tokens.unlist());
  }
  ctx.lsttokens.push(T_END!());
}

fn lst_process_end_line(ctx: &mut LstContext) {
  if ctx.colnum == 0 {
    if ctx.emptyfrom.is_none() {
      ctx.emptyfrom = ctx.linestart;
    }
  } else {
    ctx.emptyfrom = None;
  }
  ctx.lsttokens.push(T_CS!("\\@lst@endline"));
}

/// Perl: lstDoNumber — generate line number tokens.
fn lst_do_number(ctx: &LstContext, is_empty: bool) -> Tokens {
  let stepnumber = lst_get_number("stepnumber");
  let needs_number = state::lookup_value("LISTINGS_NEEDS_NUMBER")
    .map(|v| v.eq_text("true") || v.eq_text("1"))
    .unwrap_or(false);

  let number_blank = lst_get_boolean("numberblanklines");
  if (needs_number || ((ctx.linenum - 1) % stepnumber.max(1)) == 0) && (number_blank || !is_empty) {
    state::assign_value("LISTINGS_NEEDS_NUMBER", Stored::Bool(false), None);
    Tokens::new(invoke(T_CS!("\\lx@make@tags"), vec![Tokens!(T_OTHER!(
      "lstnumber"
    ))]))
  } else {
    Tokens::new(invoke(T_CS!("\\@lst@linenumber"), vec![Tokens!()]))
  }
}

/// Perl: lstProcess_internal — the recursive descent parser.
/// Order matches Perl: end_re, literate, delimiters, identifiers, newline, formfeed, whitespace,
/// quoted, default
fn lst_process_internal(
  ctx: &mut LstContext,
  end_re: Option<&Regex>,
  outer_class: Option<&str>,
) {
  // Perl listings.sty.ltxml:1411
  //   my $classname = ($outerclass ? undef : $$words{$lookup}{class} || 'identifiers');
  // When recursing inside a delimited class (string, tag, comment, …) inner
  // identifiers/keywords are emitted as plain tokens — only nested delimiters
  // (e.g. strings inside tags) re-wrap.
  let _ = outer_class;
  let mut prev_listing = String::new();
  // Precompile static regexes
  let newline_re = Regex::new(r"^\s*?\n").unwrap();
  let space_re = Regex::new(r"^[\t ]+").unwrap();

  while !ctx.listing.is_empty() {
    // Loop guard — must make progress on every step
    if ctx.listing == prev_listing {
      log::warn!(
        "lstProcess_internal failed to make progress. Content: '{}'",
        &ctx.listing[..ctx.listing.len().min(60)]
      );
      ctx.listing.clear();
      break;
    }
    prev_listing = ctx.listing.clone();

    // 1. Check end regex (close delimiter)
    // But first: if the end_re would match, check if quoted_re also matches at the same position.
    // If quoted_re matches, the end_re match is spurious (e.g. '' in doubled delimiters).
    // This replaces Perl's lookahead (?!...) in close_re which Rust regex doesn't support.
    if let Some(re) = end_re {
      let re_str = re.as_str();
      if re_str.contains("__NONWORD__") {
        let close = ctx.listing.is_empty()
          || ctx
            .listing
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric() && c != '_');
        if close {
          break;
        }
      } else if re_str.contains("__NEWLINE__") {
        let close = ctx.listing.is_empty() || ctx.listing.starts_with('\n');
        if close {
          break;
        }
      } else if let Some(m) = re.find(&ctx.listing) {
        if m.start() == 0 {
          // Before closing, check if quoted_re matches (takes priority)
          let is_quoted = ctx.quoted_re.find(&ctx.listing).is_some();
          if !is_quoted {
            ctx.colnum += m.len() as i64;
            ctx.listing = ctx.listing[m.end()..].to_string();
            break;
          }
        }
      }
    }

    // 2. Literate expressions
    // Perl: use LITERATE_INNER_RE inside delimited contexts, LITERATE_RE at top level
    {
      let lit_re = if end_re.is_some() {
        // Inside a delimited context: only non-protected patterns
        build_literate_re(true)
      } else {
        ctx.literate_re.clone()
      };
      if let Some(ref re) = lit_re {
        if let Some(m) = re.find(&ctx.listing) {
          let matched = m.as_str().to_string();
          ctx.listing = ctx.listing[m.end()..].to_string();
          ctx.colnum += matched.len() as i64;
          // Find the replacement tokens for this pattern
          let repl_key = s!("LST_LIT@{matched}");
          if let Some(Stored::Tokens(replacement)) = state::lookup_value(&repl_key) {
            ctx.lsttokens.push(T_CS!("\\@listingLiterate"));
            ctx.lsttokens.push(T_BEGIN!());
            ctx.lsttokens.extend(replacement.unlist());
            ctx.lsttokens.push(T_END!());
          }
          continue;
        }
      }
    }

    // 3. Delimiters — strings, comments, escapes, mathescape
    if let Some(ref delim_re) = ctx.delim_re.clone() {
      if let Some(m) = delim_re.find(&ctx.listing) {
        let open = m.as_str().to_string();
        ctx.listing = ctx.listing[m.end()..].to_string();
        ctx.colnum += open.len() as i64;

        // Look up delimiter info. The `invisible` flag is already applied
        // at delimiter-registration time (lst_add_delimiter sets empty
        // open_tex / close_tex when invisible, so lst_class_begin /
        // lst_class_end never emit the delim chars). No runtime check
        // needed here — Perl lstProcess_internal also has no separate
        // gate at this point.
        let class_key = s!("LST_DELIM@{open}@class");
        let close_key = s!("LST_DELIM@{open}@close");
        let classname = state::lookup_value(&class_key)
          .map(|v| v.to_string())
          .unwrap_or_default();
        let close_re_str = state::lookup_value(&close_key)
          .map(|v| v.to_string())
          .unwrap_or_default();

        // Perl: lstProcessPush(lstClassBegin($classname))
        // Note: delimiter chars come from begin/end tokens in lstClassBegin/lstClassEnd
        ctx.lsttokens.extend(lst_class_begin(&classname));

        // Check if this is an 'eval' class (mathescape, texcl, escapechar)
        let is_eval =
          lst_class_property(&classname, "eval").is_some_and(|v| v == "true" || v == "1");

        if is_eval {
          // For eval classes: match until close, then tokenize the content as TeX
          // Perl: TokenizeBalanced($string) — close delimiter is NOT separately emitted,
          // because lstClassEnd already provides the closing tokens (e.g. $ for mathescape)
          if close_re_str == "__NEWLINE__" {
            // Sentinel: read until newline (for texcl line comments)
            let content = if let Some(pos) = ctx.listing.find('\n') {
              let c = ctx.listing[..pos].to_string();
              ctx.listing = ctx.listing[pos..].to_string();
              c
            } else {
              let c = ctx.listing.clone();
              ctx.listing.clear();
              c
            };
            let content_tokens = tokenize_balanced(&content);
            ctx.lsttokens.extend(content_tokens);
          } else if let Ok(close_re) = Regex::new(&close_re_str) {
            if let Some(cm) = close_re.find(&ctx.listing) {
              let content = ctx.listing[..cm.start()].to_string();
              ctx.listing = ctx.listing[cm.end()..].to_string();
              // Tokenize the content as real TeX (not raw listing)
              let content_tokens = tokenize_balanced(&content);
              ctx.lsttokens.extend(content_tokens);
              // Note: close delimiter is NOT pushed — lstClassEnd handles it
            }
          }
        } else {
          // For non-eval classes (strings, comments): recurse with limited delimiters
          let recursive_key = s!("LST_DELIM@{open}@recursive");
          let is_recursive = matches!(
            state::lookup_value(&recursive_key),
            Some(Stored::Bool(true))
          );
          if !close_re_str.is_empty() {
            // Sentinel close patterns (for zero-width assertions) use a dummy regex
            // that contains the sentinel as a flag — the actual check happens in step 1.
            let close_re_pattern = if close_re_str.starts_with("__") {
              // Create a regex that will never match normal text but carries the sentinel
              format!("^(__{}__SENTINEL)", close_re_str.trim_matches('_'))
            } else {
              format!("^({close_re_str})")
            };
            if let Ok(close_re) = Regex::new(&close_re_pattern) {
              // Recurse with appropriate delimiter set
              let saved_delim = ctx.delim_re.clone();
              let saved_id = ctx.id_re.clone();
              let saved_quoted = ctx.quoted_re.clone();
              let saved_space = ctx.space_token;
              // Perl: local $SPACE = visible when inside string class + showstringspaces
              if classname.starts_with("string") && lst_get_boolean("showstringspaces") {
                ctx.space_token = T_CS!("\\@lst@visible@space");
              }
              if !is_recursive {
                // Perl listings.sty.ltxml:1396-1398
                //   local $DELIM_RE = ($$delim{recursive}
                //     ? $DELIM_RE
                //     : join('|', grep { $_ } $ESCAPE_RE,
                //                            $$delim{nested} && $$delim{open}));
                // For type='n' nested delims we still allow the SAME opener to
                // match recursively (so `{foo {bar} baz}`-style nesting works).
                let nested_key = s!("LST_DELIM@{open}@nested");
                let is_nested = matches!(
                  state::lookup_value(&nested_key),
                  Some(Stored::Bool(true))
                );
                let mut alternatives: Vec<String> = Vec::new();
                if let Some(ref er) = ctx.escape_re {
                  let s = er.as_str().trim_start_matches("^(").trim_end_matches(')');
                  if !s.is_empty() {
                    alternatives.push(s.to_string());
                  }
                }
                if is_nested {
                  let open_key = s!("LST_DELIM@{open}@open");
                  if let Some(Stored::String(open_re_sym)) = state::lookup_value(&open_key) {
                    let open_re = arena::with(open_re_sym, |s| s.to_string());
                    if !open_re.is_empty() {
                      alternatives.push(open_re);
                    }
                  }
                }
                ctx.delim_re = if alternatives.is_empty() {
                  None
                } else {
                  Regex::new(&format!("^({})", alternatives.join("|"))).ok()
                };
                ctx.id_re = None;
              }
              // Perl: local $QUOTED_RE = join('|', grep { $_ } $QUOTED_RE, $$delim{quoted});
              let quoted_key = s!("LST_DELIM@{open}@quoted");
              if let Some(delim_quoted) = state::lookup_value(&quoted_key) {
                let dq = delim_quoted.to_string();
                if !dq.is_empty() {
                  let new_quoted = format!(
                    "^({}|{})",
                    ctx
                      .quoted_re
                      .as_str()
                      .trim_start_matches("^(")
                      .trim_end_matches(')'),
                    dq
                  );
                  if let Ok(re) = Regex::new(&new_quoted) {
                    ctx.quoted_re = re;
                  }
                }
              }
              lst_process_internal(ctx, Some(&close_re), Some(&classname));
              ctx.delim_re = saved_delim;
              ctx.id_re = saved_id;
              ctx.quoted_re = saved_quoted;
              ctx.space_token = saved_space;
            }
          }
        }
        // Perl: lstProcessPush(invisible ? () : split(//, $close), lstClassEnd($classname))
        // For non-eval: close was consumed by end_re in recursive call; delimiter chars already
        // handled
        ctx.lsttokens.extend(lst_class_end(&classname));
        continue;
      }
    }

    // 4. Identifier (word) matching
    if let Some(ref id_re) = ctx.id_re {
      if let Some(m) = id_re.find(&ctx.listing) {
        if m.start() == 0 {
          // eprintln!("DEBUG id_re matched: '{}' in '{}'", m.as_str(),
          // &ctx.listing[..ctx.listing.len().min(40)]);
          let word = m.as_str().to_string();
          ctx.listing = ctx.listing[m.end()..].to_string();
          ctx.colnum += word.len() as i64;

          let lookup = if ctx.case_sensitive {
            word.clone()
          } else {
            word.to_uppercase()
          };

          // Look up word class
          // Perl listings.sty.ltxml:1411
          //   my $classname = ($outerclass ? undef : $$words{$lookup}{class} || 'identifiers');
          // Inside a recursive delim wrapper (e.g. the `tags<>` span), inner
          // identifiers and keywords are emitted without their own class wrap.
          let word_class_key = s!("LST_WORDS@{lookup}@class");
          let raw_class = state::lookup_value(&word_class_key);
          let classname: Option<String> = if outer_class.is_some() {
            None
          } else {
            Some(
              raw_class
                .map(|v| v.to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "identifiers".to_string()),
            )
          };

          // Rescan word characters
          let word_tokens: Vec<Token> = word
            .chars()
            .flat_map(|c| {
              let s = c.to_string();
              if let Some(rescanned) = lst_rescan(Some(Tokens::new(vec![T_OTHER!(&s)]))) {
                rescanned.unlist()
              } else {
                vec![T_OTHER!(&s)]
              }
            })
            .collect();

          // Check for index
          let index_key = s!("LST_WORDS@{lookup}@index");
          if let Some(_index_name) = state::lookup_value(&index_key) {
            // Index generation (simplified)
          }

          // Perl: if excludeslash, move the "\" outside of the styling
          let mut pre_tokens: Vec<Token> = Vec::new();
          let mut styled_tokens = word_tokens;
          if let Some(ref cname) = classname {
            let excludeslash_key = s!("LST_CLASSES@{cname}@excludeslash");
            let has_excludeslash = match state::lookup_value(&excludeslash_key) {
              Some(Stored::Bool(true)) => true,
              Some(Stored::String(s)) => arena::with(s, |v| v == "true"),
              _ => false,
            };
            if has_excludeslash && !styled_tokens.is_empty() {
              pre_tokens.push(styled_tokens.remove(0));
            }
          }

          ctx.lsttokens.extend(pre_tokens);
          if let Some(cname) = classname {
            ctx.lsttokens.extend(lst_class_begin(&cname));
            ctx.lsttokens.extend(styled_tokens);
            ctx.lsttokens.extend(lst_class_end(&cname));
          } else {
            // Perl: outerclass branch — emit identifier tokens without their own
            // class wrap; they live inside the enclosing delim's wrapper.
            ctx.lsttokens.extend(styled_tokens);
          }
          continue;
        }
      }
    }

    // 5. Newline handling
    if let Some(m) = newline_re.find(&ctx.listing) {
      ctx.listing = ctx.listing[m.end()..].to_string();
      if ctx.mode != "inline" {
        lst_process_end_line(ctx);
        if let Ok(inv) = (|| -> Result<Tokens> {
          Ok(Invocation!(T_CS!("\\stepcounter"), vec![T_OTHER!(
            "lstnumber"
          )]))
        })() {
          ctx.lsttokens.extend(inv.unlist());
        }
        ctx.linenum += 1;
        ctx.colnum = 0;
        // Perl: while ($listing ne '' && !&$linetest($linenum)) { skip lines }
        while !ctx.listing.is_empty() && !lst_linetest(ctx) {
          // Skip this line
          if let Some(pos) = ctx.listing.find('\n') {
            ctx.listing = ctx.listing[pos + 1..].to_string();
          } else {
            ctx.listing = String::new();
          }
          if let Ok(inv) = (|| -> Result<Tokens> {
            Ok(Invocation!(T_CS!("\\stepcounter"), vec![T_OTHER!(
              "lstnumber"
            )]))
          })() {
            ctx.lsttokens.extend(inv.unlist());
          }
          ctx.linenum += 1;
        }
        // Handle gobble
        let gobble = lst_get_number("gobble");
        for _ in 0..gobble {
          if !ctx.listing.is_empty() {
            ctx.listing = ctx.listing[1..].to_string();
          }
        }
        lst_process_start_line(ctx);
      }
      continue;
    }

    // 6. Formfeed
    if ctx.listing.starts_with('\x0C') {
      ctx.listing = ctx.listing[1..].to_string();
      let ff = lst_get_tokens("formfeed");
      ctx.lsttokens.extend(ff.unlist());
      ctx.colnum += 1;
      continue;
    }

    // 7. Whitespace / tab expansion
    if let Some(m) = space_re.find(&ctx.listing) {
      let s = &ctx.listing[..m.end()];
      let tabsize = lst_get_number("tabsize").max(1);
      let mut n: i64 = 0;
      for c in s.chars() {
        n += if c == ' ' {
          1
        } else {
          tabsize - ((ctx.colnum + n) % tabsize)
        };
      }
      ctx.listing = ctx.listing[m.end()..].to_string();
      ctx.lsttokens.extend(lst_class_begin("spaces"));
      for _ in 0..n {
        ctx.lsttokens.push(ctx.space_token);
      }
      ctx.lsttokens.extend(lst_class_end("spaces"));
      ctx.colnum += n;
      continue;
    }

    // 8. Quoted expressions (e.g. \\)
    {
      let quoted_re = ctx.quoted_re.clone();
      if let Some(m) = quoted_re.find(&ctx.listing) {
        let matched = m.as_str().to_string();
        ctx.listing = ctx.listing[m.end()..].to_string();
        // Perl: lstProcessPush(split(//,$quoted)) — each char goes through rescan
        for c in matched.chars() {
          let ch_str = c.to_string();
          if let Some(rescanned) = lst_rescan(Some(Tokens::new(vec![T_OTHER!(&ch_str)]))) {
            ctx.lsttokens.extend(rescanned.unlist());
          }
        }
        ctx.colnum += matched.len() as i64;
        continue;
      }
    }

    // 9. Default: pass through single character
    if let Some(ch) = ctx.listing.chars().next() {
      ctx.listing = ctx.listing[ch.len_utf8()..].to_string();
      let ch_str = ch.to_string();
      if let Some(rescanned) = lst_rescan(Some(Tokens::new(vec![T_OTHER!(&ch_str)]))) {
        ctx.lsttokens.extend(rescanned.unlist());
      }
      ctx.colnum += 1;
    }
  }
}

/// Perl: lstProcessInline — wraps inline listing in \@listings@inline.
fn lst_process_inline(text: &str) -> Vec<Token> {
  let body = lst_process("inline", text);
  invoke(T_CS!("\\@listings@inline"), vec![body])
}

/// Perl: lstProcessBlock — wraps block listing, stores data.
fn lst_process_block(name: Option<Tokens>, text: &str) -> (Vec<Token>, Vec<Token>) {
  // Store listing data for base64 encoding
  let c_val = state::lookup_value("LISTINGS_DATA_COUNTER")
    .map(|v| v.to_string().parse::<i64>().unwrap_or(0))
    .unwrap_or(0)
    + 1;
  state::assign_value(
    "LISTINGS_DATA_COUNTER",
    Stored::Int(c_val),
    Some(Scope::Global),
  );
  let data_key = s!("LISTINGS_DATA_{c_val}");
  state::assign_value(
    &data_key,
    Stored::String(arena::pin(text)),
    Some(Scope::Global),
  );

  let processed = lst_process("block", text);

  let mut body_tokens = Vec::new();
  // Add preamble_before
  if let Some(Stored::Tokens(pre)) = state::lookup_value("LISTINGS_PREAMBLE_BEFORE") {
    body_tokens.extend(pre.unlist());
  }
  // Invocation of \@@listings@block{counter}{processed}{name}
  let name_tokens = name.unwrap_or(Tokens!());
  body_tokens.extend(invoke(T_CS!("\\@@listings@block"), vec![
    Tokens::new(ExplodeText!(&c_val.to_string())),
    processed,
    name_tokens,
  ]));

  let mut trailer = Vec::new();
  if let Some(Stored::Tokens(post)) = state::lookup_value("LISTINGS_POSTAMBLE") {
    trailer.extend(post.unlist());
  }
  trailer.push(T_END!()); // balance bgroup from the caller

  (body_tokens, trailer)
}

/// Perl: lstProcessDisplay — generate full display listing with optional caption/title.
pub fn lst_process_display(name: Option<Tokens>, text: &str) -> Vec<Token> {
  let (mut body, trailer) = lst_process_block(name.clone(), text);

  // Perl: AssignValue('LST@toctitle', $name) — so it shows up in list of listings
  if let Some(ref n) = name {
    if !n.is_empty() {
      state::assign_value("LST@toctitle", Stored::from(n.clone()), None);
    }
  }

  // Check for caption/title/toctitle — Perl lines 183-193
  let caption_tokens = lst_get_tokens("caption");
  let title_tokens = lst_get_tokens("title");
  let toctitle_tokens = lst_get_tokens("toctitle");
  let label_tokens = lst_get_tokens("label");

  let mut numbered = false;
  let mut has_caption = false;

  if !caption_tokens.is_empty() {
    numbered = true;
    has_caption = true;
    // Perl lines 184-188: Extract optional [short caption] from caption text
    let mut toks: Vec<Token> = caption_tokens.unlist();
    let mut short_caption = Tokens!();
    if toks.first().map(|t| t.text == pin!("[")).unwrap_or(false) {
      while !toks.is_empty() && toks[0].text != pin!("]") {
        short_caption.unlist_mut().push(toks.remove(0));
      }
      if !toks.is_empty() {
        toks.remove(0);
      } // consume ']'
    }
    let caption = invoke(T_CS!("\\lstlisting@makecaption"), vec![
      short_caption,
      Tokens::new(toks),
    ]);
    let captionpos = lst_get_literal("captionpos");
    if captionpos == "t" {
      let mut new_body = caption;
      new_body.extend(body);
      body = new_body;
    } else {
      body.extend(caption);
    }
  } else if !title_tokens.is_empty() {
    has_caption = true;
    let title_inv = invoke(T_CS!("\\lstlisting@maketitle"), vec![title_tokens]);
    body.extend(title_inv);
  } else if !toctitle_tokens.is_empty() {
    // Perl line 192-193: toctitle without caption/title
    has_caption = true;
    let toctitle_inv = invoke(T_CS!("\\lstlisting@maketoctitle"), vec![toctitle_tokens]);
    body.extend(toctitle_inv);
  }

  if !label_tokens.is_empty() {
    let label_inv = invoke(T_CS!("\\label"), vec![label_tokens]);
    let mut new_body = label_inv;
    new_body.extend(body);
    body = new_body;
  }

  body.extend(trailer);

  let mut result = Vec::new();
  if numbered || has_caption {
    result.push(T_CS!("\\par"));
  }
  result.push(T_BEGIN!());

  // \def\lstname{...}
  if let Some(ref n) = name {
    if !n.is_empty() {
      result.push(T_CS!("\\def"));
      result.push(T_CS!("\\lstname"));
      result.push(T_BEGIN!());
      result.extend_from_slice(n.unlist_ref());
      result.push(T_END!());
    }
  }

  let name_nonempty = name.as_ref().is_some_and(|n| !n.is_empty());

  if numbered || name_nonempty {
    result.extend(invoke(T_CS!("\\@listings"), vec![Tokens::new(body)]));
  } else if has_caption {
    result.extend(invoke(T_CS!("\\@@listings"), vec![Tokens::new(body)]));
  } else {
    result.extend(body);
  }

  result.push(T_END!());
  result
}

/// Perl: lstExtractColor — extract color from a TeX command by digesting it.
fn lst_extract_color(cmd: &Tokens) -> Option<String> {
  if cmd.is_empty() {
    return None;
  }
  bgroup();
  let _ = stomach::digest(cmd.clone());
  // Use to_stored() format ("rgb r g b") for round-trip through state storage,
  // since \lst@@@set@background uses Color::from_stored() to reconstruct the color.
  let color = lookup_font().and_then(|f| f.color.as_ref().map(|c| c.to_stored()));
  egroup().ok();
  color
}

//======================================================================
// Region 10: Main LoadDefinitions block
//======================================================================

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // Region 1: Preamble (Perl lines 1-33)
  //======================================================================
  RequireResource!("ltx-listings.css");
  RequirePackage!("textcomp");

  // Stubs for listings.sty internal CSes that language-pack .sty files
  // (lstlang0.sty .. lstlang3.sty) reference when they raw-load. Our
  // hand-port implements the public API but skips the listings.sty
  // body that defines these, so the lang packs' raw-load fires
  // "undefined" cascades.
  //
  // \lst@Key{name}{default}{body} — register a Listings option key. We
  // don't materialize key registration (lst_activate has its own keyval
  // parser); stub as no-op.
  //
  // \lst@NormedDef takes `\cs` (no braces) + `{val}` and `\def`s \cs
  // to a normalized form of val. Implementing this faithfully needs a
  // DefToken parameter type — we use a closure to consume both args
  // and emit a `\def`. Witness: lstlang3.sty raw-load (3 papers in
  // Stage-13 v3) → cascade unblocks past these CSes.
  DefMacro!("\\lst@Key{}{}{}", "");
  DefMacro!("\\lst@NormedDef DefToken {}", sub[(cs, val)] {
    let mut out = vec![T_CS!("\\def"), cs];
    out.push(T_BEGIN!());
    out.extend(val.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });

  // \lstKV@SetIf{val}\ifFlag — listings' boolean key setter (TL
  // listings.sty L390): `\let \ifFlag \iftrue` if val starts with `t`,
  // else `\let \ifFlag \iffalse`. Used by `\lst@Key{name}{default}[t]
  // {\lstKV@SetIf{#1}\ifFlag}`. Note: this does NOT require a prior
  // `\newif\ifFlag` — it `\let`s the bare CS directly.
  // Witness: 2405.18399 (matlab-prettifier).
  DefMacro!("\\lstKV@SetIf{} DefToken", sub[(val, cs)] {
    let val_s = val.to_string();
    let target = if val_s.trim_start().chars().next().map(|c| c.to_ascii_lowercase()) == Some('t') {
      T_CS!("\\iftrue")
    } else {
      T_CS!("\\iffalse")
    };
    Ok(Tokens!(T_CS!("\\let"), cs, target))
  });

  // \lst@InstallFamily{...} / @{...} — from lstmisc.sty (L543, L558),
  // registers a "family" of style keys. We don't materialize this
  // machinery; stub both forms as no-ops so packages extending
  // listings (e.g. matlab-prettifier) don't crash on undefined CS.
  DefMacro!("\\lst@InstallFamily{}{}{}{}{}", "");
  DefMacro!("\\lst@InstallFamily@{}{}{}{}{}{}{}{}", "");

  // Initialize state values
  state::assign_value("LISTINGS_PREAMBLE", Stored::Tokens(Tokens!()), None);
  state::assign_value("LISTINGS_PREAMBLE_BEFORE", Stored::Tokens(Tokens!()), None);
  state::assign_value("LISTINGS_POSTAMBLE", Stored::Tokens(Tokens!()), None);
  state::assign_value("LISTINGS_DATA_COUNTER", Stored::Int(0), None);

  //======================================================================
  // Region 2: Top-level commands (Perl lines 35-153)
  //======================================================================

  // \lstset — set various Listings keys
  DefPrimitive!("\\lstset RequiredKeyVals:LST", sub[(kv)] {
    lst_activate(Some(&kv));
  });

  // \lstinline — inline listing
  DefMacro!("\\lstinline", "\\leavevmode\\lx@lstinline");
  DefMacro!("\\lx@lstinline OptionalKeyVals:LST", sub[(kv)] {
    bgroup();
    lst_activate(kv.as_ref());
    // Read opening delimiter under NORMAL catcodes — so `{`/`}` keep
    // BEGIN/END catcodes and the standard "closing brace" pairing works
    // for `\lstinline{a_word}`-style invocations. Perl reads `$init`
    // *before* the cattable swap (`listings.sty.ltxml:61` then `:289`).
    let init = gullet::read_token()?;
    let until = init.as_ref().map(|t| {
      if t.get_catcode() == Catcode::BEGIN { T_END!() } else { *t }
    });
    // Switch to verbatim catcodes BEFORE reading the body, so e.g.
    // `\lstinline![ %rcx { 0 } ] = ... ->!` reads its `%` as OTHER
    // rather than as a comment-trigger that drops the rest of the
    // line and greedily consumes subsequent input hunting for the
    // closing `!`. Mirrors Perl `listings.sty.ltxml:289` `local $STATE
    // = $EMPTY_CATTABLE;`. Driver: 2301.10618 Tracking-mode `\item
    // \lstinline![ %rcx … ]!` cascade.
    //
    // Apply catcode tweaks via a fresh `push_frame` (so they're popped
    // on `pop_frame` below) — `begin_semiverbatim` would also reassign
    // MODE/IN_MATH which is unwanted here (the `\lstinline` body lives
    // inside the surrounding paragraph mode and the post-frame `\(`
    // math switches must keep working).
    state::push_frame();
    for c in ['%', '\\', '{', '}', '$', '&', '#', '^', '_', '~'] {
      state::assign_catcode(c, Catcode::OTHER, Some(Scope::Local));
    }
    let body = listings_read_raw_string(until.as_ref());
    state::pop_frame()?;
    let mut result = Vec::new();
    if let Some(Stored::Tokens(pre)) = state::lookup_value("LISTINGS_PREAMBLE_BEFORE") {
      result.extend(pre.unlist());
    }
    result.extend(lst_process_inline(&body));
    if let Some(Stored::Tokens(post)) = state::lookup_value("LISTINGS_POSTAMBLE") {
      result.extend(post.unlist());
    }
    result.push(T_END!()); // balance bgroup
    Ok(Tokens::new(result))
  });

  // \lstMakeShortInline
  // Perl: saves [LookupCatcode($ch), $STATE->lookupMeaning($token)] in LST_SHORT_INLINE mapping
  // then sets catcode to ACTIVE and defines the active token as a \lstinline shorthand.
  DefPrimitive!("\\lstMakeShortInline [] DefToken", sub[(kv, token)] {
    let ch = token.to_string();
    if ch.is_empty() { return Ok(Vec::new()); }
    let ch_first = ch.chars().next().unwrap();
    // Save original catcode so \lstDeleteShortInline can restore it exactly
    let orig_cc = state::lookup_catcode(ch_first).unwrap_or(Catcode::OTHER);
    assign_mapping(
      "LST_SHORT_INLINE",
      &ch,
      Some(Stored::Catcode(orig_cc)),
    );
    state::assign_catcode(ch_first, Catcode::ACTIVE, None);
    let active_tok = T_ACTIVE!(ch_first);
    let mut expansion = vec![T_CS!("\\lstinline")];
    if let Some(kv_tok) = kv.as_ref().filter(|k| !k.is_empty()) {
      expansion.push(T_OTHER!("["));
      expansion.extend(kv_tok.unlist_ref().iter().cloned());
      expansion.push(T_OTHER!("]"));
    }
    expansion.push(active_tok);
    def_macro(active_tok, None, Tokens::new(expansion), None)?;
  });

  // \lstDeleteShortInline
  // Perl: restores the saved catcode and meaning from LST_SHORT_INLINE mapping.
  DefPrimitive!("\\lstDeleteShortInline DefToken", sub[(token)] {
    let ch = token.to_string();
    if !ch.is_empty() {
      let ch_first = ch.chars().next().unwrap();
      // Restore the original catcode saved by \lstMakeShortInline
      let saved_cc = match lookup_mapping("LST_SHORT_INLINE", &ch) {
        Some(Stored::Catcode(cc)) => cc,
        _ => Catcode::OTHER,
      };
      // Clear the active meaning before restoring catcode, so the active
      // definition of this character doesn't linger as a stale macro.
      let active_tok = T_ACTIVE!(ch_first);
      assign_meaning(&active_tok, Stored::None, None);
      state::assign_catcode(ch_first, saved_cc, None);
    }
  });

  // \begin{lstinline} — environment form
  {
    let cs = T_CS!("\\begin{lstinline}");
    let params = parse_parameters("OptionalKeyVals:LST", &cs, true)?;
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let kv = args.into_iter().next().unwrap_or_default();
        bgroup();
        state::assign_value("current_environment", Stored::String(arena::pin("lstlisting")), None);
        def_macro(T_CS!("\\@currenvir"), None, Tokens!(T_OTHER!("lstlisting")), None)?;
        let text = listings_read_raw_lines("lstinline");
        let _ = &kv; // lstActivate placeholder
        let mut result = Vec::new();
        if let Some(Stored::Tokens(pre)) = state::lookup_value("LISTINGS_PREAMBLE_BEFORE") {
          result.extend(pre.unlist());
        }
        result.extend(lst_process_inline(&text));
        if let Some(Stored::Tokens(post)) = state::lookup_value("LISTINGS_POSTAMBLE") {
          result.extend(post.unlist());
        }
        result.push(T_END!()); // balance bgroup
        Ok(Tokens::new(result))
      }
    )));
    def_macro(cs, params, expansion, None)?;
  }

  // \begin{lstlisting} — block listing environment
  {
    let cs = T_CS!("\\begin{lstlisting}");
    let params = parse_parameters("OptionalKeyVals:LST", &cs, true)?;
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let kv: Option<KeyVals> = args.into_iter().next().unwrap_or_default().into();
        bgroup();
        state::assign_value("current_environment", Stored::String(arena::pin("lstlisting")), None);
        def_macro(T_CS!("\\@currenvir"), None, Tokens!(T_OTHER!("lstlisting")), None)?;
        // Activate key-value options (language, style, etc.)
        lst_activate(kv.as_ref());
        let text = listings_read_raw_lines("lstlisting");
        let name = lst_get_tokens("name");
        let name_opt = if name.is_empty() { None } else { Some(name) };
        let result = lst_process_display(name_opt, &text);
        Ok(Tokens::new(result))
      }
    )));
    def_macro(cs, params, expansion, None)?;
  }

  // \lstinputlisting — read listing from file
  DefMacro!("\\lstinputlisting OptionalKeyVals:LST Semiverbatim", sub[(kv, file)] {
    let filename = file.to_string();
    let text = listings_read_raw_file(&filename).unwrap_or_default();
    bgroup();
    lst_activate(kv.as_ref());
    let mut name = lst_get_tokens("name");
    if name.is_empty() {
      name = Tokens::new(ExplodeText!(&filename));
    }
    let result = lst_process_display(Some(name), &text);
    Ok(Tokens::new(result))
  });

  // Counters
  NewCounter!("lstlisting", "document", idprefix => "LST");
  DefMacro!("\\ext@lstlisting", "lol");
  NewCounter!("lstnumber");
  DefMacro!("\\thelstnumber", "\\arabic{lstnumber}");
  DefMacro!("\\thelstlisting", "\\arabic{lstlisting}");

  // \lstnewenvironment — define new listing environments
  // Perl: DefPrimitive('\lstnewenvironment {}[Number][] DefPlain DefPlain', sub { ... })
  // Creates \begin{name} macro that digests start code (with arg substitution),
  // then reads raw lines and processes the listing display.
  DefPrimitive!("\\lstnewenvironment {}[Number][] DefPlain DefPlain", sub[(name, n_arg, opt_arg, start_code, end_code)] {
    let env_name = name.to_string();
    let n: usize = n_arg.value_of() as usize;
    // Build parameter spec matching Perl's convertLaTeXArgs($n, $opt).
    // `[N][default]` syntax: N total args; if [default] present, the FIRST
    // arg is OPTIONAL with that default value. The default IS allowed to
    // be empty (`\lstnewenvironment{mycode}[1][]{...}{...}` is the
    // typical "1-arg optional with empty default" form).
    // Was: `is_some_and(|t| !t.is_empty())` — empty-default form was
    // mis-classified as "no optional", producing a `{}` required-arg
    // env that errored on `\begin{mycode}[caption=...]`. Driver: 2301.10618
    // acmart paper using `\lstnewenvironment{mycode}[1][]{...}` then
    // `\begin{mycode}[caption=...,label=...]`.
    let has_opt = opt_arg.is_some();
    let mut param_spec = String::new();
    if has_opt {
      let opt_text = opt_arg.as_ref().unwrap().to_string();
      param_spec.push_str(&format!("[Default:{}]", opt_text));
      // optional counts as one arg
      for _ in 1..n { param_spec.push_str("{}"); }
    } else {
      for _ in 0..n { param_spec.push_str("{}"); }
    }
    let cs = T_CS!(s!("\\begin{{{env_name}}}"));
    let params = if param_spec.is_empty() {
      None
    } else {
      parse_parameters(&param_spec, &cs, true)?
    };
    // Move start_code / end_code / env_name directly into the closure
    // capture — none are used outside, so three setup-time clones are
    // avoided per `\lstnewenvironment` definition.
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        bgroup();
        state::assign_value("current_environment", Stored::String(arena::pin(&env_name)), None);
        def_macro(T_CS!("\\@currenvir"), None, Tokens!(T_OTHER!(&env_name)), None)?;
        // Convert expansion args to format for substitute_parameters
        let sub_args: Vec<Option<Cow<Tokens>>> = args.iter()
          .map(|a| match a {
            ArgWrap::None => None,
            ArgWrap::Tokens(ref t) => Some(Cow::Borrowed(t)),
            ArgWrap::Token(ref t) => Some(Cow::Owned(Tokens::new(vec![*t]))),
            other => Some(Cow::Owned(Tokens::new(ExplodeText!(other.to_string())))),
          })
          .collect();
        // Perl: lstPushValueLocally(LISTINGS_POSTAMBLE => $end->substituteParameters(@args))
        if !end_code.is_empty() {
          let end_subst = end_code.substitute_parameters(&sub_args);
          lst_push_value_locally("LISTINGS_POSTAMBLE", end_subst.unlist());
        }
        // Perl: Digest($start->substituteParameters(@args))
        // This executes \lstset{...} which activates language, styles, etc.
        if !start_code.is_empty() {
          let start_subst = start_code.substitute_parameters(&sub_args);
          let _digested = stomach::digest(start_subst)?;
        }
        let text = listings_read_raw_lines(&env_name);
        let name = lst_get_tokens("name");
        let name_opt = if name.is_empty() { None } else { Some(name) };
        let result = lst_process_display(name_opt, &text);
        Ok(Tokens::new(result))
      }
    )));
    def_macro(cs, params, expansion, None)?;
  });

  //======================================================================
  // Region 3: Display processing constructors (Perl lines 155-271)
  //======================================================================

  // Caption macros
  DefMacro!("\\lstlisting@makecaption[]{}",
    "\\def\\@captype{lstlisting}\
     \\@@add@caption@counters\
     \\@@toccaption{\\lx@format@toctitle@@{lstlisting}{\\ifx.#1.#2\\else#1\\fi}}\
     \\@@caption{\\lx@format@title@@{lstlisting}{#2}}");
  DefMacro!("\\fnum@lstlisting", "\\lstlistingname\\nobreakspace\\thelstlisting");
  DefMacro!("\\format@title@lstlisting{}", "\\lx@tag[][: ]{\\fnum@lstlisting}#1");
  DefMacro!("\\lstlisting@maketitle{}", "\\@@toccaption{#1}\\@@caption{#1}");
  DefMacro!("\\lstlisting@maketoctitle{}", "\\@@toccaption{#1}");
  DefMacro!("\\lstlistingname", "Listing");
  DefMacro!("\\lstlistlistingname", "Listings");
  DefMacro!("\\thename", "");
  DefMacro!("\\lstnumbertyperefname", "line");
  DefMacro!("\\lst@HRefStepCounter{}", "");
  // \lstname — placeholder for the current listing's filename (set inside
  // lstlisting/lstinputlisting bodies via the runtime \def\lstname{...}
  // around L1708-L1717). Pre-define as empty at top level so users can
  // reference it inside \lstset{title=\lstname,...} or other lazy-expansion
  // contexts before any listing has been opened. Driver: 1903.02915 R=1 → R=0.
  DefMacro!("\\lstname", "");

  // Inline listing constructor
  DefConstructor!("\\@listings@inline {}",
    "<ltx:text class='ltx_lstlisting' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true,
    reversion => "\\lstinline{#1}");

  // Numbered float form
  DefConstructor!("\\@listings {}",
    "<ltx:float inlist='lol' xml:id='#id' class='ltx_lstlisting'>\
     #tags\
     #1\
     </ltx:float>",
    mode => "internal_vertical",
    before_digest => {
      crate::engine::latex_constructs::before_float("lstlisting", None);
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_constructs::after_float(whatsit);
    });

  // Unnumbered float form (with caption)
  DefConstructor!("\\@@listings {}",
    "<ltx:float xml:id='#id' class='ltx_lstlisting'>\
     #tags\
     #1\
     </ltx:float>",
    mode => "internal_vertical",
    properties => {
      RefStepID!("lstlisting")?
    },
    before_digest => {
      crate::engine::latex_constructs::before_float("lstlisting", None);
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_constructs::after_float(whatsit);
    });

  // Block listing constructor — holds the actual content + base64 data.
  //
  // Audit breadcrumb: Perl listings.sty.ltxml L262 attaches an
  // `afterDigest` hook that sets whatsit properties by looking up
  // `LISTINGS_DATA_<counter>`. Rust shifts that work into a `properties`
  // closure (runs at construction time instead of digestion time).
  // Both populate the #lstdata / #lstmime / #lstenc template slots —
  // observable XML identical. Intentional afterDigest → properties
  // kind-translation so the count-diff audit (Perl 3 vs Rust 2
  // `after_digest`) is a documented false positive.
  DefConstructor!("\\@@listings@block {} {} {}",
    "<ltx:listing class='ltx_lstlisting' data='#lstdata' datamimetype='#lstmime' \
     dataencoding='#lstenc' dataname='#3'>#2</ltx:listing>",
    mode => "internal_vertical",
    properties => sub[args] {
      // Try multiple indices to find the counter
      let c = args.iter()
        .find_map(|a| {
          a.as_ref().and_then(|d| {
            let s = d.to_string();
            if s.parse::<i64>().is_ok() && !s.is_empty() { Some(s) } else { None }
          })
        })
        .unwrap_or_default();
      let data_key = s!("LISTINGS_DATA_{c}");
      let text = state::lookup_value(&data_key)
        .map(|v| v.to_string())
        .unwrap_or_default();
      let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
      Ok(stored_map!("lstdata" => Stored::String(arena::pin(&encoded)),
        "lstmime" => Stored::String(arena::pin("text/plain")),
        "lstenc" => Stored::String(arena::pin("base64"))))
    });

  // List of listings
  DefConstructor!("\\lstlistoflistings",
    "<ltx:TOC lists='lol' scope='global'><ltx:title>#name</ltx:title></ltx:TOC>",
    properties => {
      let name_toks = DigestIf!(T_CS!("\\lstlistlistingname"))?;
      stored_map!("name" => name_toks)
    });

  //======================================================================
  // Region 7: KeyVal definitions (Perl lines 652-1098)
  //======================================================================

  // 4.3 Space and placement
  DefKeyVal!("LST", "float", "");
  DefKeyVal!("LST", "floatplacement", "");
  DefKeyVal!("LST", "aboveskip", "Dimension");
  DefKeyVal!("LST", "belowskip", "Dimension");
  DefKeyVal!("LST", "lineskip", "Dimension");
  DefKeyVal!("LST", "boxpos", "");

  // 4.4 Printed range
  DefKeyVal!("LST", "print", "", "true");
  DefKeyVal!("LST", "firstline", "Number");
  DefKeyVal!("LST", "lastline", "Number");
  DefKeyVal!("LST", "showlines", "", "true");
  DefKeyVal!("LST", "emptylines", "");
  DefKeyVal!("LST", "gobble", "Number");

  // 4.5 Language and styles
  DefKeyVal!("LST", "style", "");
  DefKeyVal!("LST", "language", "");
  DefKeyVal!("LST", "alsolanguage", "");
  DefKeyVal!("LST", "defaultdialect", "");
  DefKeyVal!("LST", "printpod", "", "true");
  DefKeyVal!("LST", "usekeywordsintag", "", "true");
  DefKeyVal!("LST", "tagstyle", "");
  DefKeyVal!("LST", "markfirstintag", "");
  DefKeyVal!("LST", "makemacrouse", "", "true");

  // 4.6 Appearance
  DefKeyVal!("LST", "basicstyle", "");
  DefKeyVal!("LST", "identifierstyle", "");
  DefKeyVal!("LST", "commentstyle", "");
  DefKeyVal!("LST", "stringstyle", "");
  DefKeyVal!("LST", "keywordstyle", "");
  DefKeyVal!("LST", "ndkeywordstyle", "");
  DefKeyVal!("LST", "classoffset", "Number");
  DefKeyVal!("LST", "texcsstyle", "");
  DefKeyVal!("LST", "directivestyle", "");
  DefKeyVal!("LST", "emph", "");
  DefKeyVal!("LST", "moreemph", "");
  DefKeyVal!("LST", "deleteemph", "");
  DefKeyVal!("LST", "emphstyle", "");
  DefKeyVal!("LST", "delim", "");
  DefKeyVal!("LST", "moredelim", "");

  // 4.7 Getting characters right
  DefKeyVal!("LST", "extendedchars", "", "true");
  DefKeyVal!("LST", "inputencoding", "");
  DefKeyVal!("LST", "upquote", "", "true");
  DefKeyVal!("LST", "tabsize", "Number");
  DefKeyVal!("LST", "showtabs", "", "true");
  DefKeyVal!("LST", "tab", "");
  DefKeyVal!("LST", "showspaces", "", "true");
  DefKeyVal!("LST", "showstringspaces", "", "true");
  DefKeyVal!("LST", "formfeed", "");

  // 4.8 Line numbers
  DefKeyVal!("LST", "numbers", "");
  DefKeyVal!("LST", "stepnumber", "Number");
  DefKeyVal!("LST", "numberfirstline", "", "true");
  DefKeyVal!("LST", "numberstyle", "");
  DefKeyVal!("LST", "numbersep", "Dimension");
  DefKeyVal!("LST", "numberblanklines", "", "true");
  DefKeyVal!("LST", "firstnumber", "");
  DefKeyVal!("LST", "name", "Semiverbatim");

  // 4.9 Captions
  DefKeyVal!("LST", "title", "");
  DefKeyVal!("LST", "caption", "");
  DefKeyVal!("LST", "label", "Semiverbatim");
  DefKeyVal!("LST", "nolol", "", "true");
  DefKeyVal!("LST", "captionpos", "");
  DefKeyVal!("LST", "abovecaptionskip", "Dimension");
  DefKeyVal!("LST", "belowcaptionskip", "Dimension");

  // 4.10 Margins and line shape
  DefKeyVal!("LST", "linewidth", "Dimension");
  DefKeyVal!("LST", "xleftmargin", "Dimension");
  DefKeyVal!("LST", "xrightmargin", "Dimension");
  DefKeyVal!("LST", "resetmargins", "");
  DefKeyVal!("LST", "breaklines", "", "true");
  DefKeyVal!("LST", "prebreak", "");
  DefKeyVal!("LST", "postbreak", "");
  DefKeyVal!("LST", "breakindent", "Dimension");
  DefKeyVal!("LST", "breakautoindent", "", "true");
  DefKeyVal!("LST", "breakatwhitespace", "", "true");
  DefKeyVal!("LST", "tabs", "");
  // listings language-style internal keys (lstlang*.sty / language=...
  // declarations). Perl listings.sty.ltxml leaves them unregistered;
  // Rust-only divergence paired with `21e730e71e` Info→Warn promotion.
  DefKeyVal!("LST", "procnamekeys", "");
  DefKeyVal!("LST", "moreprocnamekeys", "");
  DefKeyVal!("LST", "MoreSelectCharTable", "");

  // 4.11 Frames
  DefKeyVal!("LST", "frame", "");
  DefKeyVal!("LST", "framearound", "");
  DefKeyVal!("LST", "framesep", "Dimension");
  DefKeyVal!("LST", "rulesep", "Dimension");
  DefKeyVal!("LST", "framerule", "Dimension");
  DefKeyVal!("LST", "framexleftmargin", "Dimension");
  DefKeyVal!("LST", "framexrightmargin", "Dimension");
  DefKeyVal!("LST", "framextopmargin", "Dimension");
  DefKeyVal!("LST", "framexbottommargin", "Dimension");
  DefKeyVal!("LST", "backgroundcolor", "");
  DefKeyVal!("LST", "rulecolor", "");
  DefKeyVal!("LST", "fillcolor", "");
  DefKeyVal!("LST", "rulesepcolor", "");
  DefKeyVal!("LST", "frameround", "");
  DefKeyVal!("LST", "frameshape", "");

  // 4.12 Indexing
  DefKeyVal!("LST", "index", "");
  DefKeyVal!("LST", "moreindex", "");
  DefKeyVal!("LST", "deleteindex", "");
  DefKeyVal!("LST", "indexstyle", "");
  DefMacro!("\\lstindexmacro{}", "\\index{{\\ttfamily #1}}");

  // 4.13 Column alignment
  DefKeyVal!("LST", "columns", "");
  DefKeyVal!("LST", "flexiblecolumns", "", "true");
  DefKeyVal!("LST", "keepspaces", "", "true");
  DefKeyVal!("LST", "basewidth", "");
  DefKeyVal!("LST", "fontadjust", "", "true");

  // 4.14 Escaping to LaTeX
  DefKeyVal!("LST", "texcl", "", "true");
  DefKeyVal!("LST", "mathescape", "", "true");
  DefKeyVal!("LST", "escapechar", "");
  DefKeyVal!("LST", "escapeinside", "");
  DefKeyVal!("LST", "escapebegin", "");
  DefKeyVal!("LST", "escapeend", "");

  // 4.15 Interface to fancyvrb
  DefKeyVal!("LST", "fancyvrb", "", "true");
  DefKeyVal!("LST", "fvcmdparams", "");
  DefKeyVal!("LST", "morefvcmdparams", "");

  //======================================================================
  // Region 8: KeyVal definitions part 2 (Perl lines 1098-1231)
  //======================================================================

  // 4.17 Language definitions
  DefKeyVal!("LST", "keywordprefix", "");
  DefKeyVal!("LST", "keywords", "Semiverbatim");
  DefKeyVal!("LST", "morekeywords", "Semiverbatim");
  DefKeyVal!("LST", "deletekeywords", "Semiverbatim");
  DefKeyVal!("LST", "ndkeywords", "Semiverbatim");
  DefKeyVal!("LST", "morendkeywords", "Semiverbatim");
  DefKeyVal!("LST", "deletendkeywords", "Semiverbatim");
  DefKeyVal!("LST", "texcs", "");
  DefKeyVal!("LST", "moretexcs", "");
  DefKeyVal!("LST", "deletetexcs", "");
  DefKeyVal!("LST", "directives", "Semiverbatim");
  DefKeyVal!("LST", "moredirectives", "Semiverbatim");
  DefKeyVal!("LST", "deletedirectives", "Semiverbatim");
  DefKeyVal!("LST", "sensitive", "", "true");
  DefKeyVal!("LST", "alsoletter", "");
  DefKeyVal!("LST", "alsodigit", "");
  DefKeyVal!("LST", "alsoother", "");
  DefKeyVal!("LST", "otherkeywords", "");

  // Tags and strings and comments
  DefKeyVal!("LST", "tag", "");
  DefKeyVal!("LST", "string", "");
  DefKeyVal!("LST", "morestring", "");
  DefKeyVal!("LST", "deletestring", "");
  DefKeyVal!("LST", "comment", "");
  DefKeyVal!("LST", "morecomment", "");
  DefKeyVal!("LST", "deletecomment", "");
  DefKeyVal!("LST", "keywordcomment", "");
  DefKeyVal!("LST", "morekeywordcomment", "");
  DefKeyVal!("LST", "deletekeywordcomment", "");
  DefKeyVal!("LST", "keywordcommentsemicolon", "");
  DefKeyVal!("LST", "podcomment", "", "true");
  DefKeyVal!("LST", "linerange", "");
  DefKeyVal!("LST", "literate", "");

  // \lstdefinestyle — define a named style
  DefPrimitive!("\\lstdefinestyle{} RequiredKeyVals:LST", sub[(style, kv)] {
    let style_name = style.to_string().to_uppercase().replace(char::is_whitespace, "");
    let key = s!("LST@STYLE@{style_name}");
    state::assign_value(&key, Stored::from(kv), None);
  });

  // \lstdefinelanguage — define a language
  // Perl: \@lstdefinelanguage [dialect]{language}[base_dialect]{base_language} ... RequiredKeyVals:LST [aspects]
  // When base_language is non-empty, adds language=[base_dialect]base_language to the keyvals,
  // so that language activation recursively activates the base language chain.
  DefPrimitive!("\\@lstdefinelanguage []{}[]{} SkipSpaces RequiredKeyVals:LST []", sub[(dialect, language, base_dialect, base_language, kv, _aspects)] {
    // Build base language tokens: [dialect]language
    // Perl: push @base, T_OTHER('['), $base_dialect->unlist, T_OTHER(']') if $base_dialect;
    //       push @base, $base_language->unlist;
    //       $keyvals->setValue('language', Tokens(@base)) if @base;
    let base_lang_str = base_language.to_string();
    let mut kv = kv;
    if !base_lang_str.trim().is_empty() {
      let mut base_tokens = Vec::new();
      let base_dialect_str = base_dialect.as_ref().map(|d| d.to_string()).unwrap_or_default();
      if !base_dialect_str.is_empty() {
        base_tokens.push(T_OTHER!("["));
        base_tokens.extend(Explode!(&base_dialect_str));
        base_tokens.push(T_OTHER!("]"));
      }
      base_tokens.extend(Explode!(&base_lang_str));
      kv.set_value("language", ArgWrap::Tokens(Tokens::new(base_tokens)), false)?;
    }
    let lang = language.to_string().to_uppercase().replace(char::is_whitespace, "");
    let mut name = s!("LST@LANGUAGE@{lang}");
    let dialect_str = dialect.map(|d| d.to_string()).unwrap_or_default();
    if !dialect_str.is_empty() {
      let d = dialect_str.to_uppercase().replace(char::is_whitespace, "");
      name = s!("{name}${d}");
    }
    state::assign_value(&name, Stored::from(kv), Some(Scope::Global));
  });
  DefMacro!("\\lstdefinelanguage []{}",
    "\\@ifnextchar[{\\@lstdefinelanguage[#1]{#2}}{\\@lstdefinelanguage[#1]{#2}[]{}}");
  Let!(T_CS!("\\lst@definelanguage"), T_CS!("\\lstdefinelanguage"));

  DefPrimitive!("\\lstalias []{} []{}", None);
  DefPrimitive!("\\lstloadlanguages Semiverbatim", None);

  //======================================================================
  // Region 9 constructors: Listing line structure
  //======================================================================

  DefConstructor!("\\@lst@startline{}",
    "<ltx:listingline xml:id='#id'>#1",
    properties => { RefStepID!("lstnumber")? });
  DefConstructor!("\\@lst@endline", "</ltx:listingline>");
  DefConstructor!("\\@lst@linenumber{}",
    "<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>");
  DefConstructor!("\\@lst@visible@space", "\u{2423}",
    enter_horizontal => true);

  // Literate replacement
  DefConstructor!("\\@listingLiterate {}",
    "<ltx:text class='ltx_lst_literate' _noautoclose='1'>#1</ltx:text>",
    enter_horizontal => true);

  // Keyword styling
  DefConstructor!("\\@listingKeyword Semiverbatim {}",
    "?#class(<ltx:text class='ltx_lst_#class' _noautoclose='1'>#2</ltx:text>)(#2)",
    enter_horizontal => true);

  // Group styling — wraps a class of tokens
  DefConstructor!("\\@listingGroup Semiverbatim {}",
    "<ltx:text class='#1'>#2",
    enter_horizontal => true,
    after_construct => sub[document, _whatsit] {
      if let Some(node) = document.get_element() {
        if latexml_core::document::get_node_qname(&node) == arena::pin_static("ltx:text") {
          document.close_element("ltx:text")?;
        }
      }
    });

  //======================================================================
  // Region 7 continued: \lst@@ handler macros
  //======================================================================

  // Style handler — Perl: lstActivate($values) from LST@STYLE@name
  // \lstdefinestyle stores as Stored::KeyVals, so match both KeyVals and Tokens
  DefMacro!("\\lst@@style Until:\\end", sub [args] {
    let s = args[0].to_string().to_uppercase().replace(char::is_whitespace, "");
    let key = s!("LST@STYLE@{s}");
    if let Some(stored) = state::lookup_value(&key) {
      match stored {
        Stored::KeyVals(kv) => lst_activate(Some(&kv)),
        Stored::Tokens(kv_tokens) => { let _ = stomach::digest(kv_tokens); },
        _ => {},
      }
    }
    Tokens!()
  });

  // Language handler
  DefMacro!("\\lst@@language [] Until:\\end", sub [args] {
    // Perl: lstClearLanguage(); lstActivateLanguage($_[2], $_[1]);
    lst_clear_language();
    let lang = args[1].to_string().to_uppercase().replace(char::is_whitespace, "");
    state::assign_value("LST@language", Stored::String(arena::pin(&lang)), None);
    let dialect = args[0].clone().owned_tokens()
      .map(|t| t.to_string().trim().to_string())
      .filter(|s| !s.is_empty());
    lst_activate_language(&lang, dialect.as_deref());
    lst_push_value_locally("LISTINGS_PREAMBLE", vec![T_CS!("\\lst@@@set@language")]);
    Tokens!()
  });

  DefConstructor!("\\lst@@@set@language",
    sub[document, _args, props] {
      let lang = props.get("language").map(|v| v.to_string()).unwrap_or_default();
      if !lang.is_empty() {
        // Perl: $lang = "$2_$1" if $lang =~ /^\[([^\]]*)\](.*)$/;
        let lang = if lang.starts_with('[') {
          if let Some(close) = lang.find(']') {
            let dialect = &lang[1..close];
            let base = &lang[close + 1..];
            format!("{base}_{dialect}")
          } else { lang }
        } else { lang };
        document.add_class(&mut document.get_element().unwrap(), &s!("ltx_lst_language_{lang}"))?;
      }
    },
    properties => {
      let lang = lst_get_literal("language");
      stored_map!("language" => Stored::String(arena::pin(&lang)))
    });

  DefMacro!("\\lst@@alsolanguage [] Until:\\end", sub [args] {
    let lang = args[1].to_string().to_uppercase().replace(char::is_whitespace, "");
    lst_activate_language(&lang, None);
    Tokens!()
  });

  DefMacro!("\\lst@@defaultdialect[] Until:\\end", sub [args] {
    let lang = args[1].to_string().to_uppercase().replace(char::is_whitespace, "");
    let key = s!("LSTDD@{lang}");
    let dialect_tokens = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    state::assign_value(&key, Stored::Tokens(dialect_tokens), None);
    Tokens!()
  });

  // Appearance style handlers
  DefMacro!("\\lst@@identifierstyle Until:\\end", sub [args] {
    lst_set_class_style("identifiers", args[0].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@commentstyle Until:\\end", sub [args] {
    lst_set_class_style("comments", args[0].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@stringstyle Until:\\end", sub [args] {
    lst_set_class_style("strings", args[0].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@keywordstyle [Number] OptionalMatch:* Until:\\end", sub [args] {
    let class = lst_class_name("keywords", args[0].to_string().parse().ok());
    lst_set_class_style(&class, args[2].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@ndkeywordstyle Until:\\end", sub [args] {
    lst_set_class_style("keywords2", args[0].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@texcsstyle OptionalMatch:* [Number] Until:\\end", sub [args] {
    // Perl: excludeslash => !$_[1] — when * is NOT present, excludeslash is true
    let star_present = !args[0].is_empty();
    let class = lst_class_name("texcss", args[1].to_string().parse().ok());
    let exclude = if star_present { "false" } else { "true" };
    lst_set_class_style(&class, args[2].clone().owned_tokens(), vec![("excludeslash", exclude)]);
    Tokens!()
  });
  DefMacro!("\\lst@@directivestyle Until:\\end", sub [args] {
    lst_set_class_style("directives", args[0].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@tagstyle Until:\\end", sub [args] {
    lst_set_class_style("tags", args[0].clone().owned_tokens(), vec![]);
    Tokens!()
  });
  DefMacro!("\\lst@@emphstyle [Number] Until:\\end", sub [args] {
    let class = lst_class_name("emph", args[0].to_string().parse().ok());
    lst_set_class_style(&class, args[1].clone().owned_tokens(), vec![]);
    Tokens!()
  });

  // Keyword handlers
  DefMacro!("\\lst@@keywords [Number] Until:\\end", sub [args] {
    let class = lst_class_name("keywords", args[0].to_string().parse().ok());
    let words = args[1].clone().owned_tokens();
    lst_set_class_words(&class, &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@morekeywords [Number] Until:\\end", sub [args] {
    let class = lst_class_name("keywords", args[0].to_string().parse().ok());
    let words = args[1].clone().owned_tokens();
    lst_add_class_words(&class, &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@deletekeywords [Number] Until:\\end", sub [args] {
    let class = lst_class_name("keywords", args[0].to_string().parse().ok());
    let words = args[1].clone().owned_tokens();
    lst_delete_class_words(&class, &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@ndkeywords Until:\\end", sub [args] {
    let words = args[0].clone().owned_tokens();
    lst_set_class_words("keywords2", &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@morendkeywords Until:\\end", sub [args] {
    let words = args[0].clone().owned_tokens();
    lst_add_class_words("keywords2", &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@deletendkeywords Until:\\end", sub [args] {
    let words = args[0].clone().owned_tokens();
    lst_delete_class_words("keywords2", &words, None);
    Tokens!()
  });

  // TeX CS keywords
  DefMacro!("\\lst@@texcs [Number] Until:\\end", sub [args] {
    state::assign_value("LST@TEXCS", Stored::Bool(true), None);
    let words = args[1].clone().owned_tokens();
    lst_set_class_words("texcss", &words, Some("\\"));
    Tokens!()
  });
  DefMacro!("\\lst@@moretexcs [Number] Until:\\end", sub [args] {
    state::assign_value("LST@TEXCS", Stored::Bool(true), None);
    let words = args[1].clone().owned_tokens();
    lst_add_class_words("texcss", &words, Some("\\"));
    Tokens!()
  });
  DefMacro!("\\lst@@deletetexcs [Number] Until:\\end", sub [args] {
    let words = args[1].clone().owned_tokens();
    lst_delete_class_words("texcss", &words, Some("\\"));
    Tokens!()
  });

  // Directives
  DefMacro!("\\lst@@directives Until:\\end", sub [args] {
    let words = args[0].clone().owned_tokens();
    lst_set_class_words("directives", &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@moredirectives Until:\\end", sub [args] {
    let words = args[0].clone().owned_tokens();
    lst_add_class_words("directives", &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@deletedirectives Until:\\end", sub [args] {
    let words = args[0].clone().owned_tokens();
    lst_delete_class_words("directives", &words, None);
    Tokens!()
  });

  // Emph
  DefMacro!("\\lst@@emph [Number] Until:\\end", sub [args] {
    let class = lst_class_name("emph", args[0].to_string().parse().ok());
    let words = args[1].clone().owned_tokens();
    lst_set_class_words(&class, &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@moreemph [Number] Until:\\end", sub [args] {
    let class = lst_class_name("emph", args[0].to_string().parse().ok());
    let words = args[1].clone().owned_tokens();
    lst_add_class_words(&class, &words, None);
    Tokens!()
  });
  DefMacro!("\\lst@@deleteemph [Number] Until:\\end", sub [args] {
    let class = lst_class_name("emph", args[0].to_string().parse().ok());
    let words = args[1].clone().owned_tokens();
    lst_delete_class_words(&class, &words, None);
    Tokens!()
  });

  // Character class handlers
  DefMacro!("\\lst@@alsoletter Until:\\end", sub [args] {
    let chars = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    lst_set_character_class("letter", &chars);
    Tokens!()
  });
  DefMacro!("\\lst@@alsodigit Until:\\end", sub [args] {
    let chars = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    lst_set_character_class("digit", &chars);
    Tokens!()
  });
  DefMacro!("\\lst@@alsoother Until:\\end", sub [args] {
    let chars = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    lst_set_character_class("other", &chars);
    Tokens!()
  });

  // Comment/String/Delimiter handlers
  // Perl: lstAddDelimiter('comment', $_[1], 'commentstyle', $_[3])
  DefMacro!("\\lst@@comment [] [] Until:\\end", sub [args] {
    let delims = args[2].clone().owned_tokens();
    lst_add_delimiter("comment", &args[0].to_string(), "commentstyle", delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@morecomment [] [] Until:\\end", sub [args] {
    let delims = args[2].clone().owned_tokens();
    lst_add_delimiter("comment", &args[0].to_string(), "commentstyle", delims, false);
    Tokens!()
  });
  // Perl: lstAddDelimiter('string', $_[1], 'stringstyle', $_[2])
  DefMacro!("\\lst@@string [] Until:\\end", sub [args] {
    let delims = args[1].clone().owned_tokens();
    lst_add_delimiter("string", &args[0].to_string(), "stringstyle", delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@morestring [] Until:\\end", sub [args] {
    let delims = args[1].clone().owned_tokens();
    lst_add_delimiter("string", &args[0].to_string(), "stringstyle", delims, false);
    Tokens!()
  });
  // Perl: lstAddDelimiter('delimiter', $_[3], $_[4], $_[5], ...)
  // Perl listings.sty.ltxml:821-830
  //   DefMacro('\lst@@delim OptionalMatch:* OptionalMatch:* [] [] Until:\end', sub {
  //     lstAddDelimiter('delimiter', $_[3], $_[4], $_[5],
  //       ($_[1] ? (recursive => 1) : ()),
  //       ($_[2] ? (cummulative => 1) : ())); });
  // First `*` => recursive (other delimiters still match inside). We deliberately
  // do NOT propagate `args[0].is_some()` here because the default `alsoletter`
  // set bundles `@`, `$`, `_` with the alphabet, and recursive ID_RE matching
  // then greedily swallows the closing delim character (e.g. `>@`-as-identifier
  // for `\moredelim=**[is]…{@}{@}` in tests/tikz/various_colors.tex). Perl has
  // the same latent ID_RE bug; until it is patched upstream (or sidestepped by
  // delimiter-aware character-class scoping) we keep the recursive=false
  // behavior that this test corpus relies on. Tracked as a parity follow-up.
  DefMacro!("\\lst@@delim OptionalMatch:* OptionalMatch:* [] [] Until:\\end", sub [args] {
    let delims = args[4].clone().owned_tokens();
    lst_add_delimiter("delimiter", &args[2].to_string(), &args[3].to_string(), delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@moredelim OptionalMatch:* OptionalMatch:* [] [] Until:\\end", sub [args] {
    let delims = args[4].clone().owned_tokens();
    lst_add_delimiter("delimiter", &args[2].to_string(), &args[3].to_string(), delims, false);
    Tokens!()
  });

  // Numbers handler
  DefPrimitive!("\\lst@@numbers Until:\\end", sub [args] { let _val = &args[0];
    lst_push_value_locally("LISTINGS_PREAMBLE", vec![T_CS!("\\lst@@@set@numbers")]);
  });
  DefConstructor!("\\lst@@@set@numbers",
    sub[document, _args, props] {
      let position = props.get("position").map(|v| v.to_string()).unwrap_or_default();
      if position != "none" && !position.is_empty() {
        document.add_class(&mut document.get_element().unwrap(), &s!("ltx_lst_numbers_{position}"))?;
      }
    },
    properties => {
      let position = lst_get_literal("numbers");
      stored_map!("position" => Stored::String(arena::pin(&position)))
    });

  // Frame handler
  DefPrimitive!("\\lst@@frame Until:\\end", sub [args] {
    let name = args[0].to_string();
    let frame = match name.as_str() {
      "none" => None,
      "leftline" => Some("left"),
      "topline" => Some("top"),
      "bottomline" => Some("bottom"),
      "lines" => Some("topbottom"),
      "single" => Some("rectangle"),
      "shadowbox" => Some("rectangle"),
      _ => None,
    };
    if let Some(f) = frame {
      state::assign_value("LISTINGS_FRAME", Stored::String(arena::pin(f)), None);
    } else {
      state::assign_value("LISTINGS_FRAME", Stored::None, None);
    }
    lst_push_value_locally("LISTINGS_PREAMBLE", vec![T_CS!("\\lst@@@set@frame")]);
  });
  DefConstructor!("\\lst@@@set@frame",
    sub[document, _args, props] {
      let frame = props.get("frame").map(|v| v.to_string()).unwrap_or_default();
      if !frame.is_empty() {
        document.set_attribute(&mut document.get_element().unwrap(), "framed", &frame)?;
      }
    },
    properties => {
      let frame = state::lookup_value("LISTINGS_FRAME").map(|v| v.to_string()).unwrap_or_default();
      stored_map!("frame" => Stored::String(arena::pin(&frame)))
    });

  // Background color handler
  DefPrimitive!("\\lst@@backgroundcolor Until:\\end", sub [args] {
    let cmd_toks = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    let color = lst_extract_color(&cmd_toks);
    if let Some(c) = color {
      state::assign_value("LISTINGS_BACKGROUND", Stored::String(arena::pin(&c)), Some(Scope::Global));
    }
    lst_push_value_locally("LISTINGS_PREAMBLE_BEFORE", vec![T_CS!("\\lst@@@set@background")]);
  });
  DefPrimitive!("\\lst@@@set@background", {
    if let Some(Stored::String(bg)) = state::lookup_value("LISTINGS_BACKGROUND") {
      if let Some(c) = arena::with(bg, latexml_core::common::color::Color::from_stored) {
        merge_font(Font { bg: Some(c), ..Font::default() });
      }
    }
    // Clear after use so subsequent listings don't inherit
    state::assign_value("LISTINGS_BACKGROUND", Stored::None, Some(Scope::Global));
  });

  // Rule color handler
  DefPrimitive!("\\lst@@rulecolor Until:\\end", sub [args] {
    let cmd_toks = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    let color = lst_extract_color(&cmd_toks);
    if let Some(c) = color {
      state::assign_value("LISTINGS_RULECOLOR", Stored::String(arena::pin(&c)), None);
    }
    lst_push_value_locally("LISTINGS_PREAMBLE", vec![T_CS!("\\lst@@@set@rulecolor")]);
  });
  DefConstructor!("\\lst@@@set@rulecolor",
    sub[document, _args, props] {
      let color_stored = props.get("color").map(|v| v.to_string()).unwrap_or_default();
      if !color_stored.is_empty() {
        // Convert from stored format ("rgb r g b") to attribute format ("#RRGGBB")
        let color_hex = latexml_core::common::color::Color::from_stored(&color_stored)
          .map(|c| c.to_attribute())
          .unwrap_or(color_stored);
        document.set_attribute(&mut document.get_element().unwrap(), "framecolor", &color_hex)?;
      }
    },
    properties => {
      let color = state::lookup_value("LISTINGS_RULECOLOR").map(|v| v.to_string()).unwrap_or_default();
      stored_map!("color" => Stored::String(arena::pin(&color)))
    });

  // Extended chars handler — Perl: listings.sty.ltxml L836-845
  // Adds/removes characters 128-255 from the letter class
  DefMacro!("\\lst@@extendedchars Until:\\end", sub [args] {
    let val = args[0].to_string();
    let enable = val == "true";
    for code in 128u32..=255 {
      if let Some(ch) = char::from_u32(code) {
        let escaped = regex::escape(&ch.to_string());
        let key = s!("LST_CHAR@letter@{escaped}");
        if enable {
          state::assign_value(&key, Stored::Bool(true), None);
        } else {
          state::assign_value(&key, Stored::None, None);
        }
      }
    }
    Tokens!()
  });

  // texcl handler
  DefMacro!("\\lst@@texcl Until:\\end", sub [args] {
    if args[0].eq_text("true") {
      state::assign_value("LST_CLASSES@comments@eval", Stored::Bool(true), None);
    }
    Tokens!()
  });

  // mathescape handler
  DefMacro!("\\lst@@mathescape Until:\\end", sub [args] {
    if args[0].eq_text("true") {
      state::assign_value("LST_DELIM@$@open", Stored::String(arena::pin("\\$")), None);
      state::assign_value("LST_DELIM@$@close", Stored::String(arena::pin("\\$")), None);
      state::assign_value("LST_DELIM@$@class", Stored::String(arena::pin("mathescape")), None);
      state::assign_value("LST_DELIM@$@escape", Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@mathescape@eval", Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@mathescape@begin",
        Stored::Tokens(Tokens::new(vec![T_MATH!()])), None);
      state::assign_value("LST_CLASSES@mathescape@end",
        Stored::Tokens(Tokens::new(vec![T_MATH!()])), None);
      // Perl: delete LookupValue('LST_CHARACTERS')->{letter}{'\$'}
      state::assign_value("LST_CHAR@letter@\\$", Stored::None, None);
    } else {
      // Perl: delete(LookupValue('LST_DELIMITERS')->{'$'})
      state::assign_value("LST_DELIM@$@open", Stored::None, None);
      state::assign_value("LST_DELIM@$@close", Stored::None, None);
      state::assign_value("LST_DELIM@$@class", Stored::None, None);
      state::assign_value("LST_DELIM@$@escape", Stored::None, None);
    }
    Tokens!()
  });

  // escapechar handler
  DefMacro!("\\lst@@escapechar Until:\\end", sub [args] {
    let esc = lst_deslash(&args[0].to_string());
    if !esc.is_empty() {
      let esc_re = regex::escape(&esc);
      state::assign_value(&s!("LST_DELIM@{esc}@open"), Stored::String(arena::pin(&esc_re)), None);
      state::assign_value(&s!("LST_DELIM@{esc}@close"), Stored::String(arena::pin(&esc_re)), None);
      state::assign_value(&s!("LST_DELIM@{esc}@class"), Stored::String(arena::pin("evaluate")), None);
      state::assign_value(&s!("LST_DELIM@{esc}@escape"), Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@evaluate@eval", Stored::Bool(true), None);
      // Perl: delete LookupValue('LST_CHARACTERS')->{letter}{$escapere}
      state::assign_value(&s!("LST_CHAR@letter@{esc_re}"), Stored::None, None);
      // Register in delimiter keys list
      lst_push_value_locally("LST_DELIM_KEYS", vec![T_OTHER!(&esc)]);
    }
    Tokens!()
  });

  // escapeinside handler
  DefMacro!("\\lst@@escapeinside Until:\\end", "\\ifx.#1.\\else\\lst@@escapeinside@#1\\end\\fi");
  DefMacro!("\\lst@@escapeinside@ {} {} Until:\\end", sub [args] {
    let esc1 = lst_deslash(&args[0].to_string());
    let esc2 = lst_deslash(&args[1].to_string());
    if !esc1.is_empty() && !esc2.is_empty() {
      state::assign_value(&s!("LST_DELIM@{esc1}@open"), Stored::String(arena::pin(regex::escape(&esc1))), None);
      state::assign_value(&s!("LST_DELIM@{esc1}@close"), Stored::String(arena::pin(regex::escape(&esc2))), None);
      state::assign_value(&s!("LST_DELIM@{esc1}@class"), Stored::String(arena::pin("evaluate")), None);
      state::assign_value(&s!("LST_DELIM@{esc1}@escape"), Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@evaluate@eval", Stored::Bool(true), None);
      // Register in delimiter keys so it's picked up by delimiter regex builder
      lst_push_value_locally("LST_DELIM_KEYS", vec![T_OTHER!(&esc1)]);
    }
    Tokens!()
  });
  DefMacro!("\\lst@@escapebegin Until:\\end", sub [args] {
    let val = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    state::assign_value("LST_CLASSES@evaluate@begin", Stored::Tokens(val), None);
    Tokens!()
  });
  DefMacro!("\\lst@@escapeend Until:\\end", sub [args] {
    let val = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    state::assign_value("LST_CLASSES@evaluate@end", Stored::Tokens(val), None);
    Tokens!()
  });

  // literate handler — parses {pattern}{replacement}length triples
  // Perl: UnshiftValue(LST_LITERATE => [ToString($pattern), $replacement, $star, $length])
  DefMacro!("\\lst@@literate OptionalMatch:* Until:\\end", sub [args] {
    let protected = args[0].eq_text("*");
    let toks = args[1].clone().owned_tokens().unwrap_or(Tokens!());
    let tokens: Vec<Token> = toks.unlist();
    let mut i = 0;
    while i < tokens.len() {
      // Skip whitespace
      while i < tokens.len() && tokens[i].get_catcode() == Catcode::SPACE { i += 1; }
      if i >= tokens.len() { break; }
      // Read pattern: balanced group
      if tokens[i].get_catcode() != Catcode::BEGIN { break; }
      let pattern = read_balanced_group(&tokens, &mut i);
      // Skip whitespace
      while i < tokens.len() && tokens[i].get_catcode() == Catcode::SPACE { i += 1; }
      if i >= tokens.len() { break; }
      // Read replacement: balanced group
      if tokens[i].get_catcode() != Catcode::BEGIN { break; }
      let replacement = Tokens::new(read_balanced_group(&tokens, &mut i));
      // Skip whitespace
      while i < tokens.len() && tokens[i].get_catcode() == Catcode::SPACE { i += 1; }
      // Read length: number token(s)
      while i < tokens.len() && tokens[i].get_catcode() != Catcode::SPACE
        && tokens[i].get_catcode() != Catcode::BEGIN { i += 1; }
      let pattern_str: String = pattern.iter().map(|t| t.to_string()).collect();
      if !pattern_str.is_empty() {
        // Store as individual entries keyed by pattern
        let key = s!("LST_LIT@{pattern_str}");
        state::assign_value(&key, Stored::from(replacement.clone()), None);
        let prot_key = s!("LST_LIT@{pattern_str}@protected");
        state::assign_value(&prot_key, Stored::Bool(protected), None);
        // Add to pattern list
        lst_push_value_locally("LST_LITERATE_KEYS", vec![T_OTHER!(&pattern_str)]);
      }
    }
    Tokens!()
  });

  // Index handlers (simplified)
  DefMacro!("\\lst@@index [Number] [] Until:\\end", sub [args] {
    let _ = &args;
    Tokens!()
  });
  DefMacro!("\\lst@@moreindex [Number] [] Until:\\end", sub [args] {
    let _ = &args;
    Tokens!()
  });
  DefMacro!("\\lst@@deleteindex [Number] [] Until:\\end", sub [args] {
    let _ = &args;
    Tokens!()
  });
  DefMacro!("\\lst@@indexstyle [Number] Until:\\end", sub [args] {
    let _ = &args;
    Tokens!()
  });

  // Perl: lstAddDelimiter('delimiter', $_[3], 'tagstyle', $_[4],
  //         ($_[1] ? (recursive => 1) : ()),
  //         ($_[2] ? (cummulative => 1) : ())); — listings.sty.ltxml:1201-1204.
  // First `*` => recursive (inner delimiters such as strings still match);
  // second `*` => cumulative (currently unmodelled).
  DefMacro!("\\lst@@tag OptionalMatch:* OptionalMatch:* [] Until:\\end", sub [args] {
    let delims = args[3].clone().owned_tokens();
    let recursive = args[0].is_some();
    lst_add_delimiter("delimiter", &args[2].to_string(), "tagstyle", delims, recursive);
    Tokens!()
  });

  //======================================================================
  // Region 10: Configuration (Perl lines 1562-1626)
  //======================================================================

  // Default lstset values
  RawTeX!(r##"\lstset{
 alsoletter={abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ@$\_},
 alsodigit={0123456789},
 alsoother={!"#\%&'()*+,-./:;<=>?[\\]^\{|\}~},
 float=tbp,floatplacement=tbp,aboveskip=\medskipamount,belowskip=\medskipamount,
 lineskip=0pt,boxpos=c,
 print=true,firstline=1,lastline=9999999,showlines=false,emptylines=9999999,gobble=0,
 style={},language={},printpod=false,usekeywordsintag=true,tagstyle={},
 markfirstintag=false,makemacrouse=true,
 basicstyle={},identifierstyle={},commentstyle=\itshape,stringstyle={},
 keywordstyle=\bfseries,classoffset=0,
 emph={},delim={},
 extendedchars=false,inputencoding={},upquote=false,tabsize=8,showtabs=false,
 tabs={},showspaces=false,showstringspaces=true,formfeed=\bigbreak,
 numbers=none,stepnumber=1,numberfirstline=false,numberstyle={},numbersep=10pt,
 numberblanklines=true,firstnumber=auto,name={},
 title={},caption={},label={},nolol=false,
 captionpos=t,abovecaptionskip=\smallskipamount,belowcaptionskip=\smallskipamount,
 linewidth=\linewidth,xleftmargin=0pt,xrightmargin=0pt,resetmargins=false,breaklines=false,
 prebreak={},postbreak={},breakindent=20pt,breakautoindent=true,
 frame=none,frameround=ffff,framesep=3pt,rulesep=2pt,framerule=0.4pt,
 framexleftmargin=0pt,framexrightmargin=0pt,framextopmargin=0pt,framexbottommargin=0pt,
 backgroundcolor={},rulecolor={},fillcolor={},rulesepcolor={},
 frameshape={},
 index={},indexstyle=\lstindexmacro,
 columns=[c]fixed,flexiblecolumns=false,keepspaces=false,basewidth={0.6em,0.45em},
 fontadjust=false,texcl=false,mathescape=false,escapechar={},escapeinside={},
 escapebegin={},escapeend={},
 fancyvrb=false,fvcmdparams=\overlay1,morefvcmdparams={},
 ndkeywordstyle=keywordstyle,texcsstyle=keywordstyle,directivestyle=keywordstyle
}"##);

  // Load language configuration files
  InputDefinitions!("listings", extension => Some(std::borrow::Cow::Borrowed("cfg")));

  // Internal macros used by sibling bindings (e.g. cleveref) AND by the
  // lang-file raw loads below (lstlang3.sty in particular calls
  // `\lst@AddToHook{...}{...}` during definition). Define BEFORE the lang
  // loop so the raw .sty files don't error on undefined `\lst@AddToHook`.
  // Driver: 2001.11875 (lstlang3.sty raw-load `\lst@AddToHook` undefined).
  DefMacro!("\\lst@UseHook{}", "\\csname\\@lst hk@#1\\endcsname");
  DefMacro!("\\lst@AddToHook{}{}", "");
  DefMacro!("\\lst@AddToHookExe{}{}", "");
  DefMacro!("\\lst@AddTo {}{}", "\\expandafter\\gdef\\expandafter#1\\expandafter{#1#2}");
  DefMacro!("\\@lst", "lst");

  // Load all language files eagerly
  let lang_files_str = stomach::digest(T_CS!("\\lstlanguagefiles"))?.to_string();
  let lang_files: Vec<String> = lang_files_str
    .replace(char::is_whitespace, "")
    .split(',')
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect();
  for file in &lang_files {
    let _ = input_definitions(file, NewDefault!(InputDefinitionOptions, noerror => true));
  }
});

//======================================================================
// Region 5 continued: lstActivate implementation
//======================================================================

/// Perl: lstActivate — process a set of keyvals, dispatching to \lst@@ handlers.
/// Iterates over pairs, looks up \lst@@KEY macros, digests them for effect,
/// and stores LST@KEY => value in state.
fn lst_activate(kv: Option<&KeyVals>) {
  let kv = match kv {
    Some(kv) => kv,
    None => return,
  };

  // Copy previous LST_ tables into current scope (for grouping effect)
  for table in &[
    "LST_CHARACTERS",
    "LST_CLASSES",
    "LST_WORDS",
    "LST_DELIMITERS",
  ] {
    if let Some(stored) = state::lookup_value(table) {
      state::assign_value(table, stored, None);
    }
  }
  // Copy LST_LITERAL
  if let Some(stored) = state::lookup_value("LST_LITERAL") {
    state::assign_value("LST_LITERAL", stored, None);
  }

  // Iterate over key-value pairs, in order
  for (key, val) in kv.get_pairs() {
    let val_tokens = lst_un_group(val.clone().owned_tokens());
    let cs = T_CS!(s!("\\lst@@{key}"));
    if state::has_meaning(&cs) {
      // Defaults for bare keys are already resolved during KeyVals parsing (add_value with
      // use_default). Digest: \lst@@KEY <value> \end
      let mut digest_tokens = vec![cs];
      if let Some(ref val_tks) = val_tokens {
        digest_tokens.extend(val_tks.unlist_ref().iter().cloned());
      }
      digest_tokens.push(T_CS!("\\end"));
      let _ = stomach::digest(Tokens::new(digest_tokens));
    }
    // Store LST@KEY => value
    let state_key = s!("LST@{key}");
    match val_tokens {
      Some(tks) => state::assign_value(&state_key, Stored::Tokens(tks), None),
      None => state::assign_value(&state_key, Stored::Tokens(Tokens!()), None),
    }
  }
}

/// Perl: lstActivateLanguage — load and activate a language definition.
/// Handles dialect lookup: first checks LSTDD@LANG for default dialect,
/// then looks up LST@LANGUAGE@LANG$DIALECT.
fn lst_activate_language(language: &str, dialect: Option<&str>) {
  let lang = language.to_uppercase().replace(char::is_whitespace, "");
  if lang.is_empty() {
    return;
  }
  // Determine dialect: explicit, or from default dialect state
  let dialect_str = match dialect {
    Some(d) if !d.is_empty() => d.to_uppercase().replace(char::is_whitespace, ""),
    _ => {
      let dd_key = s!("LSTDD@{lang}");
      state::lookup_value(&dd_key)
        .map(|v| {
          v.to_string()
            .to_uppercase()
            .replace(char::is_whitespace, "")
        })
        .unwrap_or_default()
    },
  };
  // Build the lookup key
  let name = if dialect_str.is_empty() {
    s!("LST@LANGUAGE@{lang}")
  } else {
    s!("LST@LANGUAGE@{lang}${dialect_str}")
  };
  // Try to find the language definition, also try without dialect
  let stored = state::lookup_value(&name).or_else(|| {
    if !dialect_str.is_empty() {
      state::lookup_value(&s!("LST@LANGUAGE@{lang}"))
    } else {
      None
    }
  });
  if let Some(stored) = stored {
    match stored {
      Stored::KeyVals(kv) => {
        // Stored as KeyVals — call lst_activate directly (matches Perl: lstActivate($values))
        lst_activate(Some(&kv));
      },
      Stored::Tokens(kv_tokens) => {
        // Stored as tokens — wrap in \lstset{...} and digest
        let _ = stomach::digest(Tokens::new(
          vec![T_CS!("\\lstset")]
            .into_iter()
            .chain(std::iter::once(T_BEGIN!()))
            .chain(kv_tokens.unlist().iter().cloned())
            .chain(std::iter::once(T_END!()))
            .collect(),
        ));
      },
      _ => {},
    }
  }
}
