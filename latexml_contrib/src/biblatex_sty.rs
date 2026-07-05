use latexml_package::prelude::*;

// === biblatex .bbl entry-pipeline helpers ===
// Mirror Perl ar5iv-bindings/biblatex.sty.ltxml entry/endentry/name/list/field
// closures (L127-340). The .bbl file emits \entry{key}{type}{} … \endentry per
// reference; between them \field/\strng/\list/\name records metadata into the
// `biblatex_entry` HashStored, and \endentry flushes a `\bibitem[label]{key}
// authors. \newblock title. \newblock In: journal. year, pages.` token stream
// onto `rebuilt_bibtex_variant`. The list-end macros (\enddatalist etc.) wrap
// the accumulated variant in `\thebibliography{count}…\endthebibliography`.

fn bib_entry_get() -> SymHashMap<Stored> {
  match lookup_value("biblatex_entry") {
    Some(Stored::HashStored(map)) => map,
    _ => SymHashMap::default(),
  }
}

fn bib_entry_save(map: SymHashMap<Stored>) {
  assign_value(
    "biblatex_entry",
    Stored::HashStored(map),
    Some(Scope::Global),
  );
}

fn bib_entry_set_tokens(name: &str, val: Tokens) {
  let mut entry = bib_entry_get();
  entry.insert(name, Stored::Tokens(val));
  bib_entry_save(entry);
}

fn bib_entry_get_tokens(map: &SymHashMap<Stored>, name: &str) -> Option<Tokens> {
  map.get(name).and_then(|s| match s {
    Stored::Tokens(t) => Some(t.clone()),
    _ => None,
  })
}

fn bib_state_int(key: &str) -> i64 {
  match lookup_value(key) {
    Some(Stored::Int(n)) => n,
    _ => 0,
  }
}

fn bib_state_set_int(key: &str, value: i64) {
  assign_value(key, Stored::Int(value), Some(Scope::Global));
}

fn bib_variant_push(toks: Vec<Token>) {
  let mut acc: Vec<Token> = match lookup_value("rebuilt_bibtex_variant") {
    Some(Stored::Tokens(t)) => t.unlist(),
    _ => Vec::new(),
  };
  acc.extend(toks);
  assign_value(
    "rebuilt_bibtex_variant",
    Stored::Tokens(Tokens::new(acc)),
    Some(Scope::Global),
  );
}

fn bib_as_thebibliography() -> Tokens {
  let variant: Vec<Token> = match lookup_value("rebuilt_bibtex_variant") {
    Some(Stored::Tokens(t)) => t.unlist(),
    _ => return Tokens::default(),
  };
  if variant.is_empty() {
    return Tokens::default();
  }
  // Reset variant and entry-count so re-invocation is idempotent (matches
  // Perl L113-115).
  assign_value(
    "rebuilt_bibtex_variant",
    Stored::Tokens(Tokens::default()),
    Some(Scope::Global),
  );
  let count = bib_state_int("biblatex_entry_count");
  bib_state_set_int("biblatex_entry_count", 0);
  let preamble: Vec<Token> = match lookup_value("biblatex_preamble") {
    Some(Stored::Tokens(t)) => t.unlist(),
    _ => Vec::new(),
  };
  let mut result: Vec<Token> = Vec::with_capacity(variant.len() + 16);
  result.push(T_CS!("\\thebibliography"));
  result.push(T_BEGIN!());
  result.extend(preamble);
  result.extend(ExplodeText!(&count.to_string()));
  result.push(T_END!());
  result.extend(variant);
  result.push(T_CS!("\\endthebibliography"));
  Tokens::new(result)
}

/// Perl `$fullname =~ s/\\\w+|[}{]//g` (ar5iv biblatex.sty.ltxml L324):
/// strip leftover control sequences (`\bibinitperiod`, …) and braces from a
/// name fragment, then trim. `\w` is Perl-ASCII (`[A-Za-z0-9_]`) — a backslash
/// NOT followed by a word char (e.g. the `\"` of an accent) is preserved, as
/// in Perl.
fn bib_clean_name(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '\\' {
      if chars
        .peek()
        .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
      {
        while chars
          .peek()
          .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
        {
          chars.next();
        }
      } else {
        out.push(ch);
      }
    } else if ch != '{' && ch != '}' {
      out.push(ch);
    }
  }
  out.trim().to_string()
}

/// Parse a biblatex keyval name block — the inner sub-group of a modern
/// (biber, bbl format ≥ 3.x) `\name` author record, e.g.
/// `family={Turtayev},familyi={T\bibinitperiod},given={Rustem},giveni=…,givenun=0`.
/// Splits on depth-0 commas (commas inside `{…}` don't split), then for each
/// `key=value` strips one layer of surrounding braces off the value. Mirrors
/// the outcome of Perl's `LaTeXML::Core::KeyVals->readFrom` (L302-306) without
/// the full KeyVals machinery — we only need `given`/`family` (and their
/// `i`-initial fallbacks).
fn parse_name_keyvals(s: &str) -> Vec<(String, String)> {
  let mut pairs: Vec<(String, String)> = Vec::new();
  let mut depth = 0i32;
  let mut cur = String::new();
  let flush = |seg: &str, pairs: &mut Vec<(String, String)>| {
    let seg = seg.trim();
    if seg.is_empty() {
      return;
    }
    if let Some(eq) = seg.find('=') {
      let key = seg[..eq].trim().to_string();
      let mut val = seg[eq + 1..].trim();
      if val.starts_with('{') && val.ends_with('}') && val.len() >= 2 {
        val = &val[1..val.len() - 1];
      }
      pairs.push((key, val.to_string()));
    }
  };
  for ch in s.chars() {
    match ch {
      '{' => {
        depth += 1;
        cur.push(ch);
      },
      '}' => {
        depth -= 1;
        cur.push(ch);
      },
      ',' if depth == 0 => {
        flush(&cur, &mut pairs);
        cur.clear();
      },
      _ => cur.push(ch),
    }
  }
  flush(&cur, &mut pairs);
  pairs
}

/// Perl L232-237 / L253-258: strip leading SPACE/`{` and trailing SPACE/`}`
/// tokens from a captured DOI/eprint field value before splicing it into an
/// `\href{…}` target, so a `\field{doi}{ {10.x/y} }`-style value yields a
/// clean URI.
fn bib_trim_url_tokens(toks: Tokens) -> Vec<Token> {
  let mut v = toks.unlist();
  let mut start = 0usize;
  while start < v.len() {
    let t = &v[start];
    if t.code == Catcode::SPACE || t.with_str(|s| s == "{") {
      start += 1;
    } else {
      break;
    }
  }
  let mut end = v.len();
  while end > start {
    let t = &v[end - 1];
    if t.code == Catcode::SPACE || t.with_str(|s| s == "}") {
      end -= 1;
    } else {
      break;
    }
  }
  v.truncate(end);
  v.drain(..start);
  v
}

// === biblatex author-year citation machinery ===
// Mirror of ar5iv-bindings biblatex.sty.ltxml after PRs #20/#21 + the
// 0911aec repair pass: three citation families (parenthetical / textual /
// bare) built on \@@cite/\@@bibref/\@@citephrase exactly like the natbib
// binding, greedy multicite readers, and a plain-\cite fallback for
// non-author-year styles.

/// Perl `_is_authoryear` — the style handler below is the only writer of
/// CITE_STYLE in a biblatex session.
fn blx_is_authoryear() -> bool { lookup_string("CITE_STYLE") == "authoryear" }

/// Perl `_set_biblatex_style`: detect author-year styles (apa, authoryear,
/// authoryear-comp, ...) and only then activate the author-year citation
/// commands + punctuation. Numeric/alphabetic documents keep the core
/// numeric `[ ]` defaults and plain-\cite behavior.
fn blx_set_style(style: &str) {
  if style.contains("authoryear") || style.contains("apa") {
    assign_value("CITE_STYLE", pin("authoryear"), Some(Scope::Global));
    assign_value(
      "CITE_OPEN",
      Stored::Token(T_OTHER!("(")),
      Some(Scope::Global),
    );
    assign_value(
      "CITE_CLOSE",
      Stored::Token(T_OTHER!(")")),
      Some(Scope::Global),
    );
    assign_value(
      "CITE_SEPARATOR",
      Stored::Token(T_OTHER!(";")),
      Some(Scope::Global),
    );
    assign_value(
      "CITE_NOTE_SEPARATOR",
      Stored::Token(T_OTHER!(",")),
      Some(Scope::Global),
    );
    assign_value(
      "CITE_AY_SEPARATOR",
      Stored::Token(T_OTHER!(",")),
      Some(Scope::Global),
    );
  }
}

fn blx_open() -> Tokens { lookup_tokens("CITE_OPEN").unwrap_or_default() }
fn blx_close() -> Tokens { lookup_tokens("CITE_CLOSE").unwrap_or_default() }
fn blx_ns() -> Tokens { lookup_tokens("CITE_NOTE_SEPARATOR").unwrap_or_default() }
fn blx_ay() -> Tokens { lookup_tokens("CITE_AY_SEPARATOR").unwrap_or_default() }

fn blx_nonempty(opt: &Option<Tokens>) -> bool { matches!(opt, Some(t) if !t.is_empty()) }

/// Perl convention (biblatex/natbib): one optional arg is a postnote, two
/// are prenote + postnote. Faithful nuance (arxiv-readability#10 /
/// ar5iv-bindings#4): `\parencite[see][]{key}` has a PRESENT-but-EMPTY
/// second optional — that must NOT swap ("see" stays the prenote); only an
/// ABSENT second optional makes the first one a postnote. Empties are
/// dropped after the swap decision.
fn blx_swap_pre_post(
  pre: Option<Tokens>,
  post: Option<Tokens>,
) -> (Option<Tokens>, Option<Tokens>) {
  let (pre, post) = if post.is_none() {
    (None, pre)
  } else {
    (pre, post)
  };
  (
    pre.filter(|t| !t.is_empty()),
    post.filter(|t| !t.is_empty()),
  )
}

/// Perl `_cite_fallback`: delegate to the saved core \cite for
/// non-author-year styles. Built by hand rather than via Invocation!: the
/// core \cite is robust, so its top-level binding is a parameterless
/// protect-wrapper and Invocation would revert ZERO arguments, silently
/// dropping both the postnote and the keys.
fn blx_cite_fallback(post: Option<Tokens>, keys: Tokens) -> Tokens {
  let mut toks = vec![T_CS!("\\blx@saved@cite")];
  if let Some(p) = post.filter(|t| !t.is_empty()) {
    toks.push(T_OTHER!("["));
    toks.extend(p.unlist());
    toks.push(T_OTHER!("]"));
  }
  toks.push(T_BEGIN!());
  toks.extend(keys.unlist());
  toks.push(T_END!());
  Tokens::new(toks)
}

/// Perl `_cite_parenthetical`: (prenote Author, Year, postnote) — used by
/// \parencite, \autocite, \citep.
fn blx_cite_parenthetical(
  star: bool,
  pre: Option<Tokens>,
  post: Option<Tokens>,
  keys: Tokens,
) -> Result<Tokens> {
  let (pre, post) = blx_swap_pre_post(pre, post);
  if !blx_is_authoryear() {
    return Ok(blx_cite_fallback(post, keys));
  }
  let author = if star { "FullAuthors" } else { "Authors" };
  let mut ay_space = blx_ay().unlist();
  ay_space.push(T_SPACE!());
  let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(ay_space)]);
  let bibref = Invocation!(T_CS!("\\@@bibref"), vec![
    Tokens::new(Explode!(s!("{author}Phrase1Year"))),
    keys,
    phrase1,
    Tokens!()
  ]);
  let mut body = blx_open().unlist();
  if let Some(p) = pre {
    body.extend(p.unlist());
    body.push(T_SPACE!());
  }
  body.extend(bibref.unlist());
  if let Some(p) = post {
    body.extend(blx_ns().unlist());
    body.push(T_SPACE!());
    body.extend(p.unlist());
  }
  body.extend(blx_close().unlist());
  Ok(Invocation!(T_CS!("\\@@cite"), vec![
    Tokens::new(Explode!("citep")),
    Tokens::new(body)
  ]))
}

