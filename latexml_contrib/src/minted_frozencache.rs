//! Pygments-color support for `minted` from a committed `_minted/` frozencache
//! (surpass-Perl; `docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md` #157).
//!
//! Perl LaTeXML has no `minted` binding and errors out on the environment; our
//! binding already routes minted through the `listings` substrate (bold-black
//! keywords, no color). When a paper is built with
//! `\usepackage[frozencache]{minted}`, the `_minted/` directory (a sibling of
//! the main `.tex`) carries Pygments' output on disk as plain `\textcolor`:
//!
//! * `default.style.minted` — `\@namedef{PYG@tok@<class>}{…}` per token class,
//!   each setting some of a color (`\def\PYG@tc##1{\textcolor[rgb]{r,g,b}{##1}}`),
//!   bold (`\let\PYG@bf=\textbf`), italic (`\let\PYG@it=\textit`).
//! * `<MD5>.highlight.minted` — one per highlighted snippet: a `MintedVerbatim`
//!   body of `\PYG{<tokclass>}{<text>}` runs interleaved with literal spaces and
//!   `\PYGZ*` escapes.
//!
//! Rather than replicate minted's fragile MD5 keying, we **content-match**: each
//! highlight file, with `\PYG` unwrapped and `\PYGZ*` resolved, yields the exact
//! plain code of a source snippet. We normalize a minted block's raw body the
//! same way and look it up. On a hit we emit Pygments-colored listing lines
//! (reusing the listings `\@lst@startline`/`\@lst@endline`/`\@listingGroup`
//! constructors and xcolor's `\textcolor`); on a miss the caller keeps the exact
//! current (uncolored) `listings` path. When no `_minted/` exists the whole
//! feature is a no-op, so non-frozencache papers are unaffected.
//!
//! Reading the host source tree's `_minted/` is in scope (it is like reading a
//! `.sty`, per CLAUDE.md — the ban is on reading latexml-oxide's *own* resources).
//!
//! Witness: arXiv:2605.03143 (`\begin{minted}{ocaml|python}` blocks in
//! `sections/01-introduction.tex`, `02-a-taste-of-pact.tex`, `03-memo.tex`).

use std::{cell::RefCell, path::Path};

use latexml_core::binding::content::find_file;
use latexml_package::prelude::*;

/// A resolved token style: an optional `rgb` triple string (e.g.
/// `"0.00,0.50,0.00"`, passed verbatim to `\textcolor[rgb]{…}`), plus bold /
/// italic flags. Mirrors what `default.style.minted`'s `PYG@tok@<class>` sets.
#[derive(Clone, Default)]
struct Style {
  rgb:    Option<String>,
  bold:   bool,
  italic: bool,
}

/// One `\PYG{tok}{text}` run (or a literal run with `tok = None`).
struct Seg {
  tok:  Option<String>,
  text: String,
}

type Line = Vec<Seg>;

/// A loaded `_minted/` cache, keyed on the resolved directory path so a later
/// document on the same thread (a different `_minted/`, or none) never reuses it.
struct CacheData {
  dir:          String,
  styles:       HashMap<String, Style>,
  /// normalized plaincode → the highlighted lines that produced it.
  by_plaincode: HashMap<String, Vec<Line>>,
}

thread_local! {
  static CACHE: RefCell<Option<Rc<CacheData>>> = const { RefCell::new(None) };
}

/// Map `\PYGZxx` escape names to the literal character they stand for
/// (`default.style.minted` L81-99).
fn pygz_char(name: &str) -> Option<&'static str> {
  Some(match name {
    "PYGZbs" => "\\",
    "PYGZus" => "_",
    "PYGZob" => "{",
    "PYGZcb" => "}",
    "PYGZca" => "^",
    "PYGZam" => "&",
    "PYGZlt" => "<",
    "PYGZgt" => ">",
    "PYGZsh" => "#",
    "PYGZpc" => "%",
    "PYGZdl" => "$",
    "PYGZhy" => "-",
    "PYGZsq" => "'",
    "PYGZdq" => "\"",
    "PYGZti" => "~",
    "PYGZat" => "@",
    "PYGZlb" => "[",
    "PYGZrb" => "]",
    _ => return None,
  })
}

/// Resolve every `\PYGZxx` / `\PYGZxx{}` occurrence in `s` to its literal char.
fn resolve_pygz(s: &str) -> String {
  static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\(PYGZ[a-z][a-z])(\{\})?").unwrap());
  RE.replace_all(s, |c: &regex::Captures| pygz_char(&c[1]).unwrap_or(""))
    .into_owned()
}

