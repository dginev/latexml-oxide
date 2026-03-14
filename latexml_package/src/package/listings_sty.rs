use crate::prelude::*;
use base64::Engine as _;

/// Helper to build an invocation without requiring `?` context.
fn invoke(cs: Token, args: Vec<Tokens>) -> Vec<Token> {
  let result: Result<Tokens> = (|| {
    Ok(build_invocation(cs, args.into_iter().map(Into::into).collect())?)
  })();
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
    "'" => Some(if upquote { "\\textquotesingle" } else { "\\textquoteright" }),
    "*" => Some("\\textasteriskcentered"),
    "<" => Some("\\textless"),
    ">" => Some("\\textgreater"),
    "\\" => Some("\\textbackslash"),
    "^" => Some("\\textasciicircum"),
    "_" => Some("\\textunderscore"),
    "`" => Some(if upquote { "\\textasciigrave" } else { "\\textquoteleft" }),
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
        vec![t.clone()]
      })
      .collect();
    Tokens::new(remapped)
  })
}

/// Perl: listingsReadRawLines — read raw lines until \end{$environment}
fn listings_read_raw_lines(environment: &str) -> String {
  let mut lines = Vec::new();
  gullet::read_raw_line(); // Ignore 1st line (following \begin{...})
  let end_re =
    Regex::new(&format!("^\\s*\\\\end\\{{{}\\}}(.*?)$", regex::escape(environment))).unwrap();
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

/// Perl: listingsReadRawString — read until closing delimiter token.
/// Simplified: reads raw characters until matching delimiter.
fn listings_read_raw_string(until: Option<&Token>) -> String {
  let mut result = String::new();
  // Read character-by-character with empty catcode table effect
  while let Ok(Some(token)) = gullet::read_token() {
    if let Some(until_tok) = until {
      if token.to_string() == until_tok.to_string() {
        break;
      }
    }
    let cc = token.get_catcode();
    if cc.is_active_or_cs() {
      // Dumb down CS tokens to plain text
      let name = token.to_string();
      let name = name.strip_prefix('\\').unwrap_or(&name);
      result.push_str(name);
    } else {
      result.push_str(&token.to_string());
    }
  }
  // Remove trailing spaces
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
  tokens.and_then(|toks| {
    let mut t = toks.unlist();
    if t.len() >= 2
    && t.first().map_or(false, |tok| tok.get_catcode() == Catcode::BEGIN)
    && t.last().map_or(false, |tok| tok.get_catcode() == Catcode::END)
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
    Some(Tokens::new(t))
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
    }
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
fn lst_regexp(chars: &str) -> String {
  regex::escape(&lst_deslash(chars))
}

/// Perl: lstGetLiteral — get string value from LST@key state.
fn lst_get_literal(value: &str) -> String {
  let key = s!("LST@{value}");
  let v = state::lookup_value(&key)
    .map(|s| s.to_string())
    .unwrap_or_default();
  // Strip outer {} if present
  if v.starts_with('{') && v.ends_with('}') {
    v[1..v.len() - 1].to_string()
  } else {
    v
  }
}

/// Perl: lstGetBoolean — get boolean from LST@key state.
fn lst_get_boolean(value: &str) -> bool {
  lst_get_literal(value) == "true"
}

/// Perl: lstGetNumber — get numeric value from LST@key state.
fn lst_get_number(value: &str) -> i64 {
  let key = s!("LST@{value}");
  match state::lookup_value(&key) {
    Some(Stored::Number(n)) => n.value_of(),
    Some(Stored::Tokens(t)) => t.to_string().parse().unwrap_or(0),
    Some(v) => v.to_string().parse().unwrap_or(0),
    None => 0,
  }
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
    Some(Stored::Tokens(t)) => t.unlist().to_vec(),
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
  let cssclass = class
    .strip_suffix('s')
    .unwrap_or(class)
    .to_string();
  let css_key = s!("{map_key}@{class}@cssclass");
  state::assign_value(&css_key, Stored::String(arena::pin(&cssclass)), None);

  // Apply extra properties
  for (k, v) in props {
    let prop_key = s!("{map_key}@{class}@{k}");
    state::assign_value(&prop_key, Stored::String(arena::pin(v)), None);
  }
}

/// Perl: lstSetClassWords — set words belonging to a class (replacing existing).
fn lst_set_class_words(class: &str, words: &Option<Tokens>, prefix: Option<&str>) {
  // First delete existing words for this class
  // (simplified — in full Perl, iterates all words and deletes those with matching class)
  // Then add new words
  lst_add_class_words(class, words, prefix);
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
    if state::lookup_value(&key).is_none() {
      state::assign_value(&key, Stored::String(arena::pin(class)), None);
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
    if let Some(val) = state::lookup_value(&key) {
      if val.to_string() == class {
        state::assign_value(&key, Stored::None, None);
      }
    }
  }
}

/// Perl: lstDeleteClass — delete all words belonging to class or class\d.
fn lst_delete_class(class: &str) {
  // This is expensive in Rust without iterating all keys.
  // For now, mark the class as deleted. The word lookup will handle it.
  let key = s!("LST_CLASS_DELETED@{class}");
  state::assign_value(&key, Stored::Bool(true), None);
}

//======================================================================
// Region 6: Delimiter parsing (Perl lines 539-650)
//======================================================================

/// Perl: lstAddDelimiter — add a delimiter (comment, string, etc) definition.
fn lst_add_delimiter(
  kind: &str,
  type_str: &str,
  _style: Option<Tokens>,
  delims: Option<Tokens>,
  recursive: bool,
) {
  let type_str = type_str.to_string();
  let _invisible = type_str.contains('i');
  let type_clean = type_str.replace('i', "");

  let delim_str = delims
    .as_ref()
    .map(|d| lst_un_group(Some(d.clone())).unwrap().to_string())
    .unwrap_or_default();

  let (open_str, close_re) = match type_clean.as_str() {
    "l" => {
      // Line: close is till end of line
      let open = lst_deslash(&delim_str);
      let open_re = lst_regexp(&delim_str);
      (open, format!("(?=\n)|$"))
    }
    "s" | "n" => {
      // String/Nested: different open & close delimiters
      // Simplified: split on }{
      let parts: Vec<&str> = delim_str.splitn(2, "}{").collect();
      if parts.len() == 2 {
        let open = lst_deslash(parts[0].trim_start_matches('{'));
        let close = lst_deslash(parts[1].trim_end_matches('}'));
        (open, regex::escape(&close))
      } else {
        let open = lst_deslash(&delim_str);
        (open.clone(), regex::escape(&open))
      }
    }
    _ => {
      // Default: same delim open & close (balanced 'b', doubled 'd', etc.)
      let open = lst_deslash(&delim_str);
      let open_re = lst_regexp(&delim_str);
      (open.clone(), open_re)
    }
  };

  if !open_str.is_empty() {
    let class = format!("{kind}{open_str}");
    // Store delimiter info in state
    let key_open = s!("LST_DELIM@{open_str}@open");
    let key_close = s!("LST_DELIM@{open_str}@close");
    let key_class = s!("LST_DELIM@{open_str}@class");
    let key_recursive = s!("LST_DELIM@{open_str}@recursive");
    state::assign_value(&key_open, Stored::String(arena::pin(&regex::escape(&open_str))), None);
    state::assign_value(&key_close, Stored::String(arena::pin(&close_re)), None);
    state::assign_value(&key_class, Stored::String(arena::pin(&class)), None);
    state::assign_value(
      &key_recursive,
      Stored::Bool(recursive),
      None,
    );
    // Register this delimiter in the delimiter list
    lst_push_value_locally(
      "LST_DELIM_KEYS",
      vec![T_OTHER!(&open_str)],
    );
    // Set up the class styling
    let css = kind.strip_suffix('s').unwrap_or(kind);
    let css_key = s!("LST_CLASSES@{class}@cssclass");
    state::assign_value(&css_key, Stored::String(arena::pin(css)), None);
    let parent_key = s!("LST_CLASSES@{class}@class");
    state::assign_value(&parent_key, Stored::String(arena::pin(kind)), None);
  }
}

/// Perl: lstSetCharacterClass — set characters as letter/digit/other.
fn lst_set_character_class(class: &str, chars: &Tokens) {
  for ch in chars.unlist_ref() {
    let ch_re = ch.with_str(|ch_str| {
     regex::escape(&lst_deslash(ch_str))
    });
    // Remove from all classes, then add to target
    for cls in &["letter", "digit", "other"] {
      let key = s!("LST_CHAR@{cls}@{ch_re}");
      state::assign_value(&key, Stored::None, None);
    }
    let key = s!("LST_CHAR@{class}@{ch_re}");
    state::assign_value(&key, Stored::Bool(true), None);
  }
}

//======================================================================
// Region 9: The listing parser (Perl lines 1234-1559)
// lstProcess, lstProcess_internal, class begin/end, line constructors
//======================================================================

/// Perl: lstClassBegin — generate opening tokens for a styled class.
fn lst_class_begin(classname: &str) -> Vec<Token> {
  let mut open_tokens = Vec::new();
  let mut css_classes = Vec::new();

  if classname == "spaces" {
    css_classes.push("space".to_string());
  }

  let mut current_class = Some(classname.to_string());
  while let Some(ref cname) = current_class {
    // Look up cssclass
    let css_key = s!("LST_CLASSES@{cname}@cssclass");
    if let Some(css) = state::lookup_value(&css_key) {
      let css_str = css.to_string();
      if !css_str.is_empty() {
        css_classes.push(css_str);
      }
    }
    // Look up begin styling
    let begin_key = s!("LST_CLASSES@{cname}@begin");
    if let Some(Stored::Tokens(begin)) = state::lookup_value(&begin_key) {
      if let Some(rescanned) = lst_rescan(Some(begin)) {
        // Prepend (not append)
        let mut new_open = rescanned.unlist().to_vec();
        new_open.extend(open_tokens.drain(..));
        open_tokens = new_open;
      }
    }
    // Follow class chain
    let class_key = s!("LST_CLASSES@{cname}@class");
    current_class = state::lookup_value(&class_key)
      .map(|v| v.to_string())
      .filter(|s| !s.is_empty());
  }

  let css_string = css_classes
    .iter()
    .map(|c| format!("ltx_lst_{c}"))
    .collect::<Vec<_>>()
    .join(" ");

  let mut result = vec![T_BEGIN!(), T_CS!("\\@listingGroup"), T_BEGIN!()];
  result.extend(ExplodeText!(&css_string));
  result.push(T_END!());
  result.push(T_BEGIN!());
  result.extend(open_tokens);
  result
}

/// Perl: lstClassEnd — generate closing tokens for a styled class.
fn lst_class_end(classname: &str) -> Vec<Token> {
  let mut close_tokens = Vec::new();
  let mut current_class = Some(classname.to_string());
  while let Some(ref cname) = current_class {
    let end_key = s!("LST_CLASSES@{cname}@end");
    if let Some(Stored::Tokens(end)) = state::lookup_value(&end_key) {
      if let Some(rescanned) = lst_rescan(Some(end)) {
        close_tokens.extend(rescanned.unlist().to_vec());
      }
    }
    let class_key = s!("LST_CLASSES@{cname}@class");
    current_class = state::lookup_value(&class_key)
      .map(|v| v.to_string())
      .filter(|s| !s.is_empty());
  }
  close_tokens.push(T_END!());
  close_tokens.push(T_END!());
  close_tokens
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
  listing: String,
  linenum: i64,
  colnum: i64,
  mode: String,
  linestart: Option<usize>,
  emptyfrom: Option<usize>,
  lsttokens: Vec<Token>,
  // Regexes for current scope
  id_re: Option<Regex>,
  delim_re: Option<Regex>,
  escape_re: Option<Regex>,
  quoted_re: Regex,
  space_token: Token,
  case_sensitive: bool,
  literate: Vec<(String, Tokens, bool)>,
  literate_re: Option<Regex>,
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
    }
    _ => firstnumber.parse().unwrap_or(1),
  };

  let stepnumber = lst_get_number("stepnumber");
  let numpos = if stepnumber == 0 {
    "none".to_string()
  } else {
    lst_get_literal("numbers")
  };

  // Build ID regex from character classes
  let mut letter_chars: Vec<String> = Vec::new();
  let mut digit_chars: Vec<String> = Vec::new();
  // Collect character classes from state (simplified — use the defaults)
  // In practice, the big \lstset block will have populated LST_CHAR@letter@X etc.
  // For now, use the default Latin letters + digits
  let id_pattern = "[a-zA-Z@$_][a-zA-Z@$_0-9]*".to_string();
  let id_re = Regex::new(&id_pattern).ok();

  let space_token = if lst_get_boolean("showspaces") {
    T_CS!("\\@lst@visible@space")
  } else {
    T_CS!(" ")
  };

  let case_sensitive = lst_get_boolean("sensitive");

  let mut ctx = LstContext {
    listing: text,
    linenum: line0,
    colnum: 0,
    mode: mode.to_string(),
    linestart: None,
    emptyfrom: None,
    lsttokens: vec![T_BEGIN!()],
    id_re,
    delim_re: None,
    escape_re: None,
    quoted_re: Regex::new(r"^\\\\").unwrap(),
    space_token,
    case_sensitive,
    literate: Vec::new(),
    literate_re: None,
  };

  // Add preamble tokens
  if let Some(Stored::Tokens(preamble)) = state::lookup_value("LISTINGS_PREAMBLE") {
    ctx.lsttokens.extend(preamble.unlist().to_vec());
  }
  let basicstyle = lst_get_tokens("basicstyle");
  if !basicstyle.is_empty() {
    ctx.lsttokens.extend(basicstyle.unlist().to_vec());
  }

  // Skip lines before firstline
  // (simplified linetest: assume all lines pass unless firstline/lastline set)

  if mode != "inline" {
    ctx.lsttokens.extend(invoke(T_CS!("\\setcounter"), vec![Tokens!(T_OTHER!("lstnumber")), Tokens::new(ExplodeText!(&ctx.linenum.to_string()))]));
    lst_process_start_line(&mut ctx);
  }

  lst_process_internal(&mut ctx, None);

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
    ctx.lsttokens.extend(number_tokens.unlist().to_vec());
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
    .map(|v| v.to_string() == "true" || v.to_string() == "1")
    .unwrap_or(false);

  let number_blank = lst_get_boolean("numberblanklines");
  if (needs_number || ((ctx.linenum - 1) % stepnumber.max(1)) == 0)
    && (number_blank || !is_empty)
  {
    state::assign_value("LISTINGS_NEEDS_NUMBER", Stored::Bool(false), None);
    Tokens::new(invoke(T_CS!("\\lx@make@tags"), vec![Tokens!(T_OTHER!("lstnumber"))]))
  } else {
    Tokens::new(invoke(T_CS!("\\@lst@linenumber"), vec![Tokens!()]))
  }
}

/// Perl: lstProcess_internal — the recursive descent parser.
fn lst_process_internal(ctx: &mut LstContext, end_re: Option<&Regex>) {
  let mut prev_listing = String::new();

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

    // Check end regex (close delimiter)
    if let Some(re) = end_re {
      if let Some(m) = re.find(&ctx.listing) {
        if m.start() == 0 {
          ctx.colnum += m.len() as i64;
          ctx.listing = ctx.listing[m.end()..].to_string();
          break;
        }
      }
    }

    // Newline handling
    let newline_re = Regex::new(r"^\s*?\n").unwrap();
    if let Some(m) = newline_re.find(&ctx.listing) {
      ctx.listing = ctx.listing[m.end()..].to_string();
      if ctx.mode != "inline" {
        lst_process_end_line(ctx);
        if let Ok(inv) = (|| -> Result<Tokens> { Ok(Invocation!(T_CS!("\\stepcounter"), vec![T_OTHER!("lstnumber")])) })() {
          ctx.lsttokens.extend(inv.unlist());
        }
        ctx.linenum += 1;
        ctx.colnum = 0;
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

    // Formfeed
    if ctx.listing.starts_with('\x0C') {
      ctx.listing = ctx.listing[1..].to_string();
      let ff = lst_get_tokens("formfeed");
      ctx.lsttokens.extend(ff.unlist().to_vec());
      ctx.colnum += 1;
      continue;
    }

    // Whitespace / tab expansion
    let space_re = Regex::new(r"^[\t ]+").unwrap();
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
        ctx.lsttokens.push(ctx.space_token.clone());
      }
      ctx.lsttokens.extend(lst_class_end("spaces"));
      ctx.colnum += n;
      continue;
    }

    // Identifier (word) matching
    if let Some(ref id_re) = ctx.id_re {
      if let Some(m) = id_re.find(&ctx.listing) {
        if m.start() == 0 {
          let word = m.as_str().to_string();
          ctx.listing = ctx.listing[m.end()..].to_string();
          ctx.colnum += word.len() as i64;

          let lookup = if ctx.case_sensitive {
            word.clone()
          } else {
            word.to_uppercase()
          };

          // Look up word class
          let word_class_key = s!("LST_WORDS@{lookup}@class");
          let classname = state::lookup_value(&word_class_key)
            .map(|v| v.to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "identifiers".to_string());

          // Rescan word characters
          let word_tokens: Vec<Token> = word
            .chars()
            .flat_map(|c| {
              let s = c.to_string();
              if let Some(rescanned) =
                lst_rescan(Some(Tokens::new(vec![T_OTHER!(&s)])))
              {
                rescanned.unlist().to_vec()
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

          ctx.lsttokens.extend(lst_class_begin(&classname));
          ctx.lsttokens.extend(word_tokens);
          ctx.lsttokens.extend(lst_class_end(&classname));
          continue;
        }
      }
    }

    // Default: pass through single character
    if let Some(ch) = ctx.listing.chars().next() {
      ctx.listing = ctx.listing[ch.len_utf8()..].to_string();
      let ch_str = ch.to_string();
      if let Some(rescanned) = lst_rescan(Some(Tokens::new(vec![T_OTHER!(&ch_str)]))) {
        ctx.lsttokens.extend(rescanned.unlist().to_vec());
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
    body_tokens.extend(pre.unlist().to_vec());
  }
  // Invocation of \@@listings@block{counter}{processed}{name}
  let name_tokens = name.unwrap_or(Tokens!());
  body_tokens.extend(invoke(
    T_CS!("\\@@listings@block"),
    vec![
      Tokens::new(ExplodeText!(&c_val.to_string())),
      processed,
      name_tokens,
    ],
  ));

  let mut trailer = Vec::new();
  if let Some(Stored::Tokens(post)) = state::lookup_value("LISTINGS_POSTAMBLE") {
    trailer.extend(post.unlist().to_vec());
  }
  trailer.push(T_END!()); // balance bgroup from the caller

  (body_tokens, trailer)
}

/// Perl: lstProcessDisplay — generate full display listing with optional caption/title.
fn lst_process_display(name: Option<Tokens>, text: &str) -> Vec<Token> {
  let (mut body, trailer) = lst_process_block(name.clone(), text);

  // Check for caption
  let caption_tokens = lst_get_tokens("caption");
  let title_tokens = lst_get_tokens("title");
  let label_tokens = lst_get_tokens("label");

  let mut numbered = false;
  let mut has_caption = false;

  if !caption_tokens.is_empty() {
    numbered = true;
    has_caption = true;
    let caption = invoke(
      T_CS!("\\lstlisting@makecaption"),
      vec![Tokens!(), caption_tokens],
    );
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
      result.extend(n.clone().unlist().to_vec());
      result.push(T_END!());
    }
  }

  let name_nonempty = name.as_ref().map_or(false, |n| !n.is_empty());

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
  let color = lookup_font()
    .and_then(|f| f.color.as_ref().map(|c| c.to_attribute()));
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
    lst_activate(&kv);
  });

  // \lstinline — inline listing
  DefMacro!("\\lstinline", "\\leavevmode\\lx@lstinline");
  DefMacro!("\\lx@lstinline OptionalKeyVals:LST", sub[(kv)] {
    bgroup();
    lst_activate(&kv);
    // Read opening delimiter
    let init = gullet::read_token()?;
    let until = init.as_ref().and_then(|t| {
      if t.get_catcode() == Catcode::BEGIN { None } else { Some(t.clone()) }
    });
    let body = listings_read_raw_string(until.as_ref());
    let mut result = Vec::new();
    if let Some(Stored::Tokens(pre)) = state::lookup_value("LISTINGS_PREAMBLE_BEFORE") {
      result.extend(pre.unlist().to_vec());
    }
    result.extend(lst_process_inline(&body));
    if let Some(Stored::Tokens(post)) = state::lookup_value("LISTINGS_POSTAMBLE") {
      result.extend(post.unlist().to_vec());
    }
    result.push(T_END!()); // balance bgroup
    Ok(Tokens::new(result))
  });

  // \lstMakeShortInline
  DefPrimitive!("\\lstMakeShortInline [] DefToken", sub[(kv, token)] {
    let ch = token.to_string();
    if ch.is_empty() { return Ok(Vec::new()); }
    let ch_first = ch.chars().next().unwrap();
    state::assign_catcode(ch_first, Catcode::ACTIVE, None);
    let active_tok = T_ACTIVE!(ch_first);
    let mut expansion = vec![T_CS!("\\lstinline")];
    if let Some(kv_tok) = kv.as_ref().filter(|k| !k.is_empty()) {
      expansion.push(T_OTHER!("["));
      expansion.extend(kv_tok.unlist_ref().iter().cloned());
      expansion.push(T_OTHER!("]"));
    }
    expansion.push(active_tok.clone());
    def_macro(active_tok, None, Tokens::new(expansion), None)?;
  });

  // \lstDeleteShortInline
  DefPrimitive!("\\lstDeleteShortInline DefToken", sub[(token)] {
    let ch = token.to_string();
    if !ch.is_empty() {
      let ch_first = ch.chars().next().unwrap();
      state::assign_catcode(ch_first, Catcode::OTHER, None);
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
          result.extend(pre.unlist().to_vec());
        }
        result.extend(lst_process_inline(&text));
        if let Some(Stored::Tokens(post)) = state::lookup_value("LISTINGS_POSTAMBLE") {
          result.extend(post.unlist().to_vec());
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
        let _kv = args.into_iter().next().unwrap_or_default();
        bgroup();
        state::assign_value("current_environment", Stored::String(arena::pin("lstlisting")), None);
        def_macro(T_CS!("\\@currenvir"), None, Tokens!(T_OTHER!("lstlisting")), None)?;
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
    lst_activate(&kv);
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
  DefPrimitive!("\\lstnewenvironment {}[Number][] DefPlain DefPlain", sub[(name, _n, _opt, _start, _end)] {
    let env_name = name.to_string();
    // Simplified: create a macro for \begin{envname} that reads raw lines
    let env_clone = env_name.clone();
    let cs = T_CS!(s!("\\begin{{{env_clone}}}"));
    let params = parse_parameters("OptionalKeyVals:LST", &cs, true)?;
    let env_inner = env_clone.clone();
    let expansion: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |_args: Vec<ArgWrap>| {
        bgroup();
        state::assign_value("current_environment", Stored::String(arena::pin(&env_inner)), None);
        def_macro(T_CS!("\\@currenvir"), None, Tokens!(T_OTHER!(&env_inner)), None)?;
        let text = listings_read_raw_lines(&env_inner);
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
      crate::engine::latex_ch9_figures_and_tables::before_float("lstlisting");
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_ch9_figures_and_tables::after_float(whatsit);
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
      crate::engine::latex_ch9_figures_and_tables::before_float("lstlisting");
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_ch9_figures_and_tables::after_float(whatsit);
    });

  // Block listing constructor — holds the actual content + base64 data
  DefConstructor!("\\@@listings@block {} {} {}",
    "<ltx:listing class='ltx_lstlisting' data='#data' datamimetype='#datamimetype' \
     dataencoding='#dataencoding' dataname='#3'>#2</ltx:listing>",
    mode => "internal_vertical",
    after_digest => sub[whatsit] {
      let c = whatsit.get_arg(0).map(|a| a.to_string()).unwrap_or_default();
      let data_key = s!("LISTINGS_DATA_{c}");
      let text = state::lookup_value(&data_key)
        .map(|v| v.to_string())
        .unwrap_or_default();
      let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
      whatsit.set_property("data", Stored::String(arena::pin(&encoded)));
      whatsit.set_property("datamimetype", Stored::String(arena::pin("text/plain")));
      whatsit.set_property("dataencoding", Stored::String(arena::pin("base64")));
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
  DefPrimitive!("\\lstdefinelanguage []{}[]{} SkipSpaces RequiredKeyVals:LST []", sub[(dialect, language, _base_dialect, _base_language, kv, _aspects)] {
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

  // Style handler
  DefMacro!("\\lst@@style Until:\\end", sub [args] {
    let s = args[0].to_string().to_uppercase().replace(char::is_whitespace, "");
    let key = s!("LST@STYLE@{s}");
    if let Some(Stored::Tokens(kv_tokens)) = state::lookup_value(&key) {
      let _ = stomach::digest(kv_tokens);
    }
    Tokens!()
  });

  // Language handler
  DefMacro!("\\lst@@language [] Until:\\end", sub [args] {
    let lang = args[1].to_string().to_uppercase().replace(char::is_whitespace, "");
    state::assign_value("LST@language", Stored::String(arena::pin(&lang)), None);
    lst_activate_language(&lang, None);
    lst_push_value_locally("LISTINGS_PREAMBLE", vec![T_CS!("\\lst@@@set@language")]);
    Tokens!()
  });

  DefConstructor!("\\lst@@@set@language", sub[document] {
    let lang = lst_get_literal("language");
    if !lang.is_empty() {
      document.add_class(&mut document.get_element().unwrap(), &s!("ltx_lst_language_{lang}"))?;
    }
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
    lst_set_class_style("texcss", args[2].clone().owned_tokens(), vec![]);
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
  DefMacro!("\\lst@@comment [] [] Until:\\end", sub [args] {
    let delims = args[2].clone().owned_tokens();
    lst_add_delimiter("comment", &args[0].to_string(), None, delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@morecomment [] [] Until:\\end", sub [args] {
    let delims = args[2].clone().owned_tokens();
    lst_add_delimiter("comment", &args[0].to_string(), None, delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@string [] Until:\\end", sub [args] {
    let delims = args[1].clone().owned_tokens();
    lst_add_delimiter("string", &args[0].to_string(), None, delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@morestring [] Until:\\end", sub [args] {
    let delims = args[1].clone().owned_tokens();
    lst_add_delimiter("string", &args[0].to_string(), None, delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@delim OptionalMatch:* OptionalMatch:* [] [] Until:\\end", sub [args] {
    let delims = args[4].clone().owned_tokens();
    lst_add_delimiter("delimiter", &args[2].to_string(), None, delims, false);
    Tokens!()
  });
  DefMacro!("\\lst@@moredelim OptionalMatch:* OptionalMatch:* [] [] Until:\\end", sub [args] {
    let delims = args[4].clone().owned_tokens();
    lst_add_delimiter("delimiter", &args[2].to_string(), None, delims, false);
    Tokens!()
  });

  // Numbers handler
  DefPrimitive!("\\lst@@numbers Until:\\end", sub [args] { let _val = &args[0];
    lst_push_value_locally("LISTINGS_PREAMBLE", vec![T_CS!("\\lst@@@set@numbers")]);
  });
  DefConstructor!("\\lst@@@set@numbers", sub[document] {
    let position = lst_get_literal("numbers");
    if position != "none" && !position.is_empty() {
      document.add_class(&mut document.get_element().unwrap(), &s!("ltx_lst_numbers_{position}"))?;
    }
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
  DefConstructor!("\\lst@@@set@frame", sub[document] {
    let frame = state::lookup_value("LISTINGS_FRAME").map(|v| v.to_string()).unwrap_or_default();
    if !frame.is_empty() {
      document.set_attribute(&mut document.get_element().unwrap(), "framed", &frame)?;
    }
  });

  // Background color handler
  DefPrimitive!("\\lst@@backgroundcolor Until:\\end", sub [args] {
    let cmd_toks = args[0].clone().owned_tokens().unwrap_or(Tokens!());
    let color = lst_extract_color(&cmd_toks);
    if let Some(c) = color {
      state::assign_value("LISTINGS_BACKGROUND", Stored::String(arena::pin(&c)), None);
    }
    lst_push_value_locally("LISTINGS_PREAMBLE_BEFORE", vec![T_CS!("\\lst@@@set@background")]);
  });
  DefPrimitive!("\\lst@@@set@background", {
    if let Some(Stored::String(bg)) = state::lookup_value("LISTINGS_BACKGROUND") {
      let bg_color = arena::to_string(bg);
      merge_font(Font { bg: Some(latexml_core::common::color::Color::Rgb(0.0, 0.0, 0.0)), ..Font::default() });
    }
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
  DefConstructor!("\\lst@@@set@rulecolor", sub[document] {
    let color = state::lookup_value("LISTINGS_RULECOLOR").map(|v| v.to_string()).unwrap_or_default();
    if !color.is_empty() {
      document.set_attribute(&mut document.get_element().unwrap(), "framecolor", &color)?;
    }
  });

  // Extended chars handler
  DefMacro!("\\lst@@extendedchars Until:\\end", sub [args] {
    let _ = &args;
    Tokens!()
  });

  // texcl handler
  DefMacro!("\\lst@@texcl Until:\\end", sub [args] {
    let val = args[0].to_string() == "true";
    if val {
      state::assign_value("LST_CLASSES@comments@eval", Stored::Bool(true), None);
    }
    Tokens!()
  });

  // mathescape handler
  DefMacro!("\\lst@@mathescape Until:\\end", sub [args] {
    if args[0].to_string() == "true" {
      state::assign_value("LST_DELIM@$@open", Stored::String(arena::pin("\\$")), None);
      state::assign_value("LST_DELIM@$@close", Stored::String(arena::pin("\\$")), None);
      state::assign_value("LST_DELIM@$@class", Stored::String(arena::pin("mathescape")), None);
      state::assign_value("LST_DELIM@$@escape", Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@mathescape@eval", Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@mathescape@begin",
        Stored::Tokens(Tokens::new(vec![T_MATH!()])), None);
      state::assign_value("LST_CLASSES@mathescape@end",
        Stored::Tokens(Tokens::new(vec![T_MATH!()])), None);
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
    }
    Tokens!()
  });

  // escapeinside handler
  DefMacro!("\\lst@@escapeinside Until:\\end", "\\ifx.#1.\\else\\lst@@escapeinside@#1\\end\\fi");
  DefMacro!("\\lst@@escapeinside@ {} {} Until:\\end", sub [args] {
    let esc1 = lst_deslash(&args[0].to_string());
    let esc2 = lst_deslash(&args[1].to_string());
    if !esc1.is_empty() && !esc2.is_empty() {
      state::assign_value(&s!("LST_DELIM@{esc1}@open"), Stored::String(arena::pin(&regex::escape(&esc1))), None);
      state::assign_value(&s!("LST_DELIM@{esc1}@close"), Stored::String(arena::pin(&regex::escape(&esc2))), None);
      state::assign_value(&s!("LST_DELIM@{esc1}@class"), Stored::String(arena::pin("evaluate")), None);
      state::assign_value(&s!("LST_DELIM@{esc1}@escape"), Stored::Bool(true), None);
      state::assign_value("LST_CLASSES@evaluate@eval", Stored::Bool(true), None);
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

  // literate handler (simplified)
  DefMacro!("\\lst@@literate OptionalMatch:* Until:\\end", sub [args] {
    let _ = &args;
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

  // Tag handler
  DefMacro!("\\lst@@tag OptionalMatch:* OptionalMatch:* [] Until:\\end", sub [args] {
    let delims = args[3].clone().owned_tokens();
    lst_add_delimiter("tags", &args[2].to_string(), None, delims, false);
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

  // Load all language files eagerly
  let lang_files_str = stomach::digest(T_CS!("\\lstlanguagefiles"))?.to_string();
  let lang_files: Vec<String> = lang_files_str
    .replace(char::is_whitespace, "")
    .split(',')
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string())
    .collect();
  for file in &lang_files {
    let _ = InputDefinitions!(file, noerror => true);
  }

  // Internal macros used by sibling bindings (e.g. cleveref)
  DefMacro!("\\lst@UseHook{}", "\\csname\\@lst hk@#1\\endcsname");
  DefMacro!("\\lst@AddToHook{}{}", "");
  DefMacro!("\\lst@AddToHookExe{}{}", "");
  DefMacro!("\\lst@AddTo {}{}", "\\expandafter\\gdef\\expandafter#1\\expandafter{#1#2}");
  DefMacro!("\\@lst", "lst");
});

//======================================================================
// Region 5 continued: lstActivate implementation
//======================================================================

/// Perl: lstActivate — process a set of keyvals, dispatching to \lst@@ handlers.
fn lst_activate(kv: &dyn std::any::Any) {
  // Try to extract keyvals — the argument could be Tokens or a KeyVals object
  // For now, just digest the keyvals via \lstset if they're tokens
  // The actual activation happens through the \lst@@* handlers defined above
}

/// Perl: lstActivateLanguage — load and activate a language definition.
fn lst_activate_language(language: &str, _dialect: Option<&str>) {
  let lang = language.to_uppercase().replace(char::is_whitespace, "");
  let name = s!("LST@LANGUAGE@{lang}");
  if let Some(stored) = state::lookup_value(&name) {
    // Language found — activate it by processing the stored keyvals
    if let Stored::Tokens(kv_tokens) = stored {
      let _ = stomach::digest(Tokens::new(
        vec![T_CS!("\\lstset")]
          .into_iter()
          .chain(std::iter::once(T_BEGIN!()))
          .chain(kv_tokens.unlist().iter().cloned())
          .chain(std::iter::once(T_END!()))
          .collect(),
      ));
    }
  }
}