/// Perl `_cite_textual`: prenote Author (Year, postnote) — used by
/// \textcite, \citet.
fn blx_cite_textual(
  star: bool,
  pre: Option<Tokens>,
  post: Option<Tokens>,
  keys: Tokens,
) -> Result<Tokens> {
  let (pre, post) = blx_swap_pre_post(pre, post);
  if !blx_is_authoryear() {
    return Ok(blx_cite_fallback(post, keys));
  }
  let author = if star { "FullAuthors" } else { "Authors" };
  let mut p1 = blx_open().unlist();
  if let Some(p) = pre {
    p1.extend(p.unlist());
    p1.push(T_SPACE!());
  }
  let mut p2 = Vec::new();
  if let Some(p) = post {
    p2.extend(blx_ns().unlist());
    p2.push(T_SPACE!());
    p2.extend(p.unlist());
  }
  p2.extend(blx_close().unlist());
  let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p1)]);
  let phrase2 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p2)]);
  let bibref = Invocation!(T_CS!("\\@@bibref"), vec![
    Tokens::new(Explode!(s!("{author} Phrase1YearPhrase2"))),
    keys,
    phrase1,
    phrase2
  ]);
  Ok(Invocation!(T_CS!("\\@@cite"), vec![
    Tokens::new(Explode!("citet")),
    bibref
  ]))
}

/// Perl `_cite_bare`: prenote Author, Year, postnote — no parentheses.
/// Used by \cite, \Cite, \citealt, \fullcite, \smartcite, \footcite, etc.
fn blx_cite_bare(
  star: bool,
  pre: Option<Tokens>,
  post: Option<Tokens>,
  keys: Tokens,
) -> Result<Tokens> {
  let (pre, post) = blx_swap_pre_post(pre, post);
  if !blx_is_authoryear() {
    return Ok(blx_cite_fallback(post, keys));
  }
  let author = if star { "FullAuthors" } else { "Authors" };
  let mut ay_space = blx_ay().unlist();
  ay_space.push(T_SPACE!());
  let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(ay_space)]);
  let bibref = Invocation!(T_CS!("\\@@bibref"), vec![
    Tokens::new(Explode!(s!("{author}Phrase1Year"))),
    keys,
    phrase1,
    Tokens!()
  ]);
  if pre.is_some() || post.is_some() {
    let mut body = Vec::new();
    if let Some(p) = pre {
      body.extend(p.unlist());
      body.push(T_SPACE!());
    }
    body.extend(bibref.unlist());
    if let Some(p) = post {
      body.extend(blx_ns().unlist());
      body.push(T_SPACE!());
      body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"), vec![
      Tokens::new(Explode!("cite")),
      Tokens::new(body)
    ]))
  } else {
    Ok(Invocation!(T_CS!("\\@@cite"), vec![
      Tokens::new(Explode!("cite")),
      bibref
    ]))
  }
}

/// One `[pre][post]{keys}` group of a biblatex multicite command.
type BlxCiteGroup = (Option<Tokens>, Option<Tokens>, Tokens);

/// Perl `_read_multicite_groups`: greedily read repeated
/// `[pre][post]{keys}` groups from the gullet; stop at the first token
/// that starts neither an optional nor a mandatory group.
fn blx_read_multicite_groups() -> Result<Vec<BlxCiteGroup>> {
  let mut groups: Vec<BlxCiteGroup> = Vec::new();
  loop {
    skip_spaces()?;
    let Some(next) = read_token()? else { break };
    if next.get_catcode() == Catcode::BEGIN {
      // `{keys}` with no optional args.
      unread_one(next);
      let keys = read_arg(ExpansionLevel::Off)?;
      groups.push((None, None, keys));
    } else if next.get_catcode() == Catcode::OTHER && next.with_str(|s| s == "[") {
      unread_one(next);
      let opt1 = read_optional(None)?;
      let opt2 = read_optional(None)?;
      let keys = read_arg(ExpansionLevel::Off)?;
      let (pre, post) = if opt2.is_some() {
        (opt1, opt2)
      } else {
        (None, opt1)
      };
      groups.push((
        pre.filter(|t| !t.is_empty()),
        post.filter(|t| !t.is_empty()),
        keys,
      ));
    } else {
      unread_one(next); // not ours — put it back
      break;
    }
  }
  Ok(groups)
}

/// Perl `_joined_keys`: comma-join all groups' keys for delegation to a
/// single \cite in the non-author-year fallback.
fn blx_joined_keys(groups: &[BlxCiteGroup]) -> Tokens {
  let mut toks: Vec<Token> = Vec::new();
  for (_, _, keys) in groups {
    if !toks.is_empty() {
      toks.push(T_OTHER!(","));
    }
    toks.extend(keys.clone().unlist());
  }
  Tokens::new(toks)
}

/// Join per-group token runs with "; " (Perl's multicite group separator).
fn blx_join_groups(parts: Vec<Vec<Token>>) -> Vec<Token> {
  let mut body: Vec<Token> = Vec::new();
  for (i, part) in parts.into_iter().enumerate() {
    if i > 0 {
      body.push(T_OTHER!(";"));
      body.push(T_SPACE!());
    }
    body.extend(part);
  }
  body
}

/// Perl `_multicite_parenthetical`: \parencites, \autocites — one pair of
/// parens around all "pre Author, Year, post" groups.
fn blx_multicite_parenthetical(star: bool) -> Result<Tokens> {
  let groups = blx_read_multicite_groups()?;
  if groups.is_empty() {
    return Ok(Tokens::default());
  }
  if !blx_is_authoryear() {
    let keys = blx_joined_keys(&groups);
    return Ok(blx_cite_fallback(None, keys));
  }
  let author = if star { "FullAuthors" } else { "Authors" };
  let mut parts: Vec<Vec<Token>> = Vec::new();
  for (pre, post, keys) in groups {
    let mut ay_space = blx_ay().unlist();
    ay_space.push(T_SPACE!());
    let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(ay_space)]);
    let bibref = Invocation!(T_CS!("\\@@bibref"), vec![
      Tokens::new(Explode!(s!("{author}Phrase1Year"))),
      keys,
      phrase1,
      Tokens!()
    ]);
    let mut toks = Vec::new();
    if let Some(p) = pre {
      toks.extend(p.unlist());
      toks.push(T_SPACE!());
    }
    toks.extend(bibref.unlist());
    if let Some(p) = post {
      toks.extend(blx_ns().unlist());
      toks.push(T_SPACE!());
      toks.extend(p.unlist());
    }
    parts.push(toks);
  }
  let mut body = blx_open().unlist();
  body.extend(blx_join_groups(parts));
  body.extend(blx_close().unlist());
  Ok(Invocation!(T_CS!("\\@@cite"), vec![
    Tokens::new(Explode!("citep")),
    Tokens::new(body)
  ]))
}

/// Perl `_multicite_textual`: \textcites — "Author (pre Year, post)" per
/// group, joined with "; ". Same phrase layout as `blx_cite_textual`.
fn blx_multicite_textual(star: bool) -> Result<Tokens> {
  let groups = blx_read_multicite_groups()?;
  if groups.is_empty() {
    return Ok(Tokens::default());
  }
  if !blx_is_authoryear() {
    let keys = blx_joined_keys(&groups);
    return Ok(blx_cite_fallback(None, keys));
  }
  let author = if star { "FullAuthors" } else { "Authors" };
  let mut parts: Vec<Vec<Token>> = Vec::new();
  for (pre, post, keys) in groups {
    let mut p1 = blx_open().unlist();
    if let Some(p) = pre {
      p1.extend(p.unlist());
      p1.push(T_SPACE!());
    }
    let mut p2 = Vec::new();
    if let Some(p) = post {
      p2.extend(blx_ns().unlist());
      p2.push(T_SPACE!());
      p2.extend(p.unlist());
    }
    p2.extend(blx_close().unlist());
    let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p1)]);
    let phrase2 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(p2)]);
    let bibref = Invocation!(T_CS!("\\@@bibref"), vec![
      Tokens::new(Explode!(s!("{author} Phrase1YearPhrase2"))),
      keys,
      phrase1,
      phrase2
    ]);
    parts.push(bibref.unlist());
  }
  Ok(Invocation!(T_CS!("\\@@cite"), vec![
    Tokens::new(Explode!("citet")),
    Tokens::new(blx_join_groups(parts))
  ]))
}

/// Perl `_multicite_bare`: \cites — "pre Author, Year, post" per group,
/// joined with "; ", no parens.
fn blx_multicite_bare(star: bool) -> Result<Tokens> {
  let groups = blx_read_multicite_groups()?;
  if groups.is_empty() {
    return Ok(Tokens::default());
  }
  if !blx_is_authoryear() {
    let keys = blx_joined_keys(&groups);
    return Ok(blx_cite_fallback(None, keys));
  }
  let author = if star { "FullAuthors" } else { "Authors" };
  let mut parts: Vec<Vec<Token>> = Vec::new();
  for (pre, post, keys) in groups {
    let mut ay_space = blx_ay().unlist();
    ay_space.push(T_SPACE!());
    let phrase1 = Invocation!(T_CS!("\\@@citephrase"), vec![Tokens::new(ay_space)]);
    let bibref = Invocation!(T_CS!("\\@@bibref"), vec![
      Tokens::new(Explode!(s!("{author}Phrase1Year"))),
      keys,
      phrase1,
      Tokens!()
    ]);
    let mut toks = Vec::new();
    if let Some(p) = pre {
      toks.extend(p.unlist());
      toks.push(T_SPACE!());
    }
    toks.extend(bibref.unlist());
    if let Some(p) = post {
      toks.extend(blx_ns().unlist());
      toks.push(T_SPACE!());
      toks.extend(p.unlist());
    }
    parts.push(toks);
  }
  Ok(Invocation!(T_CS!("\\@@cite"), vec![
    Tokens::new(Explode!("cite")),
    Tokens::new(blx_join_groups(parts))
  ]))
}

/// Destructure the shared `OptionalMatch:* [][] Semiverbatim` arg list of
/// the single-cite commands into (star, pre, post, keys).
fn blx_cite_args(args: Vec<ArgWrap>) -> (bool, Option<Tokens>, Option<Tokens>, Tokens) {
  let mut it = args.into_iter();
  let star: Option<Tokens> = it.next().unwrap().into();
  let pre: Option<Tokens> = it.next().unwrap().into();
  let post: Option<Tokens> = it.next().unwrap().into();
  let keys: Tokens = it.next().unwrap().into();
  (blx_nonempty(&star), pre, post, keys)
}

/// Perl label split `/^(.+),\s*(\d{4}\w*)$/` — greedy `.+` means the LAST
/// comma whose tail is a 4-digit year plus an optional disambiguation
/// suffix wins. Returns (author_part, year_with_suffix).
fn blx_split_ay_label(label: &str) -> Option<(String, String)> {
  for (idx, _) in label.char_indices().rev().filter(|(_, c)| *c == ',') {
    if idx == 0 {
      continue;
    }
    let tail = label[idx + 1..].trim_start();
    if tail.len() >= 4
      && tail.chars().take(4).all(|c| c.is_ascii_digit())
      && tail
        .chars()
        .skip(4)
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
      return Some((label[..idx].to_string(), tail.to_string()));
    }
  }
  None
}