/// Read a `{…}`-balanced group. `s` must start with `{`. Returns
/// `(inner, rest_after_close)`. `\PYGZob{}`/`\PYGZcb{}` contribute balanced
/// `{}` pairs, so brace counting stays correct.
fn read_group(s: &str) -> Option<(String, &str)> {
  let mut depth = 0i32;
  for (idx, ch) in s.char_indices() {
    match ch {
      '{' => depth += 1,
      '}' => {
        depth -= 1;
        if depth == 0 {
          return Some((s[1..idx].to_string(), &s[idx + 1..]));
        }
      },
      _ => {},
    }
  }
  None
}

/// Parse one raw highlight line into `\PYG{tok}{text}` and literal segments.
fn parse_line(line: &str) -> Line {
  let mut segs: Line = Vec::new();
  let mut lit = String::new();
  let mut rest = line;
  while !rest.is_empty() {
    if let Some(after) = rest.strip_prefix("\\PYG{") {
      // `tok}{content}` — tok never contains a brace.
      if let Some(tok_end) = after.find('}') {
        let tok = &after[..tok_end];
        let after_tok = &after[tok_end + 1..];
        if let Some((content, rest2)) = read_group(after_tok) {
          if !lit.is_empty() {
            segs.push(Seg {
              tok:  None,
              text: std::mem::take(&mut lit),
            });
          }
          segs.push(Seg {
            tok:  Some(tok.to_string()),
            text: resolve_pygz(&content),
          });
          rest = rest2;
          continue;
        }
      }
      // Malformed \PYG — treat the backslash literally and move on.
      lit.push('\\');
      rest = &rest[1..];
    } else if let Some(after) = rest.strip_prefix("\\PYGZ") {
      // A bare escape outside \PYG (e.g. in an escapeinside remnant).
      if after.len() >= 2 {
        let name = format!("PYGZ{}", &after[..2]);
        if let Some(ch) = pygz_char(&name) {
          lit.push_str(ch);
          rest = &after[2..];
          rest = rest.strip_prefix("{}").unwrap_or(rest);
          continue;
        }
      }
      lit.push('\\');
      rest = &rest[1..];
    } else {
      let c = rest.chars().next().unwrap();
      lit.push(c);
      rest = &rest[c.len_utf8()..];
    }
  }
  if !lit.is_empty() {
    segs.push(Seg { tok: None, text: lit });
  }
  segs
}

/// The concatenated plain text of a line.
fn line_text(line: &Line) -> String { line.iter().map(|s| s.text.as_str()).collect() }

/// Drop leading/trailing all-whitespace lines (in place).
fn strip_blank_edges(lines: &mut Vec<Line>) {
  while lines
    .first()
    .is_some_and(|l| line_text(l).trim().is_empty())
  {
    lines.remove(0);
  }
  while lines.last().is_some_and(|l| line_text(l).trim().is_empty()) {
    lines.pop();
  }
}

/// Normalized-plaincode key for a set of highlight lines: each line's trailing
/// whitespace trimmed, joined by `\n` (blank edges already stripped).
fn lines_key(lines: &[Line]) -> String {
  lines
    .iter()
    .map(|l| line_text(l).trim_end().to_string())
    .collect::<Vec<_>>()
    .join("\n")
}

/// Normalize a minted block's raw source body to the same key form: rstrip each
/// line, drop leading/trailing blank lines.
fn normalize_source(text: &str) -> String {
  let mut v: Vec<String> = text.split('\n').map(|l| l.trim_end().to_string()).collect();
  while v.first().is_some_and(|l| l.is_empty()) {
    v.remove(0);
  }
  while v.last().is_some_and(|l| l.is_empty()) {
    v.pop();
  }
  v.join("\n")
}

/// Extract the `MintedVerbatim` body of a highlight file → its lines.
fn parse_highlight(text: &str) -> Option<Vec<Line>> {
  let begin = text.find("\\begin{MintedVerbatim}")?;
  let body_start = text[begin..].find('\n')? + begin + 1;
  let end = text.rfind("\\end{MintedVerbatim}")?;
  if end < body_start {
    return None;
  }
  let body = &text[body_start..end];
  let mut lines: Vec<Line> = body.split('\n').map(parse_line).collect();
  strip_blank_edges(&mut lines);
  Some(lines)
}