#[rustfmt::skip]
LoadDefinitions!({
  // Strict-Perl translation of ar5iv-bindings/biblatex.sty.ltxml
  // (803 lines). All macro definitions, conditionals, registers, the
  // trailing RawTeX toggle block, AND the deep-closure bibliography
  // rebuilder (Perl L110-263 \entry/\endentry, L270-340 \name,
  // L367-397 \verb) are now ported — the `.bbl` is assembled into a
  // real `\thebibliography`.
  //
  // Audit cycle 2: caught Rust-only bugs vs Perl source
  //   * duplicate `\providetoggle{blx@citation}` (etoolbox toggle redef)
  //   * missing 38 of 60 toggles in trailing RawTeX
  //   * missing `\addbibresource` / `\printbibliography` /
  //     `Let \bibliography → \addbibresource` chain
  //   * 60+ DefMacro/DefRegister/DefConditional declarations missing
  //
  // Audit cycle 3 (full Perl-parity pass of the rebuilder closures):
  //   * \name: ported the keyval-name branch (Perl L301-306) gated on
  //     `biblatex_with_keyvals`. Modern biber .bbl (format ≥ 3.x) encodes
  //     authors as `{{meta,hash}{family={…},given={…}}}`; the old port only
  //     handled the positional form and leaked the whole keyval blob as the
  //     "family" string (`family=…,familyi=…,given=…`) into the HTML.
  //   * \endentry: ported the label collision-suffixing (L148-162) and the
  //     previously-dropped fields — series (L220-222), howpublished (L227),
  //     organization (L230), eprintclass (L248), the DOI/eprint leading-{/
  //     trailing-} URI trim (L232-237/L253-258), and the url-branch
  //     eprinttype label (L240-244).
  //   * \bibrangedash: restored the Perl/real-biblatex en-dash (a late
  //     redefinition had clobbered it to a hyphen).
  //   KNOWN LIMITATION: the 3-arg `\name` variant is not auto-detected
  //   (we declare the 4-arg modern shape).
  //
  // Audit cycle 4 (2026-07-03, ar5iv-bindings PRs #20/#21 + repair 0911aec):
  //   * author-year citation commands: \parencite/\textcite/\cite families
  //     as \@@cite/\@@bibref closures (blx_cite_*), greedy multicite
  //     readers (\cites/\parencites/\textcites + capitalized forms), and
  //     \citeauthor/\citetitle/\citeyear/\citeyearpar — all gated on
  //     style=/citestyle= author-year detection, with a saved-core-\cite
  //     fallback so numeric documents are unchanged.
  //   * author-year "Surname, Year" labels for label-less biber .bbl
  //     entries, emitted as \blx@lbibitem + a single role-tagged \bbl@tags
  //     (schema-valid, natbib \NAT@@wrout shape) so Post::CrossRef renders
  //     "(Smith, 2020)" / "Jones & Brown (2019)".
  //   * \name: prefix/suffix name parts; surnames recorded for labels.
  //   * \printbibliography[]: biber .bbl is ground truth when present,
  //     resource-based \bibliography fallback otherwise; & catcode guard.
  //   * maxbibnames= package option now wired (opt@ scan), closing the
  //     earlier limitation.

  // Perl L14-15: Warn that biblatex.sty is only minimally stubbed.
  Warn!("missing_file", "biblatex.sty",
    "biblatex.sty is only minimally stubbed and will not be interpreted raw.");

  // Perl option processing: maxbibnames, style/citestyle keyvals, ignore
  // the rest. Perl wires these through DefKeyVal `code` callbacks; here we
  // register the keys for keyset parity and read the raw package-option
  // list after ProcessOptions (the svg.sty.ltxml pattern) — biblatex's
  // style is essentially always given as a package option.
  DefKeyVal!("biblatex", "maxbibnames", "Number", "4");
  DefKeyVal!("biblatex", "style", "Semiverbatim");
  DefKeyVal!("biblatex", "citestyle", "Semiverbatim");
  // Perl `DeclareOption(undef, sub { })` — ignore unknown options.
  DeclareOption!(None, {});
  ProcessOptions!();
  if let Some(opts) = lookup_vecdeque("opt@biblatex.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      let opt_str = opt_str.trim();
      if let Some(v) = opt_str.strip_prefix("style=")
        .or_else(|| opt_str.strip_prefix("citestyle=")) {
        blx_set_style(v.trim().trim_start_matches('{').trim_end_matches('}'));
      } else if let Some(v) = opt_str.strip_prefix("maxbibnames=")
        && let Ok(n) = v.trim().trim_start_matches('{').trim_end_matches('}').parse::<i64>() {
          bib_state_set_int("biblatex_maxbibnames", n);
        }
    }
  }

  // Perl L24-30: dependencies. (`#RequirePackage('natbib')` etc commented out in Perl.)
  RequirePackage!("hyperref");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("babel_support");

  // Cite commands — the three-family architecture from ar5iv-bindings
  // PRs #20/#21 (+ 0911aec repairs). Every command is a code closure over
  // blx_cite_* / blx_multicite_*; in non-author-year styles they all
  // delegate to the saved core \cite (blx_cite_fallback), so numeric
  // documents render exactly as before.
  //
  // Save the core \cite FIRST: \cite itself is redefined below, and the
  // fallback must reach the original, not recurse into our closure.
  Let!("\\blx@saved@cite", "\\cite");

  // -- Parenthetical commands: (prenote Author, Year, postnote) ---------
  DefMacro!("\\parencite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_parenthetical(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\Parencite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_parenthetical(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\autocite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_parenthetical(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\Autocite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_parenthetical(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\citep OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_parenthetical(star, pre, post, keys)
  }, locked => true);

  // -- Textual commands: prenote Author (Year, postnote) ----------------
  DefMacro!("\\textcite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_textual(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\Textcite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_textual(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\citet OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_textual(star, pre, post, keys)
  }, locked => true);

  // -- Bare / no-parens commands: prenote Author, Year, postnote --------
  // In author-year styles \cite is "Author, Year" without parentheses; in
  // other styles it falls back to \blx@saved@cite.
  DefMacro!("\\cite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\Cite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\citealt OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\citealp OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\fullcite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  // TODO: \footcite should render inside a footnote, \footcitetext
  // likewise; \supercite as a superscript number. Stubbed as bare inline
  // citations (same as the Perl binding).
  DefMacro!("\\footcite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\footcitetext OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\smartcite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\supercite OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  DefMacro!("\\citenum OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);
  // \citem — see 1606.07864.
  DefMacro!("\\citem OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    blx_cite_bare(star, pre, post, keys)
  }, locked => true);

  // -- Author/title/year-only commands -----------------------------------
  DefMacro!("\\citeauthor OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (star, pre, post, keys) = blx_cite_args(args);
    let (pre, post) = blx_swap_pre_post(pre, post);
    if !blx_is_authoryear() {
      return Ok(blx_cite_fallback(post, keys));
    }
    let author = if star { "FullAuthors" } else { "Authors" };
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!(author)), keys, Tokens!(), Tokens!()]);
    let mut body = Vec::new();
    if let Some(p) = pre { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post {
      body.extend(blx_ns().unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citeauthor")), Tokens::new(body)]))
  }, locked => true);
  DefMacro!("\\citetitle OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (_star, pre, post, keys) = blx_cite_args(args);
    let (pre, post) = blx_swap_pre_post(pre, post);
    if !blx_is_authoryear() {
      return Ok(blx_cite_fallback(post, keys));
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Title")), keys, Tokens!(), Tokens!()]);
    let mut body = Vec::new();
    if let Some(p) = pre { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post {
      body.extend(blx_ns().unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citetitle")), Tokens::new(body)]))
  }, locked => true);
  DefMacro!("\\citeyear OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (_star, pre, post, keys) = blx_cite_args(args);
    let (pre, post) = blx_swap_pre_post(pre, post);
    if !blx_is_authoryear() {
      return Ok(blx_cite_fallback(post, keys));
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Year")), keys, Tokens!(), Tokens!()]);
    let mut body = Vec::new();
    if let Some(p) = pre { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post {
      body.extend(blx_ns().unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citeyear")), Tokens::new(body)]))
  }, locked => true);
  DefMacro!("\\citeyearpar OptionalMatch:* [][] Semiverbatim", sub[args] {
    let (_star, pre, post, keys) = blx_cite_args(args);
    let (pre, post) = blx_swap_pre_post(pre, post);
    if !blx_is_authoryear() {
      return Ok(blx_cite_fallback(post, keys));
    }
    let bibref = Invocation!(T_CS!("\\@@bibref"),
      vec![Tokens::new(Explode!("Year")), keys, Tokens!(), Tokens!()]);
    let mut body = blx_open().unlist();
    if let Some(p) = pre { body.extend(p.unlist()); body.push(T_SPACE!()); }
    body.extend(bibref.unlist());
    if let Some(p) = post {
      body.extend(blx_ns().unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
    }
    body.extend(blx_close().unlist());
    Ok(Invocation!(T_CS!("\\@@cite"),
      vec![Tokens::new(Explode!("citeyearpar")), Tokens::new(body)]))
  }, locked => true);

  // -- Multicite commands: repeated [pre][post]{keys} groups -------------
  // Greedy gullet reading (blx_read_multicite_groups); groups joined "; ".
  DefMacro!("\\cites OptionalMatch:*", sub[(star)] {
    blx_multicite_bare(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\Cites OptionalMatch:*", sub[(star)] {
    blx_multicite_bare(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\parencites OptionalMatch:*", sub[(star)] {
    blx_multicite_parenthetical(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\Parencites OptionalMatch:*", sub[(star)] {
    blx_multicite_parenthetical(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\autocites OptionalMatch:*", sub[(star)] {
    blx_multicite_parenthetical(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\Autocites OptionalMatch:*", sub[(star)] {
    blx_multicite_parenthetical(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\textcites OptionalMatch:*", sub[(star)] {
    blx_multicite_textual(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\Textcites OptionalMatch:*", sub[(star)] {
    blx_multicite_textual(blx_nonempty(&star))
  }, locked => true);
  // Rust extras beyond the Perl binding (kept from the earlier stub set):
  // footnote/superscript multicites degrade to bare; \citetexts to textual.
  DefMacro!("\\smartcites OptionalMatch:*", sub[(star)] {
    blx_multicite_bare(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\footcites OptionalMatch:*", sub[(star)] {
    blx_multicite_bare(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\supercites OptionalMatch:*", sub[(star)] {
    blx_multicite_bare(blx_nonempty(&star))
  }, locked => true);
  DefMacro!("\\citetexts OptionalMatch:*", sub[(star)] {
    blx_multicite_textual(blx_nonempty(&star))
  }, locked => true);
  // \citelist{ \cite{key1}*{pre} \cite{key2}*{pre} } — biblatex
  // multi-citation grouped under parens, where each `\cite{...}*{...}`
  // is a postnote-bearing entry. Degrade to passing the body through;
  // each inner `\cite` renders independently. Witness 2404.11319.
  DefMacro!("\\citelist{}", "#1");

  // \DeclareLabeldate — biblatex datacommands declaration. No-op stub.
  def_macro_noop("\\DeclareLabeldate {}")?;

  // Perl L64-67: passthroughs
  DefMacro!("\\unspace", "\\relax");
  DefMacro!("\\blx@imc@resetpunctfont", "\\relax");
  DefMacro!("\\blx@postpunct", "\\@empty");
  DefRegister!("\\c@highnamepenalty" => Number(0));

  // Perl L69-72
  DefMacro!("\\addslash", "/\\hskip\\z@skip");
  DefMacro!("\\adddot", ".");
  DefMacro!("\\addcomma", ",");
  DefMacro!("\\autocap{}", "#1");

  // Perl L75-85
  DefMacro!("\\addspace",        "\\space");
  DefMacro!("\\addnbspace",      "\\space");
  DefMacro!("\\addthinspace",    "\\space");
  DefMacro!("\\addnbthinspace",  "\\space");
  DefMacro!("\\addlowpenspace",  "\\space");
  DefMacro!("\\addhighpenspace", "\\space");
  DefMacro!("\\addlpthinspace",  "\\space");
  DefMacro!("\\addhpthinspace",  "\\space");
  DefMacro!("\\addabbrvspace",   "\\space");
  DefMacro!("\\addabthinspace",  "\\space");
  DefMacro!("\\adddotspace",     "\\unspace\\adddot\\space");

  // Perl L87-91
  DefMacro!("\\noligature",   "\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphen",       "\\nobreak-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\nbhyphen",     "\\nobreak\\mbox{-}\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphenate",    "\\nobreak\\-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\allowhyphens", "\\nobreak\\hskip\\z@skip");

  // Perl L93-99
  DefMacro!("\\bibinitperiod",      "\\adddot");
  DefMacro!("\\bibinithyphendelim", ".\\mbox{-}");
  DefMacro!("\\bibnamedelima",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimb",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimc",      "\\addhighpenspace");
  DefMacro!("\\bibnamedelimd",      "\\addlowpenspace");
  DefMacro!("\\bibnamedelimi",      "\\addnbspace");

  // Perl L101-106: \datalist / \sortlist set the `biblatex_with_keyvals`
  // flag globally — Perl's `\name` closure (Cycle 9) reads it to choose
  // 3-arg vs 4-arg / keyval-vs-positional dispatch.
  DefMacro!("\\datalist[]{}", sub[_args] {
    assign_value("biblatex_with_keyvals", Stored::from(1),
      Some(Scope::Global));
    Ok(Tokens::new(vec![]))
  });
  DefMacro!("\\sortlist[]{}", sub[_args] {
    assign_value("biblatex_with_keyvals", Stored::from(1),
      Some(Scope::Global));
    Ok(Tokens::new(vec![]))
  });
  // Perl L107-108: \lossort / \refsection — empty stubs.
  DefMacro!("\\lossort", "", locked => true);
  DefMacro!("\\refsection{}", "", locked => true);

  // biblatex `.bbl` files emitted by biber include `\true{moreauthor}` /
  // `\true{morelabelname}` / `\false{...}` flags on multi-author entries.
  // Perl `ar5iv-bindings/bindings/biblatex.sty.ltxml:641-645` defines
  // `\blx@bbl@booltrue{}` / `\blx@bbl@boolfalse{}` as `\relax` stubs and
  // `\let\true\blx@bbl@booltrue` if `\true` is undefined.
  //
  // Rust never sets either, so the .bbl raw-load hits
  // `Error:undefined:\true` on every multi-author bibitem (witness:
  // arXiv:2509.15629 / 2509.21728 — biblatex `.bbl` v3.3 format with
  // multi-author entries).
  DefMacro!("\\blx@bbl@booltrue{}", "", locked => true);
  DefMacro!("\\blx@bbl@boolfalse{}", "", locked => true);
  Let!("\\true", "\\blx@bbl@booltrue");
  Let!("\\false", "\\blx@bbl@boolfalse");

  // biblatex `\keyalias{alias}{target}` (TL biblatex.sty L8519-8521 +
  // L8858 `\let\keyalias\blx@bbl@keyalias`) maps a cite-key alias to
  // the canonical entry key. We don't track these mappings (our \cite
  // resolves directly), so the stub can be a no-op. Witness:
  // arXiv:2510.00068 — biblatex .bbl with 49 `\keyalias{...}{...}`
  // entries, each generating an undefined-CS error.
  DefMacro!("\\blx@bbl@keyalias{}{}", "", locked => true);
  Let!("\\keyalias", "\\blx@bbl@keyalias");
  // Perl L122-125: \enddatalist / \endsortlist / \endlossort / \endrefsection
  // → biblatex_as_thebibliography rebuilder. Wraps the accumulated bibitems
  // emitted by repeated \endentry calls in `\thebibliography{count}…
  // \endthebibliography`.
  DefMacro!("\\enddatalist", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);
  DefMacro!("\\endsortlist", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);
  DefMacro!("\\endlossort", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);
  DefMacro!("\\endrefsection", sub[_args] {
    Ok(bib_as_thebibliography())
  }, locked => true);

  // Author-year bibitem support, mirroring natbib's \@@lbibitem +
  // \NAT@@wrout: \blx@lbibitem opens the <ltx:bibitem> (no tags, no
  // bibblock — ltx:bibblock is autoOpen, so the following entry text opens
  // one), and \bbl@tags emits the bibitem's single <ltx:tags> with the
  // role="authors"/"year"/... tags that Post::CrossRef uses to build
  // author-year citations with phrase support. The schema model is
  // bibitem = (tags?, bibblock*): one tags element, first.
  DefConstructor!("\\blx@lbibitem Semiverbatim",
    "<ltx:bibitem key='#key' xml:id='#id'>",
    after_digest => sub[whatsit] {
      let key = whatsit.get_arg(1)
        .map(|a| clean_bib_key(&a.to_string()))
        .unwrap_or_default();
      let mut properties = RefStepID!("@bibitem")?;
      properties.insert("key", key.into());
      whatsit.set_properties(properties);
    });
  // Perl `bounded => 1` + `beforeDigest => Let(T_ALIGN, '\&')`; like the
  // Rust NAT@@wrout port, the bgroup/soft-egroup pair replaces `bounded`
  // (whose egroup mode-frame check trips on inner mode switches) while
  // still scope-isolating the T_ALIGN Let to argument digestion.
  DefConstructor!("\\bbl@tags{}{}{}{}",
    "<ltx:tags>\
      ?#1(<ltx:tag role='year'>#1</ltx:tag>)\
      ?#2(<ltx:tag role='authors'>#2</ltx:tag>)\
      ?#3(<ltx:tag role='fullauthors'>#3</ltx:tag>)\
      ?#4(<ltx:tag role='refnum'>#4</ltx:tag>)\
    </ltx:tags>",
    before_digest => {
      bgroup();
      Let!(T_ALIGN!(), T_CS!("\\&"));
    },
    after_digest => sub[_whatsit] {
      pop_stack_frame(false)?;
      Ok(Vec::new())
    });

  // Perl L127-130: \entry{key}{type}{} initializes the entry hash so that the
  // following \field/\strng/\name/\list directives have a place to record
  // metadata. The 3rd arg is options (Perl ignores it).
  DefMacro!("\\entry{}{}{}", sub[(key, ty, _opts)] {
    let mut entry: SymHashMap<Stored> = SymHashMap::default();
    entry.insert("key", Stored::Tokens(key));
    entry.insert("type", Stored::Tokens(ty));
    bib_entry_save(entry);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L132-263: \endentry — flush the accumulated entry hash as a
  // `\bibitem[label]{key} authors. \newblock title. \newblock In: journal.
  // year, pages.` token stream onto rebuilt_bibtex_variant.
  // Simplified port: handles label-or-auto-label, key, authors (if collected
  // by \name as plain "fullname" tokens), title, journal/booktitle, year,
  // pages, doi/url/eprint. Pre-typeset name strings (the `{names}` array
  // form) are emitted comma-joined.
  DefMacro!("\\endentry", sub[_args] {
    let entry = bib_entry_get();
    assign_value("biblatex_entry", Stored::None, Some(Scope::Global));

    // label: labelalpha if present, else label; strip CSes + braces.
    // For author-year styles (apa, nyt, ...) biber emits NO labelalpha —
    // construct a "Surname, Year" label from the recorded name/year data
    // (ar5iv-bindings PR #21 + 0911aec). Gated on the detected style:
    // numeric documents keep their sequential [1],[2],... labels.
    // If still empty, fall back to an incrementing counter; else ensure
    // uniqueness with a/b/.../z suffixing.
    let mut ay_label = false;
    let year_str = bib_entry_get_tokens(&entry, "year")
      .map(|t| bib_clean_name(&t.to_string()))
      .unwrap_or_default();
    let surnames: Vec<String> = match entry.get("authors_surnames") {
      Some(Stored::String(s)) => to_string(*s)
        .split('\u{1f}').filter(|p| !p.is_empty()).map(String::from).collect(),
      _ => Vec::new(),
    };
    let label_str: String = {
      let mut cleaned = bib_entry_get_tokens(&entry, "labelalpha")
        .or_else(|| bib_entry_get_tokens(&entry, "label"))
        .map(|t| bib_clean_name(&t.to_string()))
        .filter(|s| !s.is_empty());
      if cleaned.is_none() && blx_is_authoryear() && !surnames.is_empty() && !year_str.is_empty() {
        let author_part = match surnames.len() {
          1 => surnames[0].clone(),
          2 => format!("{} & {}", surnames[0], surnames[1]),
          _ => format!("{} et al.", surnames[0]),
        };
        cleaned = Some(format!("{author_part}, {year_str}"));
        ay_label = true;
      }
      match cleaned {
        Some(label) => {
          // Perl L148-162: collision-avoidance suffixing, tracked globally in
          // `biblatex_author_labels`. The `z`-wraparound (append another base
          // 'a' and restart) is faithfully ported — see arXiv:1212.4446.
          let mut labels: SymHashMap<Stored> = match lookup_value("biblatex_author_labels") {
            Some(Stored::HashStored(m)) => m,
            _ => SymHashMap::default(),
          };
          let final_label = if labels.contains_key(&label) {
            let mut base = label;
            let mut suffix = b'a';
            while labels.contains_key(&format!("{base}{}", suffix as char)) {
              if suffix == b'z' { base.push('a'); suffix = b'a'; }
              else { suffix += 1; }
            }
            format!("{base}{}", suffix as char)
          } else {
            label
          };
          labels.insert(&final_label, Stored::from(1));
          assign_value("biblatex_author_labels",
            Stored::HashStored(labels), Some(Scope::Global));
          final_label
        },
        None => {
          // Perl L144-147: no usable label → simple incrementing counter.
          let n = bib_state_int("biblatex_auto_label") + 1;
          bib_state_set_int("biblatex_auto_label", n);
          n.to_string()
        },
      }
    };

    // Bump entry count for thebibliography wrapper.
    bib_state_set_int("biblatex_entry_count", bib_state_int("biblatex_entry_count") + 1);

    let mut variant: Vec<Token> = Vec::with_capacity(64);
    let key_toks = bib_entry_get_tokens(&entry, "key").unwrap_or_default();
    if ay_label {
      // Author-year entry: open the bibitem via \blx@lbibitem (no tags, no
      // bibblock — ltx:bibblock is autoOpen) and emit the single
      // structured <ltx:tags> with the role="authors"/"year"/... tags that
      // Post::CrossRef needs to resolve \textcite / \parencite with phrase
      // support. Mirrors natbib's \@@lbibitem + \NAT@@wrout; keeps the
      // schema model bibitem = (tags?, bibblock*) valid.
      let (author_part, ay_year) = blx_split_ay_label(&label_str)
        .unwrap_or_else(|| (label_str.clone(), year_str.clone()));
      // The year part carries any disambiguation suffix (2020a, ...) so
      // same-author-same-year entries stay distinct.
      let refnum_str = if author_part == label_str {
        label_str.clone()
      } else {
        format!("{author_part} ({ay_year})")
      };
      let full_author_part = match surnames.len() {
        0 => author_part.clone(),
        1 | 2 => surnames.join(" & "),
        _ => format!("{} & {}",
          surnames[..surnames.len() - 1].join(", "),
          surnames[surnames.len() - 1]),
      };
      variant.push(T_CS!("\\blx@lbibitem"));
      variant.push(T_BEGIN!());
      variant.extend(key_toks.unlist());
      variant.push(T_END!());
      variant.push(T_CS!("\\bbl@tags"));
      for tag in [&ay_year, &author_part, &full_author_part, &refnum_str] {
        variant.push(T_BEGIN!());
        variant.extend(ExplodeText!(tag));
        variant.push(T_END!());
      }
    } else {
      // Numeric / alphabetic / sequential-fallback entry: plain \bibitem,
      // exactly as before author-year support was added.
      variant.push(T_CS!("\\bibitem"));
      variant.push(T_OTHER!("["));
      variant.extend(ExplodeText!(&label_str));
      variant.push(T_OTHER!("]"));
      variant.push(T_BEGIN!());
      variant.extend(key_toks.unlist());
      variant.push(T_END!());
    }

    // Authors: if \name stashed a comma-joined string under "authors_str",
    // emit it. Defer the et-al / per-author re-tokenization for now — most
    // .bbl files give us pre-formatted author tokens.
    let authors_toks = bib_entry_get_tokens(&entry, "authors_str");
    let mut have_authors = false;
    if let Some(toks) = authors_toks
      && !toks.is_empty() {
        variant.extend(toks.unlist());
        have_authors = true;
      }

    // Title
    if let Some(title) = bib_entry_get_tokens(&entry, "title")
      && !title.is_empty() {
        if have_authors {
          variant.push(T_CS!("\\newblock"));
        }
        variant.push(T_OTHER!("`"));
        variant.push(T_OTHER!("`"));
        variant.extend(title.unlist());
        variant.push(T_OTHER!("'"));
        variant.push(T_OTHER!("'"));
      }
    // Note
    if let Some(note) = bib_entry_get_tokens(&entry, "note")
      && !note.is_empty() {
        variant.push(T_SPACE!());
        variant.extend(note.unlist());
      }
    // Journal / booktitle
    let journal = bib_entry_get_tokens(&entry, "booktitle")
      .or_else(|| bib_entry_get_tokens(&entry, "journaltitle"))
      .or_else(|| bib_entry_get_tokens(&entry, "journal"));
    if let Some(j) = journal.as_ref()
      && !j.is_empty() {
        variant.push(T_CS!("\\newblock"));
        variant.extend(ExplodeText!("In "));
        variant.push(T_CS!("\\emph"));
        variant.push(T_BEGIN!());
        variant.extend(j.clone().unlist());
        variant.push(T_END!());
      }
    // Volume + (number) — Perl L217-219: gated on a booktitle/journaltitle/series.
    let series = bib_entry_get_tokens(&entry, "series");
    let has_volume = bib_entry_get_tokens(&entry, "volume")
      .map(|v| !v.is_empty()).unwrap_or(false);
    if let Some(volume) = bib_entry_get_tokens(&entry, "volume")
      && !volume.is_empty() && (journal.is_some() || series.is_some()) {
        variant.push(T_SPACE!());
        variant.push(T_CS!("\\textbf"));
        variant.push(T_BEGIN!());
        variant.extend(volume.unlist());
        if let Some(num) = bib_entry_get_tokens(&entry, "number")
          && !num.is_empty() {
            variant.push(T_OTHER!("."));
            variant.extend(num.unlist());
          }
        variant.push(T_END!());
      }
    // Series — Perl L220-222. Trailing number only when there is no volume.
    if let Some(series) = series.as_ref()
      && !series.is_empty() {
        variant.push(T_OTHER!(","));
        variant.push(T_SPACE!());
        variant.extend(series.clone().unlist());
        if !has_volume
          && let Some(num) = bib_entry_get_tokens(&entry, "number")
            && !num.is_empty() {
              variant.push(T_SPACE!());
              variant.extend(num.unlist());
            }
      }
    // Publisher / location
    if let Some(publisher) = bib_entry_get_tokens(&entry, "publisher")
      && !publisher.is_empty() {
        variant.push(T_CS!("\\newblock"));
        if let Some(loc) = bib_entry_get_tokens(&entry, "location")
          && !loc.is_empty() {
            variant.extend(loc.unlist());
            variant.push(T_OTHER!(":"));
            variant.push(T_SPACE!());
          }
        variant.extend(publisher.unlist());
      }
    // howpublished — Perl L227.
    if let Some(howpub) = bib_entry_get_tokens(&entry, "howpublished")
      && !howpub.is_empty() {
        variant.push(T_OTHER!(","));
        variant.push(T_SPACE!());
        variant.extend(howpub.unlist());
      }
    // Year
    if let Some(year) = bib_entry_get_tokens(&entry, "year")
      && !year.is_empty() {
        variant.push(T_OTHER!(","));
        variant.push(T_SPACE!());
        variant.extend(year.unlist());
      }
    // Pages
    if let Some(pages) = bib_entry_get_tokens(&entry, "pages")
      && !pages.is_empty() {
        variant.push(T_OTHER!(","));
        variant.push(T_SPACE!());
        variant.extend(ExplodeText!("pp. "));
        variant.extend(pages.unlist());
      }
    // organization — Perl L230.
    if let Some(org) = bib_entry_get_tokens(&entry, "organization")
      && !org.is_empty() {
        variant.push(T_CS!("\\newblock"));
        variant.extend(org.unlist());
      }
    // DOI / URL / eprint — Perl L231-260.
    if let Some(doi) = bib_entry_get_tokens(&entry, "doi").filter(|t| !t.is_empty()) {
      // Perl L232-237: trim leading/trailing space + braces for a clean URI.
      let doi_toks = bib_trim_url_tokens(doi);
      variant.push(T_CS!("\\newblock"));
      variant.extend(ExplodeText!("DOI: "));
      variant.push(T_CS!("\\href"));
      variant.push(T_BEGIN!());
      variant.extend(ExplodeText!("https://dx.doi.org/"));
      variant.extend(doi_toks.clone());
      variant.push(T_END!());
      variant.push(T_BEGIN!());
      variant.extend(doi_toks);
      variant.push(T_END!());
    } else if let Some(url) = bib_entry_get_tokens(&entry, "url").filter(|t| !t.is_empty()) {
      // Perl L240-244: label from eprinttype (uppercased), default URL, arXiv if ARXIV.
      let etype = bib_entry_get_tokens(&entry, "eprinttype")
        .map(|t| t.to_string().to_uppercase()).unwrap_or_default();
      let etype = if etype.is_empty() { "URL".to_string() }
        else if etype == "ARXIV" { "arXiv".to_string() }
        else { etype };
      variant.push(T_CS!("\\newblock"));
      variant.extend(ExplodeText!(&format!("{etype}: ")));
      variant.push(T_CS!("\\url"));
      variant.push(T_BEGIN!());
      variant.extend(url.unlist());
      variant.push(T_END!());
    } else if let Some(eprint) = bib_entry_get_tokens(&entry, "eprint").filter(|t| !t.is_empty()) {
      // Perl L245-260.
      let etype = bib_entry_get_tokens(&entry, "eprinttype")
        .map(|t| t.to_string().to_uppercase()).unwrap_or_default();
      let is_arxiv = etype == "ARXIV";
      let etype = if etype.is_empty() { "eprint".to_string() }
        else if is_arxiv { "arXiv".to_string() }
        else { etype };
      let eprint_class = bib_entry_get_tokens(&entry, "eprintclass").filter(|t| !t.is_empty());
      variant.push(T_CS!("\\newblock"));
      // Perl L260: no space between the "type:" label and the target.
      variant.extend(ExplodeText!(&format!("{etype}:")));
      if is_arxiv {
        let eprint_toks = bib_trim_url_tokens(eprint);
        variant.push(T_CS!("\\href"));
        variant.push(T_BEGIN!());
        variant.extend(ExplodeText!("https://arxiv.org/abs/"));
        variant.extend(eprint_toks.clone());
        variant.push(T_END!());
        variant.push(T_BEGIN!());
        variant.extend(eprint_toks);
        // Perl L248: eprintclass → " [class]" suffix, inside the link text.
        if let Some(cls) = eprint_class {
          variant.push(T_SPACE!());
          variant.push(T_OTHER!("["));
          variant.extend(cls.unlist());
          variant.push(T_OTHER!("]"));
        }
        variant.push(T_END!());
      } else {
        variant.extend(eprint.unlist());
      }
    }

    bib_variant_push(variant);
    Ok(Tokens::default())
  }, locked => true);

  // BiblatexAuthor keyvals — extended (PR #21) with prefix/suffix/…-un
  // name parts so "van der Berg" / "King Jr." names parse correctly.
  DefKeyVal!("BiblatexAuthor", "given",    "");
  DefKeyVal!("BiblatexAuthor", "giveni",   "");
  DefKeyVal!("BiblatexAuthor", "givenun",  "");
  DefKeyVal!("BiblatexAuthor", "family",   "");
  DefKeyVal!("BiblatexAuthor", "familyi",  "");
  DefKeyVal!("BiblatexAuthor", "familyun", "");
  DefKeyVal!("BiblatexAuthor", "prefix",   "");
  DefKeyVal!("BiblatexAuthor", "prefixi",  "");
  DefKeyVal!("BiblatexAuthor", "prefixun", "");
  DefKeyVal!("BiblatexAuthor", "suffix",   "");
  DefKeyVal!("BiblatexAuthor", "suffixi",  "");
  DefKeyVal!("BiblatexAuthor", "nameun",   "");

  // Perl L270-346: \name{type}{count}{maybe-content} — biblatex's author
  // record. The TeX-2.5+ .bbl shape is `\name{author}{N}{}{ {{}{Family}…} }`
  // where the 3rd arg is empty and the 4th is the author body. Older variants
  // pass 3 args. Simplified port: declare 4 mandatory args; the 4th-arg
  // capture covers the modern shape used by the vast majority of arxiv .bbl
  // files. The body holds N inner-author groups, each of the form
  // `{hash}{family}{familyi}{given}{giveni}{}{}{}{}` — we extract family +
  // given pairs into "Given Family" strings (no Perl-faithful keyval/hash
  // ordering yet) and stash them comma-joined under `authors_str` /
  // `editors_str` in the entry hash for `\endentry` to emit verbatim.
  DefMacro!("\\name{}{}{}{}", sub[(ty, _count, _maybe, body)] {
    let type_str = ty.to_string();
    // The body's tokens start with `{` `{}` (empty hash) or `{hash=…}` then
    // `{family}{familyi}{given}{giveni}{}{}{}{}` repeated, separated by
    // optional whitespace. Walk top-level groups.
    let body_toks: Vec<Token> = body.unlist();
    // Helper: scan one balanced {...} group, advancing index.
    fn read_group(tokens: &[Token], i: &mut usize) -> Option<Vec<Token>> {
      while *i < tokens.len() {
        let cc = tokens[*i].code;
        if cc == Catcode::SPACE { *i += 1; continue; }
        if cc == Catcode::BEGIN { break; }
        return None; // not a group
      }
      if *i >= tokens.len() { return None; }
      *i += 1; // consume BEGIN
      let mut depth = 1usize;
      let mut out = Vec::new();
      while *i < tokens.len() {
        let cc = tokens[*i].code;
        if cc == Catcode::BEGIN {
          depth += 1;
          out.push(tokens[*i]);
        } else if cc == Catcode::END {
          depth -= 1;
          if depth == 0 { *i += 1; return Some(out); }
          out.push(tokens[*i]);
        } else {
          out.push(tokens[*i]);
        }
        *i += 1;
      }
      Some(out) // unterminated: best effort
    }
    // Perl L286: `$keyvals_flag = LookupValue('biblatex_with_keyvals')`, set
    // by \datalist / \sortlist. Modern biber .bbl (format ≥ 3.x) encodes each
    // author as `{{meta,hash}{family={…},given={…},…}}` (keyval block); older
    // .bbl uses positional `{{hash}{family}{familyi}{given}{giveni}…}`. Without
    // this dispatch the keyval block was grabbed wholesale as "family" and
    // leaked verbatim into the bibliography (`family=…,familyi=…,given=…`).
    let keyvals_flag = bib_state_int("biblatex_with_keyvals") != 0;
    let mut names: Vec<String> = Vec::new();
    // Family name parts recorded alongside the full names, for author-year
    // label construction (\endentry): "van der Berg, Pieter" must label as
    // "Berg, YEAR", and "Martin Luther King Jr." as "King", not "Jr.".
    let mut surnames: Vec<String> = Vec::new();
    let mut idx = 0usize;
    while idx < body_toks.len() {
      // Skip space tokens between author groups.
      while idx < body_toks.len() &&
            body_toks[idx].code == Catcode::SPACE {
        idx += 1;
      }
      if idx >= body_toks.len() { break; }
      // Read the per-author group.
      let author_grp = match read_group(&body_toks, &mut idx) {
        Some(g) => g,
        None => break,
      };
      // First sub-group is always the per-author metadata/hash block
      // (`{hash=…}` or `{un=0,uniquepart=base,hash=…}`); skip it (Perl L294).
      let mut j = 0usize;
      let _meta = read_group(&author_grp, &mut j);
      let (given, prefix, family, suffix) = if keyvals_flag {
        // Keyval form: the next sub-group is the keyval block. Prefer the
        // full name parts over the `i`-initial forms; prefix/suffix are
        // optional (PR #21).
        let kv_str = read_group(&author_grp, &mut j)
          .map(|g| Tokens::new(g).to_string()).unwrap_or_default();
        let kvs = parse_name_keyvals(&kv_str);
        let get = |k: &str| kvs.iter().find(|(kk, _)| kk == k).map(|(_, v)| v.clone());
        let given = get("given").or_else(|| get("giveni")).unwrap_or_default();
        let prefix = get("prefix").or_else(|| get("prefixi")).unwrap_or_default();
        let family = get("family").or_else(|| get("familyi")).unwrap_or_default();
        let suffix = get("suffix").or_else(|| get("suffixi")).unwrap_or_default();
        (given, prefix, family, suffix)
      } else {
        // Positional form (Perl L308-321): {family}{familyi}{given}{giveni}…
        let family = read_group(&author_grp, &mut j)
          .map(|g| Tokens::new(g).to_string()).unwrap_or_default();
        let _familyi = read_group(&author_grp, &mut j);
        let given = read_group(&author_grp, &mut j)
          .map(|g| Tokens::new(g).to_string()).unwrap_or_default();
        (given, String::new(), family, String::new())
      };
      // Perl L324: strip leftover CSes/braces, then trim.
      let family = bib_clean_name(family.trim());
      let given = bib_clean_name(given.trim());
      let prefix = bib_clean_name(prefix.trim());
      let suffix = bib_clean_name(suffix.trim());
      // Perl: join(' ', grep { $_ ne '' } (given, prefix, family, suffix)).
      let fullname = [given, prefix, family.clone(), suffix]
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
      if fullname.is_empty() {
        continue;
      }
      // Surname for label construction: the family part; fall back to the
      // last word of the full name (pre-typeset corporate authors etc.).
      let surname = if family.is_empty() {
        fullname.split_whitespace().last().unwrap_or_default().to_string()
      } else {
        family
      };
      surnames.push(surname);
      names.push(fullname);
    }
    // Format with et-al limit (default 4 per Perl L192).
    let etal_limit = bib_state_int("biblatex_maxbibnames");
    let etal_limit = if etal_limit > 0 { etal_limit as usize } else { 4 };
    let joined = if names.len() > etal_limit {
      format!("{} et al.", names[0])
    } else {
      let mut acc = String::new();
      let n = names.len();
      for (k, name) in names.iter().enumerate() {
        if k > 0 {
          if k + 1 == n {
            acc.push_str(" and ");
          } else {
            acc.push_str(", ");
          }
        }
        acc.push_str(name);
      }
      acc
    };
    // Stash under "authors_str" or "editors_str" depending on type.
    let is_editor = type_str.trim() == "editor";
    let key = if is_editor { "editors_str" } else { "authors_str" };
    if !joined.is_empty() {
      let toks = Tokens::new(ExplodeText!(&joined));
      bib_entry_set_tokens(key, toks);
    }
    // Record the author surnames (US-separated) for the author-year label
    // construction in \endentry.
    if !is_editor && !surnames.is_empty() {
      let mut entry = bib_entry_get();
      entry.insert("authors_surnames",
        Stored::String(pin(surnames.join("\u{1f}"))));
      bib_entry_save(entry);
    }
    Ok(Tokens::default())
  }, locked => true);

  // Perl L342-346: \list{name}{count}{value} — record value under `name` in
  // the entry hash. Perl ignores `count` for count==1 and notes "more
  // support needed" for >1. Same here.
  DefMacro!("\\list{}{}{}", sub[(name, _count, val)] {
    let name_s = name.to_string();
    bib_entry_set_tokens(name_s.trim(), val);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L355-363: \field{name}{value} / \strng{name}{value} — record value
  // under `name` in the entry hash. Perl uses DefPrimitive (immediate
  // side-effect, no expansion). DefMacro with sub gives equivalent behavior
  // at digestion time since the body is empty.
  DefMacro!("\\field{}{}", sub[(name, val)] {
    let name_s = name.to_string();
    bib_entry_set_tokens(name_s.trim(), val);
    Ok(Tokens::default())
  }, locked => true);
  DefMacro!("\\strng{}{}", sub[(name, val)] {
    let name_s = name.to_string();
    bib_entry_set_tokens(name_s.trim(), val);
    Ok(Tokens::default())
  }, locked => true);

  // Perl L348-354
  def_macro_noop("\\AtEveryBibitem{}")?;
  def_macro_noop("\\AtEveryCitekey{}")?;
  def_macro_noop("\\keyw{}")?;
  def_macro_noop("\\bibinitdelim")?;
  // biblatex.def L219 defines `\bibsetup` as a no-arg user-overridable
  // hook for low-level bibliography layout (interlinepenalty,
  // raggedbottom, frenchspacing, etc.). Layout-only for HTML/XML.
  // Stub as no-op so its call site in `\blx@bibinit` doesn't fire
  // Error:undefined; downstream `\biburlsetup` also no-op.
  // Witness 2310.07484.
  def_macro_noop("\\bibsetup")?;
  def_macro_noop("\\biburlsetup")?;
  // Note: \bibinithyphendelim re-defined here as just "-" per Perl L352
  // (overrides the L94 definition; Perl runs them in order).
  DefMacro!("\\bibinithyphendelim", "-");
  DefMacro!("\\bibrangedash", "\u{2013}");
  DefMacro!("\\bibnamedelimi", " ");

  // Perl L364
  def_macro_noop("\\range{}{}")?;

  // Perl L367-369: \preamble{...} stashes the arg into biblatex_preamble
  // for the rebuilder (Cycle 7) and *also* re-emits the arg (Perl returns
  // $_[1]) so the preamble is digested in the current context too.
  DefMacro!("\\preamble{}", sub[(arg)] {
    assign_value("biblatex_preamble",
      Stored::Tokens(arg.clone()), Some(Scope::Global));
    Ok(arg)
  });

  // Perl L371-397: \biblatex@verb{key}…\endverb captures a verbatim field
  // that biblatex's .bbl emits in the form
  //     \verb{key}
  //     \verb VALUE
  //     \endverb
  // The first `\verb` is `\let`'d to `\biblatex@verb` and reads `{key}`;
  // the second `\verb` then reads VALUE as a raw line; `\endverb` stores
  // VALUE under key. Perl uses gullet->readRawLine + dynamic re-bind. Rust
  // simulates the same effect with a single delimited macro that reads both
  // `{key}` and "Until:\\endverb" — the captured body is everything between
  // the first `\verb{key}` line and `\endverb`, including the second
  // `\verb` token plus the URL chars. We strip the inner `\verb` token and
  // surrounding whitespace before storing.
  // Without this, \verb LEAKS the URL into body text — and consumes the
  // first character (`h` of `http`) as a `{}` arg, producing the
  // characteristic `ttp://…` corruption on egpaper_final.tex.
  DefMacro!("\\biblatex@verb{} Until:\\endverb", sub[(key, body)] {
    let key_str = key.to_string();
    let body_toks = body.unlist();
    // Skip leading whitespace + the inner `\verb` token + one space.
    let mut start = 0usize;
    while start < body_toks.len() {
      let cc = body_toks[start].code;
      if cc == Catcode::SPACE || cc == Catcode::EOL { start += 1; continue; }
      break;
    }
    if start < body_toks.len() && body_toks[start].code == Catcode::CS {
      // Skip the `\verb` CS token (or whatever CS leads — should be \verb).
      start += 1;
      // Skip following space tokens.
      while start < body_toks.len() {
        let cc = body_toks[start].code;
        if cc == Catcode::SPACE || cc == Catcode::EOL { start += 1; continue; }
        break;
      }
    }
    // Strip trailing whitespace.
    let mut end = body_toks.len();
    while end > start {
      let cc = body_toks[end - 1].code;
      if cc == Catcode::SPACE || cc == Catcode::EOL { end -= 1; continue; }
      break;
    }
    // Sanitize: biblatex `\verb` is a verbatim primitive — its body is a
    // literal string. The mouth tokenized chars with their normal catcodes
    // (so `_` is SUB, `^` is SUPER, `#` is PARAM, etc.). When `\endentry`
    // later splices these tokens into `\href{URL}{text}` the SUB chars
    // trigger `Script _ can only appear in math mode` during horizontal
    // digestion. Reset structural catcodes to OTHER so the captured string
    // round-trips through the bibitem variant safely.
    //
    // Also detokenize CS tokens (e.g. `\href`, an inner `\verb`) and
    // brace tokens to literal OTHER characters: biblatex .bbl files
    // occasionally include `\verb \href{url}{label}` inside a `\verb`
    // field (witness arXiv:1004.4538 entry 17, `\verb \href{...}{...}`
    // wrapped across two `\verb` lines). Without detokenization the
    // captured tokens later expand inside `\url{<body>}` and `\href`
    // / `\verb` execute, pushing back tens of thousands of tokens and
    // tripping the 650K PushbackLimit safety net.
    //
    // Witness cluster: ~29 papers/stage in next_warning_papers (Stages
    // 15-20 v3) hit this on biblatex bbl `\verb 10.1162/EVCO_a_00133`.
    let mut value_vec: Vec<Token> = Vec::with_capacity(end - start);
    for tok in &body_toks[start..end] {
      match tok.code {
        Catcode::CS => {
          // Spell out the CS name as OTHER chars, dropping the
          // backslash: `\href` → `href`. (Matches the visible
          // rendering of a verbatim URL.)
          tok.with_str(|s| {
            let name = s.strip_prefix('\\').unwrap_or(s);
            for ch in name.chars() {
              value_vec.push(T_OTHER!(&ch.to_string()));
            }
          });
        },
        Catcode::BEGIN | Catcode::END |
        Catcode::SUB | Catcode::SUPER | Catcode::PARAM |
        Catcode::ALIGN | Catcode::MATH | Catcode::ACTIVE => {
          value_vec.push(Token {
            text: tok.text,
            code: Catcode::OTHER,
            #[cfg(feature = "token-locators")]
            loc: 0,
          });
        },
        _ => value_vec.push(*tok),
      }
    }
    let value = Tokens::new(value_vec);
    bib_entry_set_tokens(key_str.trim(), value);
    Ok(Tokens::default())
  }, locked => true);
  // \biblatex@endverb is consumed by the Until: delimiter on \biblatex@verb,
  // but if it ever fires standalone (degenerate input), no-op it.
  DefMacro!("\\biblatex@endverb", "", locked => true);

  // Perl L400-408: \addbibresource{file,...} pushes onto biblatex_resources.
  // Then `\biblatex@saved@bibliography` is bound to whatever `\bibliography`
  // means at this point (classic LaTeX bibtex), and `\bibliography` is
  // re-let to `\addbibresource` so any classic `\bibliography{...}`
  // invocation in a biblatex doc just records resources.
  // see arXiv:1502.02314 for a paper that left in classic \bibliography
  // alongside biblatex; both forms must end up populating the resource list.
  DefPrimitive!("\\addbibresource{}", sub[(file_list_arg)] {
    // Perl: split(/\s*,\s*/, ToString($_[1])) — split on commas and
    // strip surrounding whitespace.
    let raw = file_list_arg.to_string();
    for part in raw.split(',') {
      let file = part.trim();
      if !file.is_empty() {
        push_value("biblatex_resources", Stored::String(pin(file)))?;
      }
    }
  });
  Let!("\\biblatex@saved@bibliography", "\\bibliography");
  Let!("\\bibliography",                "\\addbibresource");

  // \printbibliography: biber's \jobname.bbl is ground truth when present
  // (read it directly through the \entry/\endentry rebuilder above);
  // otherwise fall back to \biblatex@printbibliography, which routes the
  // \addbibresource declarations through LaTeXML's \bibliography
  // machinery. The `&` catcode change lets literal ampersands in .bbl
  // author/publisher names through (restored to alignment after).
  // The optional argument ([heading=bibintoc] etc.) is consumed+ignored.
  DefMacro!("\\printbibliography[]",
    "\\let\\verb\\biblatex@verb\\let\\endverb\\biblatex@endverb\
     \\catcode`\\&=12\\relax\
     \\InputIfFileExists{\\jobname.bbl}{}{\\biblatex@printbibliography}\
     \\catcode`\\&=4\\relax");
  DefMacro!("\\biblatex@printbibliography[]", sub[(_opts)] {
    let mut resources = Vec::new();
    while let Some(res) = pop_value("biblatex_resources")? {
      if !resources.is_empty() {
        resources.push(T_OTHER!(","));
        resources.push(T_SPACE!());
      }
      resources.push(T_OTHER!(res.to_string()));
    }
    Ok(Tokens!(
      T_CS!("\\biblatex@saved@bibliography"),
      T_BEGIN!(),
      Tokens::new(resources),
      T_END!()
    ))
  }, locked => true);

  // Perl L420-424. Round-34 surpass: \xref{key} is a cross-reference,
  // route to \ref so it resolves. \warn / \fakeset are internal.
  def_macro_noop("\\warn{}")?;
  DefMacro!("\\xref{}", "\\ref{#1}");
  def_macro_noop("\\fakeset{}")?;

  // Perl L429-434: language API (no-ops)
  def_macro_noop("\\DeclareLanguageMapping{}{}")?;
  def_macro_noop("\\DeclareLanguageMappingSuffix{}")?;
  def_macro_noop("\\DefineHyphenationExceptions{}{}")?;
  def_macro_noop("\\DefineBibliographyExtras{}{}")?;
  def_macro_noop("\\UndefineBibliographyExtras{}{}")?;
  def_macro_noop("\\DefineBibliographyStrings{}{}")?;

  // Perl L436-438
  def_macro_noop("\\DeclareNameFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareListFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareFieldFormat OptionalMatch:* []{}{}")?;

  // Perl L440-458
  def_macro_noop("\\DeclareNameInputHandler{}{}")?;
  def_macro_noop("\\DeclareListInputHandler{}{}")?;
  def_macro_noop("\\DeclareFieldInputHandler{}{}")?;
  def_macro_noop("\\DeclareSortingScheme[]{}")?;
  def_macro_noop("\\DeclareSortingTemplate[]{}")?;
  def_macro_noop("\\DeclareSortingNamekeyScheme[]{}")?;
  def_macro_noop("\\namepart[]{}")?;
  def_macro_noop("\\DeclareLabelalphaNameTemplate[]{}")?;
  def_macro_noop("\\DeclareNameAlias{}{}")?;
  def_macro_noop("\\DeclareIndexNameAlias{}{}")?;
  def_macro_noop("\\DeclareListAlias{}{}")?;
  def_macro_noop("\\DeclareIndexListAlias{}{}")?;
  def_macro_noop("\\DeclareFieldAlias{}{}")?;
  def_macro_noop("\\DeclareIndexFieldAlias{}{}")?;
  def_macro_noop("\\DeclareNameWrapperAlias{}{}")?;
  def_macro_noop("\\DeclareListWrapperAlias{}{}")?;
  def_macro_noop("\\DeclareDelimcontextAlias{}{}")?;
  def_macro_noop("\\UndeclareDelimcontextAlias{}")?;
  def_macro_noop("\\DeclareCiteCommand OptionalMatch:* {}[]{}{}{}{}")?;

  // Perl L460-481
  def_macro_noop("\\DeclareBibliographyExtras{}")?;
  def_macro_noop("\\DeclareBibliographyStrings{}")?;
  def_macro_noop("\\DeclareBibliographyDriver{}{}")?;
  def_macro_noop("\\DeclareHyphenationExceptions{}")?;
  def_macro_noop("\\InheritBibliographyExtras{}")?;
  def_macro_noop("\\InheritBibliographyStrings{}")?;
  def_macro_noop("\\UndeclareBibliographyExtras{}")?;
  DefMacro!("\\NewCount", "\\newcount");
  def_macro_noop("\\ExecuteBibliographyOptions[]{}")?;
  def_macro_noop("\\AtBeginBibliography{}")?;
  def_macro_noop("\\AtEveryEntrykey{}{}{}")?;
  def_macro_noop("\\UseBibitemHook")?;
  def_macro_noop("\\UseUsedriverHook")?;
  def_macro_noop("\\UseEveryCiteHook")?;
  def_macro_noop("\\UseEveryCitekeyHook")?;
  def_macro_noop("\\UseEveryMultiCiteHook")?;
  def_macro_noop("\\UseNextCiteHook")?;
  def_macro_noop("\\UseNextCitekeyHook")?;
  def_macro_noop("\\UseNextMultiCiteHook")?;
  def_macro_noop("\\UseVolciteHook")?;
  def_macro_noop("\\DeferNextCitekeyHook")?;

  // Perl L483-491: bibmacro/heading/environment helpers
  def_macro_noop("\\providebibmacro OptionalMatch:* {}[][]{}")?;
  def_macro_noop("\\renewbibmacro OptionalMatch:* {}[][]{}")?;
  def_macro_noop("\\newbibmacro OptionalMatch:* {}[][]{}")?;
  def_macro_noop("\\restorebibmacro OptionalMatch:* {}")?;
  def_macro_noop("\\savebibmacro OptionalMatch:* {}")?;
  def_macro_noop("\\defbibheading OptionalMatch:* {}[]{}")?;
  def_macro_noop("\\defbibenvironment OptionalMatch:* {}{}{}{}")?;
  def_macro_noop("\\restorecommand OptionalMatch:* {}")?;
  def_macro_noop("\\savecommand OptionalMatch:* {}")?;

  // Perl L493-500
  DefRegister!("\\labelnumberwidth" => Glue!("0pt"));
  DefRegister!("\\labelalphawidth" => Glue!("0pt"));
  DefRegister!("\\biblabelsep" => Glue!("0pt"));
  DefRegister!("\\bibnamesep" => Glue!("0pt"));
  DefRegister!("\\bibitemsep" => Glue!("0pt"));
  DefRegister!("\\bibinitsep" => Glue!("0pt"));
  DefRegister!("\\bibparsep" => Glue!("0pt"));
  DefRegister!("\\bibhang" => Glue!("0pt"));
  // \lositemsep — itemsep length for biblatex's "list of shorthands" (los).
  // Declared `\newlength{\lositemsep}` in the biblatex-chicago bibliography
  // styles (chicago-notes.bbx L22, chicago-authordate.bbx, …) and `\setlength`
  // by biblatex-chicago.sty L154. Our biblatex binding doesn't implement the
  // `style=`-option `.bbx`/`.cbx` style-file load (Perl raw-loads the whole
  // chain), so `\lositemsep` was undefined when biblatex-chicago.sty sets it →
  // `\setlength` error + `<variable> expected` cascade. Provide the length
  // defensively (biblatex-chicago always loads biblatex). Witness 1802.09944
  // (`\usepackage[notes,backend=biber]{biblatex-chicago}`).
  DefRegister!("\\lositemsep" => Glue!("0pt"));

  // Perl L553-604: 50 conditionals
  DefConditional!("\\ifandothers");
  DefConditional!("\\ifbibindex");
  DefConditional!("\\ifbibliography");
  DefConditional!("\\ifbibstring");
  DefConditional!("\\ifcapital");
  DefConditional!("\\ifcategory");
  DefConditional!("\\ifcitation");
  DefConditional!("\\ifciteibid");
  DefConditional!("\\ifciteidem");
  DefConditional!("\\ifciteindex");
  DefConditional!("\\ifciteseen");
  DefConditional!("\\ifcurrentfield");
  DefConditional!("\\ifcurrentlist");
  DefConditional!("\\ifcurrentname");
  DefConditional!("\\ifentrycategory");
  DefConditional!("\\ifentrykeyword");
  DefConditional!("\\ifentryseen");
  DefConditional!("\\ifentrytype");
  DefConditional!("\\iffieldbibstring");
  DefConditional!("\\iffieldequalcs");
  DefConditional!("\\iffieldequals");
  DefConditional!("\\iffieldequalstr");
  DefConditional!("\\iffieldint");
  DefConditional!("\\iffieldnum");
  DefConditional!("\\iffieldnums");
  DefConditional!("\\iffieldpages");
  DefConditional!("\\iffieldsequal");
  DefConditional!("\\iffieldundef");
  DefConditional!("\\iffirstinits");
  DefConditional!("\\iffirstonpage");
  DefConditional!("\\iffootnote");
  DefConditional!("\\ifhyperref");
  DefConditional!("\\ifinteger");
  DefConditional!("\\ifkeyword");
  DefConditional!("\\ifloccit");
  DefConditional!("\\ifmoreitems");
  DefConditional!("\\ifmorenames");
  DefConditional!("\\ifnameequalcs");
  DefConditional!("\\ifnameequals");
  DefConditional!("\\ifnamesequal");
  DefConditional!("\\ifnameundef");
  DefConditional!("\\ifnatbibmode");
  DefConditional!("\\ifnumeral");
  DefConditional!("\\ifnumerals");
  DefConditional!("\\ifopcit");
  DefConditional!("\\ifpages");
  DefConditional!("\\ifsamepage");
  DefConditional!("\\ifsingletitle");
  DefConditional!("\\ifuseauthor");
  DefConditional!("\\ifuseeditor");
  DefConditional!("\\ifuseprefix");
  DefConditional!("\\ifusetranslator");

  // Perl L608-610 gobbles \key / \keyword silently. Round-34
  // surpass-Perl: preserve as classification tags so author keywords
  // reach the JATS output. \key{citekey} is bib-formatting internal —
  // leave that gobbled.
  def_macro_noop("\\key{}")?;
  // \keyw is already defined L348 (DefMacro empty, see above).
  DefMacro!("\\keyword{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}");

  // Perl L632-635
  DefMacro!("\\ppspace", "\\addnbspace");
  DefMacro!("\\sqspace", "\\addnbspace");
  DefMacro!("\\labelalphaothers", "+");
  DefMacro!("\\sortalphaothers", "\\labelalphaothers");

  // Perl L638
  def_macro_noop("\\sort[]{}")?;

  // Perl L641-645: bool stubs + AtBeginDocument-guarded \true/\false bind.
  // documents such as 1811.01740 conflict with unconditional binding.
  DefMacro!("\\blx@bbl@booltrue{}",  "\\relax", locked => true);
  DefMacro!("\\blx@bbl@boolfalse{}", "\\relax", locked => true);
  at_begin_document(TokenizeInternal!(
    r"\@ifundefined{true}{\let\true\blx@bbl@booltrue}{}\@ifundefined{false}{\let\false\blx@bbl@boolfalse}{}"
  ))?;

  // Perl L646-671: \the* counter-readouts (all empty)
  def_macro_noop("\\type{}")?;
  def_macro_noop("\\subtype{}")?;
  def_macro_noop("\\theparenlevel")?;
  def_macro_noop("\\therefsection")?;
  def_macro_noop("\\therefsegment")?;
  def_macro_noop("\\theuniquelist")?;
  def_macro_noop("\\theuniquename")?;
  def_macro_noop("\\themulticitecount")?;
  def_macro_noop("\\themulticitetotal")?;
  def_macro_noop("\\thelownamepenalty")?;
  def_macro_noop("\\themaxextraalpha")?;
  def_macro_noop("\\themaxextrayear")?;
  def_macro_noop("\\themaxitems")?;
  def_macro_noop("\\themaxnames")?;
  def_macro_noop("\\themaxparens")?;
  def_macro_noop("\\theminitems")?;
  def_macro_noop("\\theminnames")?;
  def_macro_noop("\\theabbrvpenalty")?;
  def_macro_noop("\\thecitecount")?;
  def_macro_noop("\\thecitetotal")?;
  def_macro_noop("\\thehighnamepenalty")?;
  def_macro_noop("\\theinstcount")?;
  def_macro_noop("\\thelistcount")?;
  def_macro_noop("\\theliststart")?;
  def_macro_noop("\\theliststop")?;
  def_macro_noop("\\thelisttotal")?;

  // Perl L673-688: print*/index*/entry* (all empty)
  def_macro_noop("\\printtext[]{}")?;
  def_macro_noop("\\printfield[]{}")?;
  def_macro_noop("\\printlist[][]{}")?;
  def_macro_noop("\\printnames[][]{}")?;
  def_macro_noop("\\printtime")?;
  def_macro_noop("\\printdate")?;
  def_macro_noop("\\printdateextra")?;
  def_macro_noop("\\printlabeldate")?;
  def_macro_noop("\\printlabeldateextra")?;
  def_macro_noop("\\printfile[]{}")?;
  def_macro_noop("\\indexfield[]{}")?;
  def_macro_noop("\\indexlist[][]{}")?;
  def_macro_noop("\\indexnames[][]{}")?;
  def_macro_noop("\\entrydata OptionalMatch:* {}{}")?;
  def_macro_noop("\\entryset{}{}")?;
  def_macro_noop("\\setunit OptionalMatch:* {}")?;

  // Perl L690-705
  def_macro_noop("\\mkbibendnote{}")?;
  def_macro_noop("\\mkbibendnotetext{}")?;
  DefMacro!("\\mkbibfootnote", "\\footnote");
  DefMacro!("\\mkbibfootnotetext", "\\footnotetext");
  DefMacro!("\\mkbibbrackets{}", "\\begingroup\\bibopenbracket#1\\bibclosebracket\\endgroup");
  DefMacro!("\\bibopenparen", "\\bibleftparen");
  DefMacro!("\\bibcloseparen", "\\bibrightparen");
  DefMacro!("\\bibopenbracket", "\\bibleftbracket");
  DefMacro!("\\bibclosebracket", "\\bibrightbracket");
  DefMacro!("\\bibleftparen", "\\blx@postpunct(");
  DefMacro!("\\bibrightparen", "\\blx@postpunct)\\midsentence");
  DefMacro!("\\bibleftbracket", "\\blx@postpunct[");
  DefMacro!("\\bibrightbracket", "\\blx@postpunct]\\midsentence");
  // Perl L704: redefine \blx@postpunct to \relax (overrides L66 \@empty).
  DefMacro!("\\blx@postpunct", "\\relax");
  DefMacro!("\\midsentence", "\\relax");

  // Perl L707-708 gobble; surpass by preserving as endnote-style.
  // \pagenote{text} is author-typed marginal note.
  DefMacro!("\\pagenote{}",
    "\\@add@frontmatter{ltx:note}[role=pagenote]{#1}");
  DefMacro!("\\pagenotetext{}",
    "\\@add@frontmatter{ltx:note}[role=pagenote-text]{#1}");

  // Perl L710-721
  DefMacro!("\\blx@uniquename", "false");
  DefMacro!("\\blx@uniquelist", "false");
  DefMacro!("\\blx@maxbibnames", "0");
  DefMacro!("\\blx@minbibnames", "0");
  DefMacro!("\\blx@maxcitenames", "0");
  DefMacro!("\\blx@mincitenames", "0");
  DefMacro!("\\blx@maxsortnames", "0");
  DefMacro!("\\blx@minsortnames", "0");
  DefMacro!("\\blx@maxalphanames", "0");
  DefMacro!("\\blx@minalphanames", "0");
  DefMacro!("\\blx@maxitems", "0");
  DefMacro!("\\blx@minitems", "0");

  // Perl L724-734: blx-internal counter registers
  DefRegister!("\\blx@tempcnta" => Number(0));
  DefRegister!("\\blx@tempcntb" => Number(0));
  DefRegister!("\\blx@tempcntc" => Number(0));
  DefRegister!("\\blx@maxsection" => Number(0));
  DefRegister!("\\blx@notetype" => Number(0));
  DefRegister!("\\blx@parenlevel@text" => Number(0));
  DefRegister!("\\blx@parenlevel@foot" => Number(0));
  // Note: `\blx@maxsegment@0` and `\blx@sectionciteorder@0` are CS names
  // with a trailing digit, which the prototype parser's CS regex
  // (`\\[a-zA-Z@]+`) cannot match — leftover `0` would then be parsed as
  // an unknown parameter type. Use the `(cs, None, value)` form so the
  // parser is skipped: a Token is built directly via `T_CS!` and no
  // parameter parsing occurs. Mirrors Perl's L731-732 register names exactly.
  DefRegister!(T_CS!("\\blx@maxsegment@0"), None, Number(0));
  DefRegister!(T_CS!("\\blx@sectionciteorder@0"), None, Number(0));
  DefRegister!("\\blx@entrysetcounter" => Number(0));
  DefRegister!("\\blx@biblioinstance" => Number(0));

  // Perl L736-801: trailing RawTeX with 9 \newbool + 60 \newtoggle
  // declarations. EXACT order and content from the Perl source.
  //
  // Use `\providetoggle` (etoolbox's define-if-absent form) rather than
  // `\newtoggle` for the toggle allocations: when a paper bundles a
  // `mybiblatex.sty`-style wrapper that re-enters biblatex's init (the
  // `_loaded` guard covers only a direct second `\usepackage{biblatex}`,
  // not every re-entry path), this block runs twice and `\newtoggle`
  // hard-errors on an already-defined toggle (`Package etoolbox Error:
  // Toggle 'blx@…' already defined`), 57× per re-entry. The `\newbool`
  // (=`\newif`) half is already tolerated (redefinition downgraded to
  // Info), so only the toggle half surfaced as errors. Allocating these
  // toggles is idempotent (same names, no carried state), and
  // `\providetoggle` is etoolbox's own re-entrant allocator — so this
  // stays faithful (Perl's `\newtoggle` never re-enters because Perl
  // can't find the bundled wrapper). Witness 2007.06815
  // (`\usepackage{mybiblatex}` → 55 toggle errors → 0).
  RawTeX!(r#"
\newbool{refcontextdefaults}
\booltrue{refcontextdefaults}%
\newbool{sourcemap}
\newbool{citetracker}
\newbool{pagetracker}
\newbool{backtracker}
\newbool{citerequest}
\booltrue{citerequest}
\newbool{sortcites}
\providetoggle{blx@bbldone}
\providetoggle{blx@tempa}
\providetoggle{blx@tempb}
\providetoggle{blx@runltx}
\providetoggle{blx@runbiber}
\providetoggle{blx@block}
\providetoggle{blx@unit}
\providetoggle{blx@skipentry}
\providetoggle{blx@insert}
\providetoggle{blx@lastins}
\providetoggle{blx@keepunit}
\providetoggle{blx@bibtex}
\providetoggle{blx@debug}
\providetoggle{blx@sortcase}
\providetoggle{blx@sortupper}
\providetoggle{blx@autolangbib}
\providetoggle{blx@autolangcite}
\providetoggle{blx@clearlang}
\providetoggle{blx@defernumbers}
\providetoggle{blx@omitnumbers}
\providetoggle{blx@footnote}
\providetoggle{blx@labelalpha}
\providetoggle{blx@labelnumber}
\providetoggle{blx@labeltitle}
\providetoggle{blx@labeltitleyear}
\providetoggle{blx@labeldateparts}
\providetoggle{blx@natbib}
\providetoggle{blx@mcite}
\providetoggle{blx@loadfiles}
\providetoggle{blx@sortsets}
\providetoggle{blx@crossrefsource}
\providetoggle{blx@xrefsource}
\providetoggle{blx@terseinits}
\providetoggle{blx@useprefix}
\providetoggle{blx@addset}
\providetoggle{blx@setonly}
\providetoggle{blx@dataonly}
\providetoggle{blx@skipbib}
\providetoggle{blx@skipbiblist}
\providetoggle{blx@skiplab}
\providetoggle{blx@citation}
\providetoggle{blx@volcite}
\providetoggle{blx@bibliography}
\providetoggle{blx@citeindex}
\providetoggle{blx@bibindex}
\providetoggle{blx@localnumber}
\providetoggle{blx@refcontext}
\providetoggle{blx@noroman}
\providetoggle{blx@nohashothers}
\providetoggle{blx@nosortothers}
\providetoggle{blx@singletitle}
\providetoggle{blx@uniquebaretitle}
\providetoggle{blx@uniqueprimaryauthor}
\providetoggle{blx@uniquetitle}
\providetoggle{blx@uniquework}
"#);

  // biblatex internals commonly invoked by user preamble. Witnesses
  // 2406.10485 (\newrefcontext), 2406.01081 (\newrefsection).
  def_macro_noop("\\newrefsection[]")?;
  def_macro_noop("\\newrefcontext[]")?;
  def_macro_noop("\\endrefcontext")?;
  def_macro_noop("\\refsection[]{}")?;
  def_macro_noop("\\endrefsection")?;
  def_macro_noop("\\refcontext[]{}")?;

  // biblatex L3408+ bibliography range separators. Define defensively.
  // NB: do NOT redefine \bibrangedash here — Perl L353 sets it to an en-dash
  // (U+2013), as do we above; a hyphen override would diverge from both Perl
  // and real biblatex. The date/time range separators are en-dashes too.
  def_macro_noop("\\bibrangessep")?;
  DefMacro!("\\bibdaterangesep", "\u{2013}");
  DefMacro!("\\bibtimerangesep", "\u{2013}");
});