/// Parse `default.style.minted` into a `tokclass → Style` map.
fn parse_styles(text: &str) -> HashMap<String, Style> {
  static NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\@namedef\{PYG@tok@([A-Za-z0-9]+)\}").unwrap());
  static COLOR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\textcolor\[rgb\]\{([0-9.,]+)\}").unwrap());
  let mut map: HashMap<String, Style> = HashMap::default();
  for line in text.lines() {
    if let Some(caps) = NAME_RE.captures(line) {
      let class = caps[1].to_string();
      let style = Style {
        rgb:    COLOR_RE.captures(line).map(|c| c[1].to_string()),
        bold:   line.contains(r"\PYG@bf=\textbf"),
        italic: line.contains(r"\PYG@it=\textit"),
      };
      map.insert(class, style);
    }
  }
  map
}

/// Load (and memoize) the `_minted/` cache for the current document. Keyed on
/// the resolved directory, so repeated blocks reuse it and a different document
/// reloads. Returns `None` when no frozencache is present.
fn load_cache() -> Option<Rc<CacheData>> {
  let style_path = find_file("_minted/default.style.minted", None)?;
  let dir = Path::new(&style_path)
    .parent()?
    .to_string_lossy()
    .into_owned();

  if let Some(cached) = CACHE.with(|c| c.borrow().as_ref().filter(|d| d.dir == dir).cloned()) {
    return Some(cached);
  }

  let styles = std::fs::read_to_string(&style_path)
    .ok()
    .map(|s| parse_styles(&s))?;
  let mut by_plaincode: HashMap<String, Vec<Line>> = HashMap::default();
  if let Ok(entries) = std::fs::read_dir(&dir) {
    for entry in entries.flatten() {
      let name = entry.file_name();
      let name = name.to_string_lossy();
      if !name.ends_with(".highlight.minted") {
        continue;
      }
      if let Ok(text) = std::fs::read_to_string(entry.path())
        && let Some(lines) = parse_highlight(&text)
        && !lines.is_empty()
      {
        by_plaincode.entry(lines_key(&lines)).or_insert(lines);
      }
    }
  }

  let data = Rc::new(CacheData { dir, styles, by_plaincode });
  CACHE.with(|c| *c.borrow_mut() = Some(data.clone()));
  Some(data)
}

/// True when a `_minted/` frozencache is present for the current document.
pub fn cache_available() -> bool { load_cache().is_some() }

/// Resolve a (possibly compound, e.g. `n+nf`, `l+m+mf`) token class to a style.
/// Mirrors `\PYG@toks`: each `+`-separated sub-class is applied in order — color
/// takes the LAST sub-class that sets one, bold/italic accumulate.
fn resolve_style(cache: &CacheData, tok: &Option<String>) -> Style {
  let mut st = Style::default();
  if let Some(t) = tok {
    for sub in t.split('+') {
      if let Some(s) = cache.styles.get(sub) {
        if s.rgb.is_some() {
          st.rgb = s.rgb.clone();
        }
        st.bold |= s.bold;
        st.italic |= s.italic;
      }
    }
  }
  st
}

/// Map a single code character to its emit tokens: TeX-special chars → the
/// text-command escape (mirrors listings' `lst_char_mapping`, upquote off);
/// everything else → a literal `OTHER` char (as the listings char path does).
fn map_char(c: char) -> Vec<Token> {
  let cs: &'static str = match c {
    '#' => "\\#",
    '$' => "\\textdollar",
    '&' => "\\&",
    '\'' => "\\textquoteright",
    '*' => "\\textasteriskcentered",
    '<' => "\\textless",
    '>' => "\\textgreater",
    '\\' => "\\textbackslash",
    '^' => "\\textasciicircum",
    '_' => "\\textunderscore",
    '`' => "\\textquoteleft",
    '{' => "\\textbraceleft",
    '}' => "\\textbraceright",
    '%' => "\\%",
    '|' => "\\textbar",
    '~' => "\\textasciitilde",
    _ => {
      let mut tmp = [0u8; 4];
      return vec![T_OTHER!(c.encode_utf8(&mut tmp))];
    },
  };
  vec![T_CS!(cs)]
}

/// Emit a run of `n` spaces as a `white-space:pre` group, exactly like the
/// listings space path (`\@listingGroup{ltx_lst_space}{ … }` with control-space
/// tokens that survive TeX space-collapsing).
fn emit_space_run(n: usize, out: &mut Vec<Token>) {
  out.push(T_CS!("\\@listingGroup"));
  out.push(T_BEGIN!());
  out.extend(ExplodeText!("ltx_lst_space"));
  out.push(T_END!());
  out.push(T_BEGIN!());
  for _ in 0..n {
    out.push(T_CS!(" "));
  }
  out.push(T_END!());
}

/// Emit the characters of a text run, wrapping maximal space runs in a
/// `ltx_lst_space` group so indentation and interior spacing are preserved
/// (`.ltx_listingline` is `white-space:nowrap`, which otherwise collapses runs).
fn emit_text(text: &str, out: &mut Vec<Token>) {
  let mut chars = text.chars().peekable();
  while let Some(c) = chars.next() {
    if c == ' ' || c == '\t' {
      let mut n = 1usize;
      while matches!(chars.peek(), Some(' ') | Some('\t')) {
        chars.next();
        n += 1;
      }
      emit_space_run(n, out);
    } else {
      out.extend(map_char(c));
    }
  }
}

/// Wrap `inner` in `\cs{…}`.
fn wrap_cs(cs: &'static str, inner: Vec<Token>) -> Vec<Token> {
  let mut out = vec![T_CS!(cs), T_BEGIN!()];
  out.extend(inner);
  out.push(T_END!());
  out
}

/// Wrap `inner` in `\textcolor[rgb]{r,g,b}{…}`.
fn wrap_color(rgb: &str, inner: Vec<Token>) -> Vec<Token> {
  let mut out = vec![T_CS!("\\textcolor"), T_OTHER!("[")];
  out.extend(ExplodeText!("rgb"));
  out.push(T_OTHER!("]"));
  out.push(T_BEGIN!());
  out.extend(ExplodeText!(rgb));
  out.push(T_END!());
  out.push(T_BEGIN!());
  out.extend(inner);
  out.push(T_END!());
  out
}

/// Emit one segment: its text (with space runs preserved), wrapped in bold /
/// italic / color per its style. Order mirrors `\PYG@do`: color outside italic
/// outside bold. Pure-whitespace segments skip the style wrappers (a colored
/// space is invisible and only bloats the tree).
fn emit_seg(cache: &CacheData, seg: &Seg, out: &mut Vec<Token>) {
  let mut inner = Vec::new();
  emit_text(&seg.text, &mut inner);
  if seg.text.trim().is_empty() {
    out.extend(inner);
    return;
  }
  let style = resolve_style(cache, &seg.tok);
  if style.bold {
    inner = wrap_cs("\\textbf", inner);
  }
  if style.italic {
    inner = wrap_cs("\\textit", inner);
  }
  if let Some(rgb) = style.rgb.as_ref() {
    inner = wrap_color(rgb, inner);
  }
  out.extend(inner);
}

/// Build the per-line block body tokens (`\@lst@startline{}` … `\@lst@endline`
/// per line) for a set of highlighted lines.
fn build_block_body(cache: &CacheData, lines: &[Line]) -> Vec<Token> {
  let mut out = Vec::new();
  for line in lines {
    out.push(T_CS!("\\@lst@startline"));
    out.push(T_BEGIN!());
    out.push(T_END!()); // empty line-number tags
    for seg in line {
      emit_seg(cache, seg, &mut out);
    }
    out.push(T_CS!("\\@lst@endline"));
  }
  out
}

/// If `raw_body` (a minted block's raw source body) matches a frozencache
/// highlight file, return the Pygments-colored block body tokens (to hand to
/// `listings_sty::lst_process_display_with`). Otherwise `None`.
pub fn colored_display_body(raw_body: &str) -> Option<Vec<Token>> {
  let cache = load_cache()?;
  let key = normalize_source(raw_body);
  let lines = cache.by_plaincode.get(&key)?;
  Some(build_block_body(&cache, lines))
}

/// Inline body tokens for a `\mintinline` snippet, for use as the argument of
/// `\@listings@inline`. On a frozencache hit the segments are Pygments-colored;
/// on a miss (cache present but snippet not found) the code is emitted plain
/// (uncolored, but text/spacing preserved). Call only when [`cache_available`].
pub fn inline_body(raw_code: &str) -> Vec<Token> {
  let key = normalize_source(raw_code);
  if let Some(cache) = load_cache() {
    if let Some(lines) = cache.by_plaincode.get(&key) {
      let mut out = Vec::new();
      for (i, line) in lines.iter().enumerate() {
        if i > 0 {
          out.push(T_CS!(" "));
        }
        for seg in line {
          emit_seg(&cache, seg, &mut out);
        }
      }
      return out;
    }
    // Miss: emit plain (uncolored) text, spacing preserved.
    let mut out = Vec::new();
    emit_text(&normalize_source(raw_code), &mut out);
    return out;
  }
  let mut out = Vec::new();
  emit_text(raw_code, &mut out);
  out
}
