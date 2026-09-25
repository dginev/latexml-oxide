use latexml_engine::bibtex::{BibEntry, register_entry};
use latexml_package::prelude::*;

// === biblatex .bbl reader ===
// biber writes each entry of a `.bbl` as `\entry{key}{type}{options}` …
// `\endentry`, its data in `\field`/`\list`/`\name`/`\verb`/`\keyw` records
// and its truncated name lists as `\true{more<list>}` (biblatex.sty:7862-8360
// the record definitions, :8681 `\blx@bbl@entry`). The records are read back
// into the BibTeX entry they came from ([`BblEntry`]): the fields of
// biblatex's data model (blx-dm.def:473-634) under their own names, the date
// parts rejoined into the dates, the name and literal lists into BibTeX's
// "and" lists. Each entry is then digested by the BibTeX reader
// (`\ProcessBibTeXEntry`, bibtex.rs), so a `.bbl` yields the same
// `ltx:bibentry` as the `.bib` it was made from, and MakeBibliography formats
// the two alike. What biber adds is kept where biblatex prints it: the list
// order (biber applied the style's sorting, `sort='false'` on the
// bibliography) and the alphabetic label (`labelalpha`, `extraalpha`;
// alphabetic.bbx:22-25). The rest of biber's records — hashes, sort keys,
// `extra*` counters, `\range` — only drive biblatex's typesetting and are
// dropped. Beyond Perl: the ar5iv binding (biblatex.sty.ltxml:495-690
// `\entry`/`\endentry`, :705 `\name`, :821-842 `\verb`) rebuilds a
// `\bibitem` per entry from a dozen fields, dropping the rest, and suffixes
// colliding labels itself where biber's `extraalpha` now does (its
// z-wraparound, arXiv 1212.4446). Witnesses arXiv 2605.08378, 2605.10053,
// 2605.11417, 2605.14864, 2605.18215, 2605.21199, 2605.28832.

/// One `.bbl` entry being read, between `\entry` and `\endentry`.
struct BblEntry {
  key:        String,
  entry_type: String,
  /// Whether the bibliography lists it: not when its options say `skipbib`
  /// or `dataonly` (biblatex.sty:8715-8717; biber's clones of related
  /// entries are `dataonly`).
  listed:     bool,
  /// BibTeX fields in record order: `(name, BibTeX source)`.
  fields:     Vec<(String, String)>,
  /// The date parts (`year`, `urlmonth`, `eventendday`, …), rejoined into
  /// the dates at `\endentry` ([`bbl_date`]).
  date_parts: Vec<(String, String)>,
  /// The lists biber marks as cut short (`\true{moreauthor}`): they end in
  /// BibTeX's "and others".
  more:       Vec<String>,
}

/// The datalists of the refsection being read (biblatex.sty:9280-9287
/// `\blx@bbl@dlist[type]{name}`) and the entries it prints.
#[derive(Default)]
struct BblDatalists {
  /// The datalist being read: its type (`entry`; `list` for a biblist, the
  /// shorthands of `\printshorthands`) and its name.
  kind:  String,
  name:  String,
  /// Its entries, as each `\endentry` is read.
  keys:  Vec<String>,
  /// The entries the refsection prints ([`bbl_end_datalist`]), and whether
  /// they are the default refcontext's.
  print: Option<(Vec<String>, bool)>,
}

thread_local! {
  /// The entry the `.bbl` is reading; replaced at every `\entry`, taken at
  /// its `\endentry`.
  static BBL_ENTRY: std::cell::RefCell<Option<BblEntry>> = const { std::cell::RefCell::new(None) };
  /// The refsection's datalists; emptied by [`bbl_flush`] and at package load.
  static BBL_DATALISTS: std::cell::RefCell<BblDatalists> =
    std::cell::RefCell::new(BblDatalists::default());
}

/// The fields of biblatex's default data model (blx-dm.def:473-634) that a
/// `.bbl` carries as `\field` and `\verb` records — every field and verbatim
/// field, less the `skipout` ones biber never writes. The `date` fields come as
/// parts ([`BBL_DATES`]). `labelalpha` and `extraalpha` are biber's own: the
/// label an alphabetic style prints (alphabetic.bbx:22-25), which
/// MakeBibliography prints too.
const BBL_FIELDS: &[&str] = &[
  "sortyear",
  "volume",
  "volumes",
  "abstract",
  "addendum",
  "annotation",
  "booksubtitle",
  "booktitle",
  "booktitleaddon",
  "chapter",
  "edition",
  "eid",
  "entrysubtype",
  "eprintclass",
  "eprinttype",
  "eventtitle",
  "eventtitleaddon",
  "gender",
  "howpublished",
  "indexsorttitle",
  "indextitle",
  "isan",
  "isbn",
  "ismn",
  "isrn",
  "issn",
  "issue",
  "issuesubtitle",
  "issuetitle",
  "issuetitleaddon",
  "iswc",
  "journalsubtitle",
  "journaltitle",
  "journaltitleaddon",
  "label",
  "langid",
  "langidopts",
  "library",
  "mainsubtitle",
  "maintitle",
  "maintitleaddon",
  "nameaddon",
  "note",
  "number",
  "origtitle",
  "pagetotal",
  "part",
  "relatedstring",
  "relatedtype",
  "reprinttitle",
  "series",
  "shorthandintro",
  "subtitle",
  "title",
  "titleaddon",
  "usera",
  "userb",
  "userc",
  "userd",
  "usere",
  "userf",
  "venue",
  "version",
  "shorthand",
  "shortjournal",
  "shortseries",
  "shorttitle",
  "authortype",
  "editoratype",
  "editorbtype",
  "editorctype",
  "editortype",
  "bookpagination",
  "nameatype",
  "namebtype",
  "namectype",
  "pagination",
  "pubstate",
  "type",
  "crossref",
  "xref",
  "related",
  "keywords",
  "pages",
  "execute",
  "doi",
  "eprint",
  "file",
  "verba",
  "verbb",
  "verbc",
  "url",
  "labelalpha",
  "extraalpha",
];

/// The literal lists of the data model (blx-dm.def:552-564, :604).
const BBL_LISTS: &[&str] = &[
  "institution",
  "lista",
  "listb",
  "listc",
  "listd",
  "liste",
  "listf",
  "location",
  "organization",
  "origlocation",
  "origpublisher",
  "publisher",
  "language",
  "origlanguage",
];

/// The name lists of the data model (blx-dm.def:566-586).
const BBL_NAMES: &[&str] = &[
  "afterword",
  "annotator",
  "author",
  "bookauthor",
  "commentator",
  "editor",
  "editora",
  "editorb",
  "editorc",
  "foreword",
  "holder",
  "introduction",
  "namea",
  "nameb",
  "namec",
  "translator",
  "shortauthor",
  "shorteditor",
];

/// The date fields of the data model (blx-dm.def:610-614) by the prefix of
/// the parts biber writes for them: `year`/`month`/`day` and their `end*`
/// forms (`urlyear`, `eventendday`, …).
const BBL_DATES: &[(&str, &str)] = &[
  ("", "date"),
  ("event", "eventdate"),
  ("orig", "origdate"),
  ("url", "urldate"),
];

/// Run `f` on the entry being read, if any.
fn with_bbl_entry(f: impl FnOnce(&mut BblEntry)) {
  BBL_ENTRY.with(|slot| {
    if let Some(entry) = slot.borrow_mut().as_mut() {
      f(entry);
    }
  });
}

/// The BibTeX source of a `.bbl` value: its tokens as TeX, with biber's range
/// markup read back — `\bibrangedash` is the `--` and `\bibrangessep` the
/// ", " of a `pages` range list (biber writes "1\bibrangedash 10\bibrangessep
/// 15" for "1--10, 15").
fn bbl_source(value: Tokens) -> String {
  bbl_replace_macros(&value.untex(), &[
    ("bibrangedash", "--"),
    ("bibrangessep", ", "),
  ])
}

/// Replace the named control words of `text` (with the space that ends one).
fn bbl_replace_macros(text: &str, macros: &[(&str, &str)]) -> String {
  let mut out = String::with_capacity(text.len());
  let mut rest = text;
  while let Some(at) = rest.find('\\') {
    out.push_str(&rest[..at]);
    let tail = &rest[at + 1..];
    let len = tail
      .find(|c: char| !c.is_ascii_alphabetic())
      .unwrap_or(tail.len());
    match macros.iter().find(|(name, _)| *name == &tail[..len]) {
      Some((_, replacement)) if len > 0 => {
        out.push_str(replacement);
        rest = &tail[len..];
        rest = rest.strip_prefix(' ').unwrap_or(rest);
      },
      _ => {
        out.push('\\');
        rest = tail;
      },
    }
  }
  out.push_str(rest);
  out
}

/// A part of a name as BibTeX source: biblatex's delimiters (biblatex.def
/// `\bibnamedelima`-`d`, `\bibnamedelimi`, `\bibinitdelim`) are the spaces the
/// `.bib` had.
fn bbl_name_part(part: &str) -> String {
  let text = bbl_replace_macros(part, &[
    ("bibnamedelima", " "),
    ("bibnamedelimb", " "),
    ("bibnamedelimc", " "),
    ("bibnamedelimd", " "),
    ("bibnamedelimi", " "),
    ("bibinitdelim", " "),
    ("bibinithyphendelim", "-"),
    ("bibinitperiod", "."),
  ]);
  text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One name as BibTeX writes it, "von Last, Jr, First": the comma forms keep
/// a many-word family name one last name. A name with no given part is
/// braced when it has several words, so none of them reads as a first name.
fn bbl_bibtex_name(given: &str, prefix: &str, family: &str, suffix: &str) -> String {
  let family =
    if given.is_empty() && suffix.is_empty() && family.contains(' ') && !family.starts_with('{') {
      format!("{{{family}}}")
    } else {
      family.to_string()
    };
  let last = if prefix.is_empty() {
    family
  } else {
    format!("{prefix} {family}")
  };
  match (suffix.is_empty(), given.is_empty()) {
    (true, true) => last,
    (true, false) => format!("{last}, {given}"),
    (false, _) => format!("{last}, {suffix}, {given}"),
  }
}

/// The top-level `{…}` groups of `tokens`, spaces between them skipped.
fn bbl_groups(tokens: &[Token]) -> Vec<Vec<Token>> {
  let mut groups = Vec::new();
  let mut i = 0usize;
  while i < tokens.len() {
    if tokens[i].code != Catcode::BEGIN {
      i += 1;
      continue;
    }
    i += 1;
    let mut depth = 1usize;
    let mut group = Vec::new();
    while i < tokens.len() {
      match tokens[i].code {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            break;
          }
        },
        _ => {},
      }
      group.push(tokens[i]);
      i += 1;
    }
    i += 1;
    groups.push(group);
  }
  groups
}

/// A `.bbl` date, rejoined from biber's parts as biblatex's ISO form
/// "YYYY-MM-DD", a range "…/…" (an empty end year is an open range).
fn bbl_date(parts: &[(String, String)], prefix: &str) -> Option<String> {
  let get = |part: &str| {
    let name = format!("{prefix}{part}");
    parts
      .iter()
      .find(|(key, _)| *key == name)
      .map(|(_, value)| value.as_str())
  };
  let one = |year: &str, month: Option<&str>, day: Option<&str>| {
    let mut date = year.to_string();
    if let Some(month) = month.filter(|m| !m.is_empty()) {
      date.push_str(&format!("-{month:0>2}"));
      if let Some(day) = day.filter(|d| !d.is_empty()) {
        date.push_str(&format!("-{day:0>2}"));
      }
    }
    date
  };
  let mut date = one(get("year")?, get("month"), get("day"));
  if let Some(end) = get("endyear") {
    date.push('/');
    if !end.is_empty() {
      date.push_str(&one(end, get("endmonth"), get("endday")));
    }
  }
  Some(date)
}

/// Whether a datalist is the one `\printbibliography` prints: the default
/// refcontext's, `<sorting>/global//global/global[/global]` — every template
/// after the sorting scheme `global`, no label prefix. biber writes another
/// for each further refcontext (biblatex-apa's `nyt/apasortcite//…` sorts
/// citations).
fn bbl_default_datalist(name: &str) -> bool {
  name
    .split('/')
    .skip(1)
    .all(|part| part.is_empty() || part == "global")
}

/// End of a datalist: its entries are the ones the refsection prints if it is
/// the first entry datalist, or the first of the default refcontext
/// ([`bbl_default_datalist`]). A biblist (`[list]`) or an empty list is not.
fn bbl_end_datalist() {
  BBL_DATALISTS.with(|lists| {
    let mut lists = lists.borrow_mut();
    let keys = std::mem::take(&mut lists.keys);
    let kind = std::mem::take(&mut lists.kind);
    let is_default = bbl_default_datalist(&std::mem::take(&mut lists.name));
    if kind == "list" || keys.is_empty() {
      return;
    }
    match lists.print {
      None => lists.print = Some((keys, is_default)),
      Some((_, false)) if is_default => lists.print = Some((keys, true)),
      Some(_) => {},
    }
  });
}

/// The bibliography of the refsection read so far: a `ltx:bibliography` of
/// the chosen datalist's entries ([`bbl_end_datalist`]; entries outside any
/// datalist, in an old `.bbl`, are one list), each digested by the BibTeX
/// reader. Resets the refsection's state.
fn bbl_flush() -> Tokens {
  bbl_end_datalist();
  BBL_ENTRY.with(|entry| *entry.borrow_mut() = None);
  let Some((keys, _)) = BBL_DATALISTS.with(|lists| std::mem::take(&mut *lists.borrow_mut()).print)
  else {
    return Tokens::default();
  };
  assign_value("BIBSTYLE", pin(blx_bibstyle()), Some(Scope::Global));
  let mut tokens = vec![T_CS!("\\biblatex@bbl@thebibliography")];
  for key in keys {
    tokens.push(T_CS!("\\ProcessBibTeXEntry"));
    tokens.push(T_BEGIN!());
    tokens.extend(Explode!(&key));
    tokens.push(T_END!());
  }
  tokens.push(T_CS!("\\endthebibliography"));
  Tokens::new(tokens)
}

/// A datalist starts (`\datalist[type]{name}`, `\sortlist` in a format-2
/// `.bbl`); an absent type is an entry list.
fn bbl_start_datalist(kind: Option<Tokens>, name: Tokens) {
  BBL_DATALISTS.with(|lists| {
    let mut lists = lists.borrow_mut();
    lists.kind = kind
      .map(|kind| kind.to_string().trim().to_string())
      .unwrap_or_default();
    lists.name = name.to_string().trim().to_string();
    lists.keys.clear();
  });
}

/// Whether an `\entry`'s options (`skipbib`, `dataonly=true`, …) keep it out
/// of the bibliography (biblatex.sty:8715-8717 `blx@skipbib`; `dataonly` sets
/// it too).
fn bbl_skips_bib(options: &str) -> bool {
  options.split(',').any(|option| {
    let (name, value) = option.split_once('=').unwrap_or((option, "true"));
    matches!(name.trim(), "skipbib" | "dataonly") && value.trim() != "false"
  })
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

/// One `key=value` package option, split at the first `=` with both sides
/// trimmed and the value's outer braces removed — biblatex manuals write
/// `\usepackage[style = abnt, ...]{biblatex}` with spaces around `=`
/// (biblatex-abnt.tex:52), which a prefix match on `style=` never saw, so
/// `abnt.cbx` was never loaded (`\apud` undefined).
fn blx_opt_kv(opt: &str) -> Option<(String, String)> {
  let (k, v) = opt.split_once('=')?;
  let v = v.trim();
  let v = v
    .strip_prefix('{')
    .and_then(|v| v.strip_suffix('}'))
    .unwrap_or(v);
  Some((k.trim().to_string(), v.trim().to_string()))
}

#[rustfmt::skip]
/// The `.bbl` command set (biblatex.sty:8995-9024 `\\blx@bblstart`): saved
/// and rebound around the `.bbl` input, restored after it.
const BBL_START: &str = "\\let\\biblatex@saved@verb\\verb\\let\\verb\\biblatex@bbl@verb\\let\\biblatex@saved@endverb\\endverb\\let\\endverb\\biblatex@bbl@endverb\\let\\biblatex@saved@datalist\\datalist\\let\\datalist\\biblatex@bbl@datalist\\let\\biblatex@saved@enddatalist\\enddatalist\\let\\enddatalist\\biblatex@bbl@enddatalist\\let\\biblatex@saved@entry\\entry\\let\\entry\\biblatex@bbl@entry\\let\\biblatex@saved@endentry\\endentry\\let\\endentry\\biblatex@bbl@endentry\\let\\biblatex@saved@name\\name\\let\\name\\biblatex@bbl@name\\let\\biblatex@saved@list\\list\\let\\list\\biblatex@bbl@list\\let\\biblatex@saved@field\\field\\let\\field\\biblatex@bbl@field\\let\\biblatex@saved@strng\\strng\\let\\strng\\biblatex@bbl@strng\\let\\biblatex@saved@keyw\\keyw\\let\\keyw\\biblatex@bbl@keyw\\let\\biblatex@saved@range\\range\\let\\range\\biblatex@bbl@range\\let\\biblatex@saved@preamble\\preamble\\let\\preamble\\biblatex@bbl@preamble\\let\\biblatex@saved@warn\\warn\\let\\warn\\biblatex@bbl@warn\\let\\biblatex@saved@xref\\xref\\let\\xref\\biblatex@bbl@xref\\let\\biblatex@saved@fakeset\\fakeset\\let\\fakeset\\biblatex@bbl@fakeset\\let\\biblatex@saved@refsection\\refsection\\let\\refsection\\biblatex@bbl@refsection\\let\\biblatex@saved@endrefsection\\endrefsection\\let\\endrefsection\\biblatex@bbl@endrefsection";
const BBL_END: &str = "\\let\\verb\\biblatex@saved@verb\\let\\endverb\\biblatex@saved@endverb\\let\\datalist\\biblatex@saved@datalist\\let\\enddatalist\\biblatex@saved@enddatalist\\let\\entry\\biblatex@saved@entry\\let\\endentry\\biblatex@saved@endentry\\let\\name\\biblatex@saved@name\\let\\list\\biblatex@saved@list\\let\\field\\biblatex@saved@field\\let\\strng\\biblatex@saved@strng\\let\\keyw\\biblatex@saved@keyw\\let\\range\\biblatex@saved@range\\let\\preamble\\biblatex@saved@preamble\\let\\warn\\biblatex@saved@warn\\let\\xref\\biblatex@saved@xref\\let\\fakeset\\biblatex@saved@fakeset\\let\\refsection\\biblatex@saved@refsection\\let\\endrefsection\\biblatex@saved@endrefsection";

LoadDefinitions!({
  // Strict-Perl translation of ar5iv-bindings/biblatex.sty.ltxml
  // (803 lines): its macro definitions, conditionals, registers, the
  // trailing RawTeX toggle block, the author-year citation commands
  // (\parencite/\textcite/\cite families as \@@cite/\@@bibref closures,
  // `blx_cite_*`, greedy multicite readers, \citeauthor/\citetitle/
  // \citeyear/\citeyearpar — gated on style=/citestyle= author-year
  // detection, with a saved-core-\cite fallback), `\addbibresource` and
  // `\printbibliography` (biber's `.bbl` when present, the resources through
  // `\bibliography` otherwise; `&` catcode guard).
  //
  // Beyond the ar5iv binding: its `\entry`/`\endentry` rebuilder
  // (biblatex.sty.ltxml:495-690, `\name` :705, `\verb` :821-842), which typeset a
  // `\bibitem` from a dozen fields, is replaced by the `.bbl` reader at the
  // top of this file: a `.bbl`'s entries become the `ltx:bibentry`s their
  // `.bib` gives, formatted by MakeBibliography with biber's labels and order
  // (round 12, W6-B).

  // ar5iv-bindings biblatex.sty.ltxml L14-15 opens with
  //   Warn('missing_file', 'biblatex.sty',
  //        'biblatex.sty is only minimally stubbed and will not be interpreted raw.')
  // which this binding faithfully carried. That message is now doubly wrong,
  // and expensively so:
  //   * `missing_file` is inaccurate — nothing is missing. biblatex.sty is
  //     deliberately NOT raw-loaded because this binding stands in for it. The
  //     warning made biblatex.sty the #1 `missing_file` "what" in the corpus
  //     (1,167 papers across sandboxes 2605+2606, second only to arydshln),
  //     drowning genuinely missing files in bibliography surveys.
  //   * "only minimally stubbed" stopped being true long ago (above):
  //     author-year cite families, biber .bbl as ground truth,
  //     \printbibliography, a `.bbl` read into BibTeX entries.
  // It also fires UNCONDITIONALLY at load, so it says nothing about the paper
  // in hand — no diagnostic value, yet it downgraded every biblatex paper from
  // `no_problem` to `warning`.
  //
  // Keep the one genuinely useful fact — this is an approximation, not the real
  // package — as an Info, under an accurate category. A biblatex feature we get
  // wrong reports itself through its own error where it happens.
  //
  // OXIDIZED_DESIGN #62 — which also records the measurement trap: retiring this
  // moves ~1,167 papers out of `warning` for a LOGGING reason, so any corpus
  // `no_problem` delta straddling this commit is confounded.
  Info!(
    "bibliography",
    "biblatex",
    "biblatex.sty is provided by a native binding, not interpreted raw."
  );

  // Mark biblatex as provided, exactly as the real biblatex.sty's
  // `\ProvidesPackage{biblatex}[…]` does (latex_constructs `\ProvidesPackage`
  // → `\ver@biblatex.sty`). Every biber-generated `.bbl` opens with the guard
  //   \@ifundefined{ver@biblatex.sty}{\@latex@error{Missing 'biblatex' package}
  //     …\aftergroup\endinput}{}
  // so without `\ver@biblatex.sty` the `.bbl` `\endinput`s itself and the whole
  // References list is empty. The normal `\usepackage{biblatex}` path gets this
  // marker under the requested name from the package machinery, but a biblatex
  // *variant* (`\usepackage{biblatex-chicago}`, routed here by
  // `latexml_contrib::dispatch`'s `biblatex-*` fallback) would otherwise only
  // set `\ver@biblatex-chicago.sty`, leaving the guard's `\ver@biblatex.sty`
  // undefined. Setting it in the binding itself covers every load path.
  // Witness arXiv 2605.11180 (html_feedback #6601): biber `.bbl` format 3.3,
  // empty References before this marker. The string is biblatex.sty:34-35's
  // (`\abx@date\space v\abx@version\space programmable bibliographies`, the
  // version the `\abx@*` strings below carry); `\@ifpackagelater{biblatex}`
  // compares its date since batch 56ia.
  {
    let ver_cs = T_CS!("\\ver@biblatex.sty");
    DefMacro!(ver_cs, None, "2025/07/10 v3.21 programmable bibliographies (PK/MW)",
      scope => Some(Scope::Global));
  }

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
  // biblatex-chicago.sty:24-29 (`\DeclareVoidOption{authordate}` …) selects
  // `style=chicago-authordate` / `chicago-notes`; when routed here as the
  // variant, the options sit under `opt@biblatex-chicago.sty`. The
  // author-date family is our authoryear rendering (cms-dates-intro,
  // cms-dates-sample; lualatex clean).
  // Guard: `perfect_kernel_batch56::biblatex_chicago_authordate_has_gentextcite`.
  if let Some(opts) = lookup_vecdeque("opt@biblatex-chicago.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      if opt_str.trim().starts_with("authordate") {
        blx_set_style("authoryear");
      }
    }
  }
  if let Some(opts) = lookup_vecdeque("opt@biblatex.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      let Some((k, v)) = blx_opt_kv(&opt_str) else {
        continue;
      };
      match k.as_str() {
        "style" | "citestyle" => blx_set_style(&v),
        _ => {},
      }
    }
  }

  // Perl L24-30: dependencies. (`#RequirePackage('natbib')` etc commented out in Perl.)
  RequirePackage!("hyperref");
  RequirePackage!("ifthen");
  RequirePackage!("etoolbox");
  RequirePackage!("babel_support");
  // biblatex.sty:16305-16339: under the default UTF-8 input encoding,
  // `casechanger=auto` selects the expl3 case changer, whose
  // blx-case-expl3.sty:2 loads xparse. Classes rely on it: nwejmart.cls:462
  // `\NewDocumentCommand … { o u\q__nwejm }` needs xparse's `u` argument type
  // (TeX Live class census 2026-09-24).
  RequirePackage!("xparse");
  // biblatex.sty:15-21: the version strings every biblatex style file echoes in
  // its `\ProvidesFile` (dtk.cbx:12 `\ProvidesFile{dtk.cbx}[\abx@cbxid]`, a
  // third-party style the binding raw-loads below; dtk, TeX Live class census
  // 2026-09-24).
  RawTeX!(
    r"\def\abx@date{2025/07/10}
\def\abx@version{3.21}
\def\abx@bbxid{\abx@date\space v\abx@version\space biblatex bibliography style (PK/MW)}
\def\abx@cbxid{\abx@date\space v\abx@version\space biblatex citation style (PK/MW)}
\def\abx@lbxid{\abx@date\space v\abx@version\space biblatex localization (PK/MW)}
\def\abx@cptid{\abx@date\space v\abx@version\space biblatex compatibility (PK/MW)}
\def\abx@dmid{\abx@date\space v\abx@version\space biblatex datamodel (PK/MW)}"
  );

  // Cite commands — the three-family architecture from ar5iv-bindings
  // PRs #20/#21 (+ 0911aec repairs). Every command is a code closure over
  // blx_cite_* / blx_multicite_*; in non-author-year styles they all
  // delegate to the saved core \cite (blx_cite_fallback), so numeric
  // documents render exactly as before.
  //
  // Save the core \cite FIRST: \cite itself is redefined below, and the
  // fallback must reach the original, not recurse into our closure.
  // IDEMPOTENT: the versioned-package fallback (\RequirePackage{myBiblatex} ->
  // biblatex) probes by running the binding and then loads it again
  // (content.rs find_file_fallback + input_definitions reloadable), so this init
  // can execute twice. A bare `\let` on the 2nd pass would save biblatex's OWN
  // \cite closure, and \cite -> \blx@saved@cite -> \cite loops to the TokenLimit
  // (witness 2605.03965; Perl never loads biblatex on this name, so no loop).
  RawTeX!(r"\@ifundefined{blx@saved@cite}{\let\blx@saved@cite\cite}{}");

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
  // biblatex.sty:11066 `\footfullcite` — full citation in a footnote; bare
  // inline like `\footcite` above.
  DefMacro!("\\footfullcite OptionalMatch:* [][] Semiverbatim", sub[args] {
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
  // biblatex.sty:12726 `\Citetitle` — the capitalised form (cms-notes-sample).
  Let!("\\Citetitle", "\\citetitle");
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
  // biblatex.sty:12862 `\citefield` / :12802 `\citename` —
  // `[pre][post]{keys}[format]{field | namelist}`: the field-cite family.
  // The bibref model carries authors, title and year, so those fields map
  // to its `show` keys; any other field, and every non-authoryear style,
  // takes the plain-cite fallback `\citetitle` already uses. Style docs
  // exercise them at top level (oxref manuals ×4, biblatex-german-legal).
  // Guard: `perfect_kernel_batch56::biblatex_field_cites_and_page_strings`.
  for (cs, is_name) in [("\\citefield", false), ("\\citename", true)] {
    let spec = s!("{cs} OptionalMatch:* [][] Semiverbatim [] {{}}");
    DefMacro!(&spec, sub[args] {
      let mut it = args.into_iter();
      let star: Option<Tokens> = it.next().unwrap().into();
      let pre: Option<Tokens> = it.next().unwrap().into();
      let post: Option<Tokens> = it.next().unwrap().into();
      let keys: Tokens = it.next().unwrap().into();
      let _format: Option<Tokens> = it.next().unwrap().into();
      let field: Tokens = it.next().unwrap().into();
      let star = blx_nonempty(&star);
      let (pre, post) = blx_swap_pre_post(pre, post);
      let field = field.to_string().trim().to_ascii_lowercase();
      let show = match field.as_str() {
        "author" | "editor" | "translator" | "bookauthor" | "labelname" | "sortname" =>
          Some(if star { "FullAuthors" } else { "Authors" }),
        "title" | "shorttitle" | "maintitle" | "booktitle" | "labeltitle" => Some("Title"),
        "year" | "date" | "labelyear" => Some("Year"),
        _ => None,
      };
      let Some(show) = show.filter(|_| blx_is_authoryear()) else {
        return Ok(blx_cite_fallback(post, keys));
      };
      let bibref = Invocation!(T_CS!("\\@@bibref"),
        vec![Tokens::new(Explode!(show)), keys, Tokens!(), Tokens!()]);
      let mut body = Vec::new();
      if let Some(p) = pre { body.extend(p.unlist()); body.push(T_SPACE!()); }
      body.extend(bibref.unlist());
      if let Some(p) = post {
        body.extend(blx_ns().unlist()); body.push(T_SPACE!()); body.extend(p.unlist());
      }
      let class = if is_name { "citename" } else { "citefield" };
      Ok(Invocation!(T_CS!("\\@@cite"),
        vec![Tokens::new(Explode!(class)), Tokens::new(body)]))
    }, locked => true);
  }
  // biblatex.sty:3649 `\mkcomprange*[postpro]{string}` and :3505
  // `\mknormrange` compress or normalise a page-range string item by item;
  // the string itself is the rendering, with the postprocessor applied to it
  // when given. :3863 `\mkfirstpage*[postpro]{string}` keeps the first page
  // of the (first) range.
  RawTeX!(
    r"\def\lx@blx@rangeaux{\@ifnextchar[{\lx@blx@range@i}{\lx@blx@range@i[]}}
\def\lx@blx@range@i[#1]#2{\ifx\\#1\\#2\else#1{#2}\fi}
\def\mkcomprange{\@ifstar\lx@blx@rangeaux\lx@blx@rangeaux}
\def\mknormrange{\@ifstar\lx@blx@rangeaux\lx@blx@rangeaux}
\def\lx@blx@firstaux{\@ifnextchar[{\lx@blx@first@i}{\lx@blx@first@i[]}}
\def\lx@blx@first@i[#1]#2{\lx@blx@range@i[#1]{\lx@blx@firstpage#2-\@nil}}
\def\lx@blx@firstpage#1-#2\@nil{#1}
\def\mkfirstpage{\@ifstar\lx@blx@firstaux\lx@blx@firstaux}"
  );
  // biblatex.sty:4371-4377 (`\blx@blxinit`): the page-string family used
  // inside cite postnotes — `\pno`/`\ppno` are the localised page strings,
  // `\psq`/`\psqq` "sequens"/"sequentes", `\nopp` suppresses the prefix,
  // `\pnfmt` formats a postnote. Real biblatex hardens them to `\ERROR`
  // outside a citation context (:4382-4389); the manuals use them at top
  // level only inside `\cite[...]`, which this binding renders verbatim.
  RawTeX!(
    r"\protected\def\pnfmt#1{#1}
\protected\def\pno{\bibstring{page}}
\protected\def\ppno{\bibstring{pages}}
\let\nopp\relax
\protected\def\psq{\sqspace\bibstring{sequens}}
\protected\def\psqq{\sqspace\bibstring{sequentes}}"
  );

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
  // chicago-dates-common.cbx:2966 `\gentextcite` (genitive text cite) and
  // its `s`/`G` forms: the case is prose-only, so they render as `\textcite`.
  DefMacro!("\\gentextcite", "\\textcite");
  DefMacro!("\\gentextcites", "\\textcites");
  DefMacro!("\\Gentextcite", "\\textcite");
  DefMacro!("\\Gentextcites", "\\textcites");
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
  // biblatex.sty:15020 `\newrobustcmd*{\DeclareLabeldate}[2][]` — the optional
  // entrytype list (apa.bbx:337 `\DeclareLabeldate[constitution]{\field{date}}`).
  def_macro_noop("\\DeclareLabeldate []{}")?;
  // biblatex.sty:14952 `\DeclareExtradate{\scope{\field{…}}}` — the extradate
  // (year-suffix) scoping; chicago-authordate.bbx declares one at load.
  def_macro_noop("\\DeclareExtradate{}")?;
  // biblatex.sty:15093 `\DeclareLabeltitle[types]{spec}` and :14853
  // `\DeclareLabelalphaTemplate[types]{spec}` (biblatex-apa.bbx, ext-*.bbx).
  def_macro_noop("\\DeclareLabeltitle []{}")?;
  def_macro_noop("\\DeclareLabelalphaTemplate []{}")?;
  // biblatex.sty:2694-2705 `\uspunctuation`/`\stdpunctuation` toggle the
  // punctuation-inside-quotes tracker; no quote tracker here → no-ops.
  def_macro_noop("\\uspunctuation")?;
  def_macro_noop("\\stdpunctuation")?;
  // biblatex.sty:5541 `\letbibmacro*{new}{old}` — the macro bank is not
  // modelled (`\newbibmacro` is a declaration no-op), so aliasing is too.
  def_macro_noop("\\letbibmacro OptionalMatch:* {}{}")?;

  // Datamodel-declaration family (biblatex.sty L16639-16642 renews these to
  // warn-only outside the datamodel read; no XML consequence) plus internals
  // that raw-loaded style files reach for (witness biblatex-abnt abnt.bbx:
  // abntexto-exemplo). \abx@classtype is read as `\ifcase\abx@classtype`
  // (abnt.bbx L1264), so it must be a number source, not a macro noop.
  def_macro_noop("\\DeclareDatamodelFields[]{}")?;
  // biblatex.sty:16643 `\DeclareDatamodelConstraints[types]{\constraint…}`
  // (biblatex-cv.dbx:23/37): validation-only, its body (`\constraintfield`,
  // `\regexp{^…$}`) must never run in text.
  def_macro_noop("\\DeclareDatamodelConstraints[]{}")?;
  def_macro_noop("\\DeclareDatamodelEntryfields[]{}")?;
  def_macro_noop("\\DeclareDatamodelEntrytypes[]{}")?;
  def_macro_noop("\\DeclareDatamodelConstant[]{}{}")?;
  // \DeclareBibliographyAlias{alias}{entrytype} (biblatex.sty L2297).
  def_macro_noop("\\DeclareBibliographyAlias{}{}")?;
  def_macro_noop("\\DeclareNumChars OptionalMatch:* {}")?;
  // Declaration-only biber/data-model and setup hooks reached by raw-loaded
  // style chains (oxref.bbx, biblatex-sbl.def, chicago): they shape biber's
  // data model or the number checks, which our config-driven bibliography
  // never executes, so each swallows its body (biblatex.sty:14519
  // `\DeclareDataInheritance[opt]{src}{tgt}{rules}` — `\inherit`/`\noinherit`
  // live only inside its body; :3348 `\DeclareRangeChars*{}`; :3420
  // `\NumCheckSetup{}`; :14571 `\DeclareBiblistFilter{}{}` with
  // `\filter`/`\filteror` inside; :9372 `\defbibnote{}{}`; :9379
  // `\defbibfilter{}{}` whose `\type`/`\keyword`/`\and`… are local to the
  // body). `\ifbibmacroundef` (:2412) is `\ifcsundef{abx@macro@#1}` and our
  // `\newbibmacro` records nothing, so the undefined branch is faithful.
  // Witnesses: biblatex-oxref oxnum/oxalph/oxyear/oxnotes-doc, biblatex-cse-doc,
  // biblatex-musuos. Guard: `perfect_kernel_batch54::biblatex_loads_bbx_before_cbx`.
  def_macro_noop("\\DeclareDataInheritance[]{}{}{}")?;
  def_macro_noop("\\DeclareRangeChars OptionalMatch:* {}")?;
  def_macro_noop("\\NumCheckSetup{}")?;
  def_macro_noop("\\NumsCheckSetup{}")?;
  // The biblatex.sty internal/public surface that raw STYLE files reach at
  // cite/bibliography time (the binding stands in for biblatex.sty; Perl
  // raw-loads it and never gets this far — it fails at biblatex.sty:7113
  // `\ProcessLocalKeyvalOptions` on every biblatex document). Inventory by
  // static breadth across the biblatex-* bundles (wave-15 index-bib agent):
  // biber/data-model declarations are faithful-signature gobbles —
  // `\clearfield` (:2538, 35 bundles), `\DeclareCitePunctuationPosition`
  // (:12771), `\DeclareAutoPunctuation` (:12754), `\ResetDataInheritance`
  // (:14566), `\DefaultInheritance[]{}` (:14474, whose optional carries
  // `\except{}{}{}`, :14485), `\AtDataInput[][*]{}` (:8985, `.bbl` item
  // hooks our `.bbl` reader does not run), `\OnManualCitation{}` (:11442),
  // `\abx@missing@entry{}` (:11661), `\blx@nocite@do{}` (:12337, the kernel
  // `\nocite` records keys); control flow keeps its real shape —
  // `\blx@blxinit` (:1105, one-shot init → `\relax`), `\blx@safe@actives`/
  // `\blx@rest@actives` (:127), `\blx@ifdata{key}{t}{f}` (:8439
  // `\ifcsdef{blx@data@#1}`: no biber data in our model → false branch),
  // `\blx@xsanitizeafter{cmd}{text}` (:1216: `cmd{<detokenized text>}`).
  // `\DeclareMultiCiteCommand{\cites}[wrap]{\cite}{sep}` (:12500) defines a
  // NEW multicite name as its underlying cite command, as
  // `\DeclareCiteCommand` above does (windycity.cbx:179 `\idemcites`).
  // Witnesses: windycity, biblatex-sbl (`\citeshorthand`), biblatex-juradiss.
  // Guard: `perfect_kernel_batch54::biblatex_style_internal_surface`.
  def_macro_noop("\\clearfield{}")?;
  def_macro_noop("\\DeclareCitePunctuationPosition{}{}")?;
  def_macro_noop("\\DeclareAutoPunctuation{}")?;
  def_macro_noop("\\ResetDataInheritance")?;
  def_macro_noop("\\DefaultInheritance[]{}")?;
  def_macro_noop("\\except{}{}{}")?;
  def_macro_noop("\\AtDataInput[] OptionalMatch:* {}")?;
  def_macro_noop("\\OnManualCitation{}")?;
  def_macro_noop("\\abx@missing@entry{}")?;
  def_macro_noop("\\blx@nocite@do{}")?;
  DefMacro!("\\blx@blxinit", "\\relax");
  def_macro_noop("\\blx@safe@actives")?;
  def_macro_noop("\\blx@rest@actives")?;
  DefMacro!("\\blx@ifdata{}{}{}", "#3");
  DefMacro!(
    "\\blx@xsanitizeafter{}{}",
    "\\begingroup\\def\\blx@tempa{\\endgroup#1}\\edef\\blx@tempb{#2}\\expandafter\\blx@tempa\\expandafter{\\detokenize\\expandafter{\\blx@tempb}}"
  );
  def_macro_noop("\\DeclareBiblistFilter{}{}")?;
  def_macro_noop("\\defbibnote{}{}")?;
  def_macro_noop("\\defbibfilter{}{}")?;
  DefMacro!("\\ifbibmacroundef{}{}{}", "#2");
  // \blx@regimcs{\csa\csb…} registers "imc" wrappers (biblatex.sty L1137).
  def_macro_noop("\\blx@regimcs{}")?;
  RawTeX!(r"\chardef\abx@classtype=0 ");
  // nameref internal the abnt style patches into sectioning.
  def_macro_noop("\\NR@gettitle{}")?;

  // Perl L64-67: passthroughs
  DefMacro!("\\unspace", "\\relax");
  DefMacro!("\\blx@imc@resetpunctfont", "\\relax");
  DefMacro!("\\blx@postpunct", "\\@empty");
  // biblatex.sty:809-870/3488-3492 + biblatex.def:160-168/267-271/2175 declare
  // the counters with `\newcounter`, so `\defcounter`/`\setcounter`/`\value`
  // on them resolve (fiwi.bbx:59 `\defcounter{lownamepenalty}{0}` — "No
  // counter defined", biblatex-fiwi ×3; the biburl*/maxnames "not a register"
  // warnings on every biblatex doc). Defaults from :16390 `maxnames=3,
  // minnames=1` and :862-868 (penalties 0). `\the<counter>` is the arabic value.
  for name in [
    "tabx@nest",
    "listtotal",
    "listcount",
    "liststart",
    "liststop",
    "citecount",
    "citetotal",
    "multicitecount",
    "multicitetotal",
    "instcount",
    "maxnames",
    "minnames",
    "maxitems",
    "minitems",
    "citecounter",
    "maxcitecounter",
    "savedcitecounter",
    "uniquelist",
    "uniquename",
    "refsection",
    "refsegment",
    "maxextratitle",
    "maxextratitleyear",
    "maxextraname",
    "maxextradate",
    "maxextraalpha",
    "abbrvpenalty",
    "highnamepenalty",
    "lownamepenalty",
    "maxparens",
    "parenlevel",
    "mincomprange",
    "maxcomprange",
    "mincompwidth",
    "textcitecount",
    "textcitetotal",
    "textcitemaxnames",
    "biburlbigbreakpenalty",
    "biburlbreakpenalty",
    "biburlnumpenalty",
    "biburlucpenalty",
    "biburllcpenalty",
    "smartand",
  ] {
    NewCounter!(name);
  }
  SetCounter!("maxnames", Number::new(3));
  SetCounter!("minnames", Number::new(1));
  SetCounter!("maxitems", Number::new(3));
  SetCounter!("minitems", Number::new(1));
  SetCounter!("maxparens", Number::new(3));
  SetCounter!("smartand", Number::new(1));

  // Perl L69-72
  DefMacro!("\\addslash", "/\\hskip\\z@skip");
  DefMacro!("\\adddot", ".");
  DefMacro!("\\addcomma", ",");
  DefMacro!("\\autocap{}", "#1");

  // Perl L75-85
  DefMacro!("\\addspace", "\\space");
  DefMacro!("\\addnbspace", "\\space");
  DefMacro!("\\addthinspace", "\\space");
  DefMacro!("\\addnbthinspace", "\\space");
  DefMacro!("\\addlowpenspace", "\\space");
  DefMacro!("\\addhighpenspace", "\\space");
  DefMacro!("\\addlpthinspace", "\\space");
  DefMacro!("\\addhpthinspace", "\\space");
  DefMacro!("\\addabbrvspace", "\\space");
  DefMacro!("\\addabthinspace", "\\space");
  DefMacro!("\\adddotspace", "\\unspace\\adddot\\space");

  // Perl L87-91
  DefMacro!("\\noligature", "\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphen", "\\nobreak-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\nbhyphen", "\\nobreak\\mbox{-}\\nobreak\\hskip\\z@skip");
  DefMacro!("\\hyphenate", "\\nobreak\\-\\nobreak\\hskip\\z@skip");
  DefMacro!("\\allowhyphens", "\\nobreak\\hskip\\z@skip");

  // Perl L93-99
  DefMacro!("\\bibinitperiod", "\\adddot");
  DefMacro!("\\bibinithyphendelim", ".\\mbox{-}");
  DefMacro!("\\bibnamedelima", "\\addhighpenspace");
  DefMacro!("\\bibnamedelimb", "\\addlowpenspace");
  DefMacro!("\\bibnamedelimc", "\\addhighpenspace");
  DefMacro!("\\bibnamedelimd", "\\addlowpenspace");
  DefMacro!("\\bibnamedelimi", "\\addnbspace");

  // biblatex.sty:9280-9283 `\blx@bbl@dlist[type]{name}` (a format-2
  // `.bbl`'s `\sortlist`): a datalist starts; its name says which refcontext
  // it is for ([`bbl_default_datalist`]), its type whether it is the
  // bibliography's (`entry`) or a biblist's (`list`). (The ar5iv binding,
  // biblatex.sty.ltxml:428-434, set a keyval flag for its `\name`; the reader
  // tells the name forms apart by their content.)
  DefMacro!("\\biblatex@bbl@datalist[]{}", sub[(kind, name)] {
    bbl_start_datalist(kind, name);
    Ok(Tokens::default())
  });
  DefMacro!("\\sortlist[]{}", sub[(kind, name)] {
    bbl_start_datalist(kind, name);
    Ok(Tokens::default())
  });
  // Perl L107-108: \lossort / \refsection — empty stubs there; `\refsection`
  // records its resources here (below).
  DefMacro!("\\lossort", "", locked => true);
  // biblatex.sty:10757-10769 `\newrobustcmd*{\refsection}{…\@ifnextchar[{\blx@refsection}
  // {\blx@refsection[]}}`: an OPTIONAL resource list only — a mandatory `{}` here
  // swallowed the `\begin` of the next environment inside `\begin{refsection}[…]`
  // (biblatex-apa-test:1203-1208; batch 56ai).
  // biblatex.sty:10757 `\blx@refsection[resources]` records the section's
  // resources: biblatex-apa6-test.tex:444 declares its only `.bib` as
  // `\begin{refsection}[../bibtex/bib/…-references]` (dropped, every
  // citation was "Missing Entry"). The list goes to `\addbibresource`, which
  // splits the commas; one resource list serves all sections here.
  DefMacro!("\\refsection[]", "\\addbibresource{#1}", locked => true);

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
  // biblatex.sty:8316 `\blx@bbl@booltrue`: `\true{more<list>}` marks a name or
  // literal list the `.bib` ended with "and others" ([`BblEntry::more`]).
  DefMacro!("\\blx@bbl@booltrue{}", sub[(flag)] {
    if let Some(list) = flag.to_string().trim().strip_prefix("more") {
      let list = list.to_string();
      with_bbl_entry(|entry| entry.more.push(list));
    }
    Ok(Tokens::default())
  }, locked => true);
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

  // biblatex `\missing{entrykey}` (TL biblatex.sty L8503-8515
  // `\blx@bbl@missing`, aliased at L8857 `\let\missing\blx@bbl@missing`):
  // biber writes one per cite-key it could NOT find in any `.bib`. Upstream
  // records the key on `blx@miss@<refsection>` and emits a package WARNING
  // ("The following entry could not be found in the database … Please verify
  // the spelling and rerun") — it typesets NOTHING. (The bold marker a reader
  // sees at the citation site is a different macro, `\abx@missing` L1368
  // = `\mbox{\reset@font\bfseries#1}`, invoked from the `\cite` side.)
  //
  // So the faithful port is a no-op that warns. We don't keep the miss-list —
  // nothing consumes it: our `\cite` resolves against the entries actually
  // built from the `.bbl`, so an unfound key is already dangling by
  // construction. Perl's binding leaves `\missing` commented out
  // (ar5iv-bindings/biblatex.sty.ltxml L613), so every biber `.bbl` carrying
  // one costs an `Error:undefined:\missing` there.
  //
  // Warning, not error: an unresolvable cite-key is the AUTHOR's bug (a
  // stale `.bib`), reported by biblatex itself as a warning — not a gap in
  // our engine. Naming the key makes it actionable (issue #92,
  // "Rust-grade author errors"). Witness: arXiv 2605.17646, whose `.bbl`
  // ends in `\missing{Cowen2021}` for a `\cite{Cowen2021}` with no entry.
  DefMacro!("\\blx@bbl@missing{}", sub[(key)] {
    Warn!("missing_entry", "biblatex",
      format!("the entry '{}' could not be found in the database; \
               \\cite of it will not resolve. Please verify the spelling \
               and re-run biber.", key.to_string().trim()));
    Ok(Tokens::default())
  }, locked => true);
  Let!("\\missing", "\\blx@bbl@missing");
  // biblatex.sty:9285-9287: a datalist (`\sortlist` in a format-2 `.bbl`)
  // ends; the refsection prints one of them ([`bbl_end_datalist`]), and prints
  // it at its end ([`bbl_flush`]). `\lossort` lists shorthands, not entries.
  DefMacro!("\\biblatex@bbl@enddatalist", sub[_args] {
    bbl_end_datalist();
    Ok(Tokens::default())
  }, locked => true);
  DefMacro!("\\endsortlist", sub[_args] {
    bbl_end_datalist();
    Ok(Tokens::default())
  }, locked => true);
  def_macro_noop("\\endlossort")?;
  // The document's `\end{refsection}` prints what a `.bbl` read inside it left.
  DefMacro!("\\endrefsection", sub[_args] {
    Ok(bbl_flush())
  }, locked => true);
  DefMacro!("\\biblatex@bbl@flush", sub[_args] {
    Ok(bbl_flush())
  }, locked => true);

  // The bibliography a `.bbl` prints ([`bbl_flush`]): `\thebibliography`'s
  // element and placement, with the `bibstyle`/`citestyle` a `.bib`'s
  // `\bibliography` records, and `sort='false'` — biber sorted the list, so
  // MakeBibliography keeps its order. Its entries are `ltx:bibentry`, which
  // MakeBibliography formats; no `\bibitem`, so `\thebibliography`'s
  // pseudo-`\bibitem` rescue is not armed (as for `{bibtex@bibliography}`,
  // OXIDIZED_DESIGN #75). `\endthebibliography` closes it.
  DefConstructor!("\\biblatex@bbl@thebibliography",
  "<ltx:bibliography xml:id='#id' bibstyle='#bibstyle' citestyle='#citestyle' sort='false'>\
   <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title><ltx:biblist>",
  before_digest => {
    latexml_engine::latex_constructs::before_digest_bibliography()?;
  },
  after_digest => sub[whatsit] {
    latexml_engine::latex_constructs::begin_bibliography_clean(whatsit)?;
  },
  before_construct => sub[doc, whatsit] {
    latexml_engine::latex_constructs::adjust_backmatter_element(doc, whatsit)?;
  });

  // biblatex.sty:8681 `\blx@bbl@entry{key}{type}{options}`: an entry starts.
  DefMacro!("\\biblatex@bbl@entry{}{}{}", sub[(key, ty, options)] {
    BBL_ENTRY.with(|slot| {
      *slot.borrow_mut() = Some(BblEntry {
        key: key.to_string().trim().to_string(),
        entry_type: ty.to_string().trim().to_string(),
        listed: !bbl_skips_bib(&options.to_string()),
        fields: Vec::new(),
        date_parts: Vec::new(),
        more: Vec::new(),
      });
    });
    Ok(Tokens::default())
  }, locked => true);

  // The entry ends: its dates are rejoined and its cut-short name lists end
  // in "and others", and it is registered for the BibTeX reader under its key
  // and listed in its datalist — only now, so an entry with no `\endentry`
  // is never printed from an earlier registration.
  DefMacro!("\\biblatex@bbl@endentry", sub[_args] {
    let Some(mut bbl) = BBL_ENTRY.with(|slot| slot.borrow_mut().take()) else {
      return Ok(Tokens::default());
    };
    for &(prefix, date) in BBL_DATES {
      if let Some(value) = bbl_date(&bbl.date_parts, prefix) {
        bbl.fields.push((date.to_string(), value));
      }
    }
    let mut entry = BibEntry::new(bbl.key.clone(), bbl.entry_type);
    for (name, mut value) in bbl.fields {
      if bbl.more.contains(&name) {
        value.push_str(" and others");
      }
      entry.add_raw_field(name, value);
    }
    register_entry(&bbl.key, entry);
    if bbl.listed {
      BBL_DATALISTS.with(|lists| lists.borrow_mut().keys.push(bbl.key));
    }
    Ok(Tokens::default())
  }, locked => true);

  // BiblatexAuthor keyvals — extended (PR #21) with prefix/suffix/…-un
  // name parts so "van der Berg" / "King Jr." names parse correctly.
  DefKeyVal!("BiblatexAuthor", "given", "");
  DefKeyVal!("BiblatexAuthor", "giveni", "");
  DefKeyVal!("BiblatexAuthor", "givenun", "");
  DefKeyVal!("BiblatexAuthor", "family", "");
  DefKeyVal!("BiblatexAuthor", "familyi", "");
  DefKeyVal!("BiblatexAuthor", "familyun", "");
  DefKeyVal!("BiblatexAuthor", "prefix", "");
  DefKeyVal!("BiblatexAuthor", "prefixi", "");
  DefKeyVal!("BiblatexAuthor", "prefixun", "");
  DefKeyVal!("BiblatexAuthor", "suffix", "");
  DefKeyVal!("BiblatexAuthor", "suffixi", "");
  DefKeyVal!("BiblatexAuthor", "nameun", "");

  // biblatex.sty:8339 `\blx@bbl@namedef{list}{count}{options}{names}`: a name
  // list, one `{{<meta>}{<parts>}}` group per name. A format-3 `.bbl` (a
  // `\datalist` one) gives the parts as keys, `family={…},given={…},
  // prefix={…},suffix={…}` and their `…i` initials; an older one gives them in
  // place, `{family}{familyi}{given}{giveni}{prefix}{prefixi}{suffix}{suffixi}`.
  // The full parts are read back into BibTeX's "von Last, Jr, First" form
  // ([`bbl_bibtex_name`]) and the names joined with "and".
  DefMacro!("\\biblatex@bbl@name{}{}{}{}", sub[(role, _count, _options, body)] {
    let role = role.to_string().trim().to_string();
    if !BBL_NAMES.contains(&role.as_str()) {
      return Ok(Tokens::default());
    }
    let mut names: Vec<String> = Vec::new();
    for name in bbl_groups(&body.unlist()) {
      // The first group is the name's metadata (`hash=…`, `un=0,…`).
      let parts: Vec<String> = bbl_groups(&name)
        .into_iter()
        .skip(1)
        .map(|part| Tokens::new(part).untex())
        .collect();
      let keyvals = parts.first().is_some_and(|p| p.contains("family=") || p.contains("given="));
      let (given, prefix, family, suffix) = if keyvals {
        let kvs = parts.first().map(|kv| parse_name_keyvals(kv)).unwrap_or_default();
        let get = |k: &str| {
          kvs.iter().find(|(key, _)| key == k).map(|(_, v)| bbl_name_part(v)).unwrap_or_default()
        };
        (get("given"), get("prefix"), get("family"), get("suffix"))
      } else {
        let get = |i: usize| parts.get(i).map(|p| bbl_name_part(p)).unwrap_or_default();
        (get(2), get(4), get(0), get(6))
      };
      if !(family.is_empty() && given.is_empty()) {
        names.push(bbl_bibtex_name(&given, &prefix, &family, &suffix));
      }
    }
    if !names.is_empty() {
      with_bbl_entry(|entry| entry.fields.push((role, names.join(" and "))));
    }
    Ok(Tokens::default())
  }, locked => true);

  // biblatex.sty:8325 `\blx@bbl@listdef{list}{count}{items}`: a literal list,
  // one `{…}` group per item, joined with "and" as the `.bib` wrote it; an item
  // holding an "and" of its own is braced (biber's `{Smith and Sons}`).
  DefMacro!("\\biblatex@bbl@list{}{}{}", sub[(name, _count, items)] {
    let name = name.to_string().trim().to_string();
    if !BBL_LISTS.contains(&name.as_str()) {
      return Ok(Tokens::default());
    }
    let items: Vec<String> = bbl_groups(&items.unlist())
      .into_iter()
      .map(|item| {
        let item = bbl_source(Tokens::new(item));
        if item.split_whitespace().any(|word| word.eq_ignore_ascii_case("and")) {
          format!("{{{item}}}")
        } else {
          item
        }
      })
      .filter(|item| !item.is_empty())
      .collect();
    if !items.is_empty() {
      with_bbl_entry(|entry| entry.fields.push((name, items.join(" and "))));
    }
    Ok(Tokens::default())
  }, locked => true);

  // biblatex.sty:7871 `\blx@bbl@fielddef{field}{value}`: a data-model field
  // ([`BBL_FIELDS`]) or a date part ([`BBL_DATES`]); biber's own fields
  // (sort keys, `extra*` counters, `label*source`) are dropped.
  DefMacro!("\\biblatex@bbl@field{}{}", sub[(name, value)] {
    let name = name.to_string().trim().to_string();
    let is_date_part = BBL_DATES.iter().any(|(prefix, _)| {
      name.strip_prefix(prefix).is_some_and(|part| {
        matches!(part, "year" | "month" | "day" | "endyear" | "endmonth" | "endday")
      })
    });
    if is_date_part {
      let value = value.to_string().trim().to_string();
      with_bbl_entry(|entry| entry.date_parts.push((name, value)));
    } else if BBL_FIELDS.contains(&name.as_str()) {
      // (`labelalpha` is TeX, digested by the BibTeX reader: bibtex.rs
      // `\bib@field@default@labelalpha`.)
      let value = bbl_source(value);
      with_bbl_entry(|entry| entry.fields.push((name, value)));
    }
    Ok(Tokens::default())
  }, locked => true);
  // Name-list hashes and the like: nothing biblatex prints.
  def_macro_noop("\\biblatex@bbl@strng{}{}")?;

  // Perl L348-354
  def_macro_noop("\\AtEveryBibitem{}")?;
  def_macro_noop("\\AtEveryCitekey{}")?;
  // biblatex.sty:8385 `\blx@bbl@keyw{keywords}`: the entry's `keywords` field.
  DefMacro!("\\biblatex@bbl@keyw{}", sub[(keywords)] {
    let keywords = bbl_source(keywords);
    with_bbl_entry(|entry| entry.fields.push(("keywords".to_string(), keywords)));
    Ok(Tokens::default())
  }, locked => true);
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
  def_macro_noop("\\biblatex@bbl@range{}{}")?;

  // ar5iv biblatex.sty.ltxml:814: \preamble{...} is digested where it stands
  // (the ar5iv binding also stashed it for the `\thebibliography` it rebuilt).
  DefMacro!("\\biblatex@bbl@preamble{}", "#1");

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
    // (so `_` is SUB, `^` is SUPER, `#` is PARAM, etc.). Reset structural
    // catcodes to OTHER so the captured string is the field's text, which the
    // BibTeX reader reads verbatim (`doi`, `url`, `eprint`).
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
    let name = key_str.trim().to_string();
    if BBL_FIELDS.contains(&name.as_str()) {
      let value = value.to_string();
      with_bbl_entry(|entry| entry.fields.push((name, value)));
    }
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
  // The optional argument is real: `\blx@addbib` (biblatex L11285-11288) does
  // `\@ifnextchar[` before reading the resource, so `\addbibresource[
  // location=local]{refs.bib}` is valid biblatex. Without the `[]` in the
  // signature the `[` was taken as the resource itself, corrupting the
  // resource list and spilling `location=local]{refs.bib}` into the body —
  // the bibliography then resolved to nothing. Its keys (`location`, `label`,
  // `datatype`) are biber-side, so consuming and ignoring them is right here.
  // Witness 2605.27263; audit family F4(b).
  // The resource argument is EXPANDED: biblatex.sty:1216-1222 (`\blx@addbib`)
  // `\edef`s it and `\detokenize`s the result, so `\addbibresource
  // {\jobname.bib}` — the idiom of every manual that ships its `.bib` through
  // `filecontents` (biblatex-nejm/-spbasic/-juradiss/-license, cleanthesis's
  // `\cthesis@bibfile`, gitlog's `\gitLog@bibfile`, shtthesis's
  // `\sht@bib@resource`; 16 corpus docs) — records `<jobname>.bib`, not the
  // literal control sequence that no bibliography stage can open. Guard:
  // `cluster_package_guards::biblatex_addbibresource_expands_its_argument`.
  DefPrimitive!("\\addbibresource[] Expanded", sub[(_opts, file_list_arg)] {
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
  // biblatex.sty:11277-11283 `\addglobalbib`/`\addsectionbib` = `\blx@addbib`
  // with the global / per-refsection register; one resource list here
  // (shtthesis.cls, biblatex-apa-test `\addglobalbib`).
  Let!("\\addglobalbib", "\\addbibresource");
  Let!("\\addsectionbib", "\\addbibresource");
  // Idempotent for the same double-init reason as \blx@saved@cite above: a bare
  // \let on a 2nd init would save the already-rebound \bibliography (=\addbibresource).
  RawTeX!(
    r"\@ifundefined{biblatex@saved@bibliography}{\let\biblatex@saved@bibliography\bibliography}{}"
  );
  Let!("\\bibliography", "\\addbibresource");

  // \printbibliography: biber's \jobname.bbl is ground truth when present
  // (read it directly through the \entry/\endentry rebuilder above);
  // otherwise fall back to \biblatex@printbibliography, which routes the
  // \addbibresource declarations through LaTeXML's \bibliography
  // machinery. The `&` catcode change lets literal ampersands in .bbl
  // author/publisher names through (restored to alignment after).
  // The optional argument ([heading=bibintoc] etc.) is consumed+ignored.
  //
  // The `\verb`/`\endverb` rebinding is for the `.bbl` only and must NOT
  // outlive it: Perl (ar5iv-bindings biblatex.sty.ltxml:410) `\let`s them
  // unscoped, so every later `\verb+.dtx+` in the body became
  // `\biblatex@verb{} Until:\endverb` and swallowed the rest of the document
  // (docsurvey.tex:2876-2898 — 7 `\verb` after the bibliographies, ~500 lines
  // of content lost; rub-kunstgeschichte-example likewise; sweep 28).
  // KNOWN_PERL_ERRORS #114. Restore the saved meanings after the input — the
  // restores sit after `\InputIfFileExists` in the expansion, so they run once
  // the `.bbl` mouth is exhausted, like the `&` catcode restore already does.
  // Guard: `perfect_kernel_batch51::verb_survives_printbibliography`.
  //
  // The same scoping covers EVERY `.bbl` command: biblatex.sty:8995-9024
  // `\blx@bblstart` `\let`s `\entry`, `\name`, `\list`, `\field`, `\strng`,
  // `\range`, `\keyw`, `\preamble`, `\warn`, `\xref`, `\fakeset`,
  // `\datalist`… to their `\blx@bbl@…` bodies only for the `.bbl` read. A
  // document-wide `\list{}{}{}` (the ar5iv binding's shape, KNOWN_PERL_ERRORS
  // #133) swallows LaTeX's `\list{label}{setup}` — every `\list`-built
  // environment in a biblatex document (cnltx-doc `commands` under
  // `add-bib`) then ended with `\endlx@list` "Attempt to end mode". The
  // bodies live under `\biblatex@bbl@<name>` and `\biblatex@bblstart` /
  // `\biblatex@bblend` bracket the input. Guard:
  // `perfect_kernel_batch54::biblatex_bbl_commands_do_not_shadow_list`.
  DefMacro!(
    "\\printbibliography[]",
    "\\biblatex@bblstart\
     \\catcode`\\&=12\\relax\
     \\InputIfFileExists{\\jobname.bbl}{}{\\biblatex@printbibliography}\
     \\biblatex@bbl@flush\
     \\catcode`\\&=4\\relax\
     \\biblatex@bblend"
  );
  // A raw style `.def` can REPLACE `\printbibliography` with a copy of
  // biblatex's real one — biblatex-sbl.def:663 `\renewrobustcmd*
  // {\printbibliography}{\begingroup\blx@key@bibcheck{bibliography}…
  // \@ifnextchar[{\blx@printbibliography}{\blx@printbibliography[]}}` — which
  // reaches two biblatex.sty internals: the `check=` handler
  // `\blx@key@bibcheck` (biblatex.sty:9643) and the render worker
  // `\blx@printbibliography[#1]` (:9820, ends with the matching `\endgroup`).
  // Perl raw-loads biblatex.sty and has both; the binding stands in for the
  // package (RUST-ONLY), so it defines them: the check handler `\let`s
  // `\blx@bibcheck` to a defined `\blx@bibcheck@<name>` (`\defbibcheck` is a
  // no-op here, so the miss branch is silent), and the worker routes to the
  // binding's own `\printbibliography` body then closes the style's group.
  // Witness biblatex-sbl/sbl-paper (`\printbibliography[heading=bibintoc]`,
  // 2 errors; also biblatex-sbl.tex, biblatex-sbl-ibid). Guard:
  // `perfect_kernel_batch54::style_def_printbibliography_override_routes_to_binding`.
  DefMacro!(
    "\\blx@key@bibcheck{}",
    "\\@ifundefined{blx@bibcheck@#1}{}{\\expandafter\\let\\expandafter\\blx@bibcheck\\csname blx@bibcheck@#1\\endcsname}"
  );
  DefMacro!(
    "\\blx@printbibliography[]",
    "\\biblatex@sty@printbibliography[#1]\\endgroup"
  );
  Let!("\\biblatex@sty@printbibliography", "\\printbibliography");
  Let!("\\biblatex@bbl@verb", "\\biblatex@verb");
  Let!("\\biblatex@bbl@endverb", "\\biblatex@endverb");
  def_macro(
    T_CS!("\\biblatex@bblstart"),
    None,
    mouth::tokenize_internal(TeXString::assembled(BBL_START.to_string())),
    None,
  )?;
  def_macro(
    T_CS!("\\biblatex@bblend"),
    None,
    mouth::tokenize_internal(TeXString::assembled(BBL_END.to_string())),
    None,
  )?;
  DefMacro!("\\biblatex@printbibliography[]", sub[(_opts)] {
    assign_value("BIBSTYLE", pin(blx_bibstyle()), Some(Scope::Global));
    // DEDUPLICATE. The same `.bib` can be registered twice — a document that
    // declares `\addbibresource{refs.bib}` while its shipped `.cls` declares
    // the same file, for instance — and naming it twice makes
    // `\lx@bibliography` read it twice, which duplicates every entry. A
    // bibliography that doubles looks like a win by entry count, so guard it
    // here rather than trust the producers.
    let mut seen: Vec<String> = Vec::new();
    let mut resources = Vec::new();
    while let Some(res) = pop_value("biblatex_resources")? {
      let name = res.to_string();
      if seen.contains(&name) {
        continue;
      }
      seen.push(name.clone());
      if !resources.is_empty() {
        resources.push(T_OTHER!(","));
        resources.push(T_SPACE!());
      }
      resources.push(T_OTHER!(name));
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
  def_macro_noop("\\biblatex@bbl@warn{}")?;
  DefMacro!("\\biblatex@bbl@xref{}", "\\ref{#1}");
  def_macro_noop("\\biblatex@bbl@fakeset{}")?;
  // biblatex.sty:8634-8641 + 8999-9000: while the `.bbl` is read, `\refsection`
  // is the bbl variant taking a MANDATORY section number (`\refsection{0}` opens
  // every biber .bbl) and `\endrefsection` closes its group; the document-level
  // `\refsection[]` (optional resource list only) must not see that `{0}`.
  DefMacro!(
    "\\biblatex@bbl@refsection{}",
    "\\begingroup\\c@refsection#1\\relax"
  );
  // (biblatex.sty:8638-8642 also flushes `blx@addset` cross-reference sets
  // here; the binding models no `\set`/`\inset`.) The refsection's
  // bibliography is printed as it ends ([`bbl_flush`]).
  DefMacro!(
    "\\biblatex@bbl@endrefsection",
    "\\biblatex@bbl@flush\\endgroup"
  );

  // biblatex source-mapping API (a biber pre-processing stage LaTeXML does not
  // run): gobble the whole rule argument WITHOUT expanding it, so the nested
  // `\maps`/`\map`/`\step`/`\regexp` etc. never reach the stomach as undefined
  // control sequences. Undefined `\DeclareSourcemap` let those inner tokens run
  // and cascade into math-mode/`\fi` errors, breaking the whole conversion
  // (arXiv:2607.03177, html_feedback #6720). `\DeclareStyleSourcemap` is the
  // style-file sibling.
  def_macro_noop("\\DeclareSourcemap{}")?;
  def_macro_noop("\\DeclareStyleSourcemap{}")?;
  // biblatex.sty:14133 `\newrobustcmd*{\DeclareDriverSourcemap}[2][]` — the
  // driver-level sibling; cnltx.bbx:210 `\DeclareDriverSourcemap[datatype=bibtex]
  // {\map{\step[…]}}`. Undefined, the `\map`/`\step` body leaked into the
  // document (witnesses cnltx_en, endiagram_en, chemformula-manual). Guard:
  // `perfect_kernel_batch54::bbx_declaration_bodies_are_absorbed`.
  def_macro_noop("\\DeclareDriverSourcemap[]{}")?;

  // Perl L429-434: language API (no-ops)
  def_macro_noop("\\DeclareLanguageMapping{}{}")?;
  def_macro_noop("\\DeclareLanguageMappingSuffix{}")?;
  def_macro_noop("\\DefineHyphenationExceptions{}{}")?;
  def_macro_noop("\\DefineBibliographyExtras{}{}")?;
  def_macro_noop("\\UndefineBibliographyExtras{}{}")?;
  def_macro_noop("\\DefineBibliographyStrings{}{}")?;

  // Perl L436-438. A name format is also recorded for the given-name form
  // ([`blx_record_name_format`]); `[<entrytype>]` formats are not modelled.
  DefPrimitive!("\\DeclareNameFormat OptionalMatch:* []{}{}", sub[(_star, types, name, code)] {
    if blx_untyped(types.as_ref()) {
      blx_record_name_format(&name.to_string(), blx_name_format_given_form(&code.to_string()));
    }
  });
  def_macro_noop("\\DeclareListFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareFieldFormat OptionalMatch:* []{}{}")?;
  // biblatex.sty:4407-4425: the `\DeclareIndex{Name,List,Field}Format` forms
  // share `\blx@defformat` (and thus the exact signature) with their
  // non-Index siblings above. cnltx.bbx:131-163 declares six index field
  // formats and one index name format whose `#1` bodies, left unabsorbed,
  // reached the Stomach (`misdefined:#`) and fired `\nameparts`/`\usebibmacro`
  // /`\actualoperator` as undefined (witnesses cnltx_en, endiagram_en,
  // chemformula-manual). Guard: `perfect_kernel_batch54::bbx_declaration_bodies_are_absorbed`.
  def_macro_noop("\\DeclareIndexNameFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareIndexListFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareIndexFieldFormat OptionalMatch:* []{}{}")?;

  // Perl L440-458
  def_macro_noop("\\DeclareNameInputHandler{}{}")?;
  def_macro_noop("\\DeclareListInputHandler{}{}")?;
  def_macro_noop("\\DeclareFieldInputHandler{}{}")?;
  def_macro_noop("\\DeclareSortingScheme[]{}")?;
  def_macro_noop("\\DeclareSortingTemplate[]{}")?;
  def_macro_noop("\\DeclareSortingNamekeyScheme[]{}")?;
  def_macro_noop("\\namepart[]{}")?;
  def_macro_noop("\\DeclareLabelalphaNameTemplate[]{}")?;
  // `\DeclareNameAlias[<entrytype>]{<alias>}[<entrytype>]{<format>}`
  // (biblatex.sty:4494-4505, :4545), recorded like a name format.
  DefPrimitive!("\\DeclareNameAlias[]{}[]{}", sub[(types, alias, _format_types, format)] {
    if blx_untyped(types.as_ref()) {
      blx_record_name_alias(&alias.to_string(), &format.to_string());
    }
  });
  def_macro_noop("\\DeclareIndexNameAlias{}{}")?;
  def_macro_noop("\\DeclareListAlias{}{}")?;
  def_macro_noop("\\DeclareIndexListAlias{}{}")?;
  def_macro_noop("\\DeclareFieldAlias{}{}")?;
  def_macro_noop("\\DeclareIndexFieldAlias{}{}")?;
  def_macro_noop("\\DeclareNameWrapperAlias{}{}")?;
  def_macro_noop("\\DeclareListWrapperAlias{}{}")?;
  def_macro_noop("\\DeclareDelimcontextAlias{}{}")?;
  def_macro_noop("\\UndeclareDelimcontextAlias{}")?;
  // `\DeclareCiteCommand{\foo}[wrapper]{pre}{loop}{sep}{post}` is biblatex's own
  // API for declaring a citation command, and papers use it heavily. As a pure
  // no-op it consumed the arguments and defined NOTHING, so the declared
  // command stayed undefined: every call raised `Error:undefined:\foo`, no
  // citation record was made, and `MakeBibliography` then reported
  // "N bibentries, **0 cited**" and rendered an empty References list — the
  // bibliography was read and then thrown away for want of a `\cite`.
  //
  // Witness 2605.02115: `\DeclareCiteCommand{\citeq}` at main.tex L321, used
  // 83 times; 14 entries read, 0 cited, 0 rendered. pdflatex renders the list.
  //
  // The declared command is aliased to biblatex's `\cite`, which has the same
  // `OptionalMatch:* [][] Semiverbatim` shape the family uses. That keeps the
  // CITATION semantics — keys register, the reference resolves, the
  // bibliography renders — and drops only the author's bespoke label
  // formatting, which is the right trade for HTML. A redeclaration of an
  // existing command (`\DeclareCiteCommand{\cite}{…}`, restyling) aliases it
  // to itself and is harmless.
  DefMacro!("\\DeclareCiteCommand OptionalMatch:* {}[]{}{}{}{}",
  sub[(_star, target, _wrapper, _precode, _loopcode, _sepcode, _postcode)] {
    if let Some(tok) = target.unlist().into_iter().find(|t| t.get_catcode() == Catcode::CS)
      // Only give a NEW command the \cite fallback. A redeclaration of an
      // EXISTING command is a restyling — keep our native pipeline's version
      // (a raw .cbx's `\DeclareCiteCommand{\parencite}…` was aliasing
      // \parencite to plain \cite, dropping the parentheses; guard
      // `cluster_biblatex_authoryear*`).
      && lookup_meaning(&tok).is_none()
    {
      Let!(tok, T_CS!("\\cite"));
    }
    Ok(Tokens!())
  });

  // Perl L460-481
  def_macro_noop("\\DeclareBibliographyExtras{}")?;
  def_macro_noop("\\DeclareBibliographyStrings{}")?;
  def_macro_noop("\\DeclareBibliographyDriver{}{}")?;
  def_macro_noop("\\DeclareHyphenationExceptions{}")?;
  def_macro_noop("\\InheritBibliographyExtras{}")?;
  def_macro_noop("\\InheritBibliographyStrings{}")?;
  def_macro_noop("\\UndeclareBibliographyExtras{}")?;
  DefMacro!("\\NewCount", "\\newcount");
  // `\ExecuteBibliographyOptions{<options>}` (biblatex.sty:15080) sets the
  // global defaults, from a style's `.bbx` (phys.bbx:49, lncs.bbx:5,
  // chem-acs.bbx:51) or the preamble. Only `giveninits` reaches the native
  // bibliography ([`blx_record_giveninits`]); a per-type list (`[misc]{…}`,
  // fiwi.bbx:328) is scoped to those entry types and not modelled (no TL
  // style sets `giveninits` per type).
  DefPrimitive!("\\ExecuteBibliographyOptions[]{}", sub[(types, options)] {
    if blx_options_untyped(types.as_ref().map(|t| t.to_string()).as_deref()) {
      blx_execute_options(&options.to_string());
    }
  });
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
  // biblatex.sty:2393 `\newrobustcmd*{\usebibmacro}` (`\@ifstar`, one
  // mandatory arg). The binding never runs the bibmacros it stores as no-ops
  // (the bibliography is rendered from the model), so a use is a no-op too;
  // undefined, it fired inside every `.bbx` format body that reached the
  // document (cnltx.bbx:164; rub-kunstgeschichte example). Guard:
  // `perfect_kernel_batch54::bbx_declaration_bodies_are_absorbed`.
  def_macro_noop("\\usebibmacro OptionalMatch:* {}")?;
  // biblatex.sty:9436 `\newrobustcmd*{\defbibcheck}[2]` and :7029
  // `\newrobustcmd*{\DeclareRedundantLanguages}[2]`: arthistory-bonn.bbx:159/199.
  // Undefined `\defbibcheck` leaked its check body, whose `\ifcsdef{\strfield…}`
  // mis-nested into a live `\iffalse` that scanned to the .bbx EOF (witness
  // rub-kunstgeschichte-example: 3 undefined → 4 errors). :9784
  // `\printbibheading` takes one `\@ifnextchar[` optional (the standalone
  // heading; `\printbibliography` renders the list natively). Guard:
  // `perfect_kernel_batch54::biblatex_check_and_heading_commands_absorb_args`.
  def_macro_noop("\\defbibcheck{}{}")?;
  def_macro_noop("\\DeclareRedundantLanguages{}{}")?;
  def_macro_noop("\\printbibheading[]")?;
  def_macro_noop("\\defbibheading OptionalMatch:* {}[]{}")?;
  // biblatex.sty:9780 `\DeclarePrintbibliographyDefaults{<options>}` sets the
  // default `\printbibliography` options (njuthesis, omgtudoc-asoiu; class census
  // 2026-09-24); the options are presentational here.
  def_macro_noop("\\DeclarePrintbibliographyDefaults{}")?;
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

  // \MakeCapital{text} capitalizes the FIRST character only
  // (blx-case-latex2e.sty `\newrobustcmd{\MakeCapital}[1]{…\blx@mkcp@parse…}`):
  // "ibidem" → "Ibidem", never "IBIDEM".
  DefMacro!("\\MakeCapital{}", sub[(arg)] {
    let mut toks = arg.unlist();
    if let Some(first) = toks.first_mut()
      && matches!(first.get_catcode(), Catcode::LETTER | Catcode::OTHER)
    {
      let up = first.to_string().to_uppercase();
      *first = Token { text: pin(&up), code: first.code, #[cfg(feature = "token-locators")] loc: 0 };
    }
    Ok(Tokens::new(toks))
  });
  // \blx@err@patch{pkg} — error reporter if patching a package fails (biblatex.sty:154).
  DefMacro!("\\blx@err@patch{}", "");

  // \NewBibliographyString{key,key,…} — declares localization-string KEYS
  // (biblatex.sty; real def registers each key in the lbx string bank and
  // makes \bibstring{key} legal). This binding models no localization bank —
  // strings render through the rebuilt \thebibliography — so registering is
  // a declaration-only noop; the argument carries key names, not content.
  // Style packages call it at load (hep-bibliography.sty L116).
  def_macro_noop("\\NewBibliographyString{}")?;

  // Perl L553-604 declares these 50 as `\newif`-style TeX conditionals, but
  // biblatex.sty defines every one as a BRANCH-SELECTING MACRO —
  // `\iffieldundef{field}{true}{false}` (biblatex.sty:6205), `\ifcitation
  // {true}{false}` (:6003) — so a style/document calling them left the branch
  // groups in the stream and swallowed text to a stray `\fi` (biblatex
  // manuals: "Extra \fi", `\else` cascades). This binding models no data
  // fields, so the *undef* tests take their true branch and every other
  // predicate its false branch, except the `use<name>` toggles (default true,
  // biblatex.sty:8931) and `\ifhyperref`, answered from the loaded package.
  // Round 3 of the biblatex root-causing; guard:
  // `perfect_kernel_batch54::biblatex_tests_are_branch_macros`.
  {
    // (name, number of test arguments, take-the-true-branch?)
    let tests: &[(&str, usize, bool)] = &[
      ("andothers", 1, false),
      ("bibindex", 0, false),
      ("bibliography", 0, false),
      ("bibstring", 1, false),
      ("capital", 0, false),
      ("category", 1, false),
      ("citation", 0, false),
      ("citeibid", 0, false),
      ("citeidem", 0, false),
      ("citeindex", 0, false),
      ("citeseen", 0, false),
      ("currentfield", 1, false),
      ("currentlist", 1, false),
      ("currentname", 1, false),
      ("entrycategory", 2, false),
      ("entrykeyword", 2, false),
      ("entryseen", 1, false),
      ("entrytype", 1, false),
      ("fieldbibstring", 1, false),
      ("fieldequalcs", 2, false),
      ("fieldequals", 2, false),
      ("fieldequalstr", 2, false),
      ("fieldint", 1, false),
      ("fieldnum", 1, false),
      ("fieldnums", 1, false),
      ("fieldpages", 1, false),
      ("fieldsequal", 2, false),
      ("fieldundef", 1, true),
      ("firstinits", 0, false),
      ("firstonpage", 0, false),
      ("footnote", 0, false),
      ("integer", 1, false),
      ("keyword", 1, false),
      ("loccit", 0, false),
      ("moreitems", 0, false),
      ("morenames", 0, false),
      ("nameequalcs", 2, false),
      ("nameequals", 2, false),
      ("namesequal", 2, false),
      ("nameundef", 1, true),
      ("natbibmode", 0, false),
      ("numeral", 1, false),
      ("numerals", 1, false),
      ("opcit", 0, false),
      ("pages", 1, false),
      ("samepage", 2, false),
      ("singletitle", 0, false),
      ("useauthor", 0, true),
      ("useeditor", 0, true),
      ("useprefix", 0, false),
      ("usetranslator", 0, true),
    ];
    for (name, nargs, take_true) in tests {
      let proto = format!("\\if{name}{}{{}}{{}}", "{}".repeat(*nargs));
      let body = format!("#{}", nargs + if *take_true { 1 } else { 2 });
      let (cs_tok, params) = parse_prototype(&proto, true)?;
      def_macro(
        cs_tok,
        params,
        ExpansionBody::Tokens(mouth::tokenize_internal(TeXString::assembled(body))),
        None,
      )?;
    }
    DefMacro!("\\ifhyperref{}{}", "\\@ifpackageloaded{hyperref}{#1}{#2}");
  }
  // biblatex.sty:15506-15520 `\BiblatexManualHyperrefOn`/`Off` run
  // `\blx@mkhyperref`/`\blx@mknohyperref` once under `hyperref=manual`
  // (`\blx@hyperref`=3). Our citation/bibliography constructors link
  // unconditionally, so the manual switch has nothing to do here
  // (shtthesis.cls:330; Perl has no biblatex binding).
  def_macro_noop("\\BiblatexManualHyperrefOn")?;
  def_macro_noop("\\BiblatexManualHyperrefOff")?;

  // Perl L608-610 gobbles \key / \keyword silently. Round-34
  // surpass-Perl: preserve as classification tags so author keywords
  // reach the JATS output. \key{citekey} is bib-formatting internal —
  // leave that gobbled.
  def_macro_noop("\\key{}")?;
  // \keyw is already defined L348 (DefMacro empty, see above).
  DefMacro!(
    "\\keyword{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );

  // Perl L632-635
  DefMacro!("\\ppspace", "\\addnbspace");
  DefMacro!("\\sqspace", "\\addnbspace");
  DefMacro!("\\labelalphaothers", "+");
  DefMacro!("\\sortalphaothers", "\\labelalphaothers");

  // Perl L638
  def_macro_noop("\\sort[]{}")?;

  // Perl L641-645: bool stubs + AtBeginDocument-guarded \true/\false bind.
  // documents such as 1811.01740 conflict with unconditional binding.
  at_begin_document(TokenizeInternal!(
    r"\@ifundefined{true}{\let\true\blx@bbl@booltrue}{}\@ifundefined{false}{\let\false\blx@bbl@boolfalse}{}"
  ))?;

  // \type / \subtype are DELIBERATELY NOT defined here (the ar5iv
  // `biblatex.sty.ltxml` L646-647 defined both globally as `Tokens()`). Real
  // biblatex.sty defines `\def\type#1{type=#1}` / `\def\subtype#1{...}` ONLY
  // inside the `\begingroup` of a bibliography-filter body (biblatex.sty
  // L9380-9388, `\blx@defbibfilter`) — so in the document body `\type` is
  // undefined and a user's `\newcommand{\type}{...}` (a ubiquitous math macro)
  // takes effect. Defining `\type{}` globally as an arg-grabbing noop shadowed
  // that user macro AND, being `#1`-arity, ate the token after it: `\(\type\)`
  // consumed the closing `\)`, inline math never closed, and every following
  // display equation nested as `<ltx:equation> isn't allowed in <ltx:XMath>` —
  // the document rendered "half missing". arXiv/html_feedback#6681, witness
  // 2606.22155. (A `\defbibfilter{...\type{...}...}` is the only place biblatex
  // itself uses `\type`; our binding doesn't process filter bodies, so nothing
  // here needs the global stub.)
  //
  // Perl L648-671 made every `\the<counter>` empty; the counters above
  // (`NewCounter!`) give them their arabic value, as biblatex.sty:812-868 does.

  // Perl L673-688: print*/index*/entry* (all empty)
  def_macro_noop("\\printtext[]{}")?;
  def_macro_noop("\\printfield[]{}")?;
  def_macro_noop("\\printlist[][]{}")?;
  def_macro_noop("\\printnames[][]{}")?;
  def_macro_noop("\\printtime")?;
  def_macro_noop("\\printdate")?;
  def_macro_noop("\\printdateextra")?;
  // biblatex generates a `\print<type>date[extra]` for every date field
  // (origdate/eventdate/urldate/labeldate; chicago-dates-common.cbx:1872
  // `\printorigdate`, cms-notes-sample).
  for d in ["orig", "event", "url"] {
    def_macro_noop(&s!("\\print{d}date"))?;
    def_macro_noop(&s!("\\print{d}dateextra"))?;
  }
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
  DefMacro!(
    "\\mkbibbrackets{}",
    "\\begingroup\\bibopenbracket#1\\bibclosebracket\\endgroup"
  );
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
  DefMacro!(
    "\\pagenote{}",
    "\\@add@frontmatter{ltx:note}[role=pagenote]{#1}"
  );
  DefMacro!(
    "\\pagenotetext{}",
    "\\@add@frontmatter{ltx:note}[role=pagenote-text]{#1}"
  );

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
  RawTeX!(
    r#"
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
"#
  );

  // biblatex.sty:7649-7654 registers, for every name-list field of the data
  // model, the use-toggle `blx@use<name>` (default true) and the test
  // `\ifuse<name>` = `\iftoggle{blx@use<name>}` (styles test them:
  // standard-dw.bbx:1878 `\iftoggle{blx@useeditor}`, authortitle-dw.bbx:690
  // `\ifuseeditor`; biblatex-juradiss). The standard data model's name fields
  // (blx-dm.def) are registered here; `\providetoggle` keeps re-entry safe.
  RawTeX!(
    r"\def\lx@blx@regnameuse#1{\providetoggle{blx@use#1}\toggletrue{blx@use#1}\expandafter\def\csname ifuse#1\endcsname{\iftoggle{blx@use#1}}}
\lx@blx@regnameuse{author}\lx@blx@regnameuse{editor}\lx@blx@regnameuse{translator}
\lx@blx@regnameuse{annotator}\lx@blx@regnameuse{commentator}\lx@blx@regnameuse{introduction}
\lx@blx@regnameuse{foreword}\lx@blx@regnameuse{afterword}\lx@blx@regnameuse{holder}
\lx@blx@regnameuse{bookauthor}\lx@blx@regnameuse{editora}\lx@blx@regnameuse{editorb}
\lx@blx@regnameuse{editorc}\lx@blx@regnameuse{namea}\lx@blx@regnameuse{nameb}
\lx@blx@regnameuse{namec}\lx@blx@regnameuse{shortauthor}\lx@blx@regnameuse{shorteditor}"
  );

  // biblatex internals commonly invoked by user preamble. Witnesses
  // 2406.10485 (\newrefcontext), 2406.01081 (\newrefsection).
  // `\newrefsection[resources]` (biblatex.sty:10771) records resources as
  // `\refsection` does (defined above).
  DefMacro!("\\newrefsection[]", "\\addbibresource{#1}", locked => true);
  def_macro_noop("\\endrefcontext")?;
  // `\refsection[]` / `\endrefsection` are defined above (batch 56ai/56cs);
  // the no-ops that stood here overrode them.

  // `\refcontext`/`\newrefcontext` take an optional `[...]` and then a
  // mandatory group ONLY IF one actually follows: `\refcontext@i` guards it
  // with `\@ifnextchar\bgroup` and supplies `{}` itself otherwise (biblatex
  // L10437-10445). Declaring the group unconditionally made the noop eat
  // whatever came next — and what comes next is the whole point of the
  // block:
  //
  //     \begin{refcontext}[sorting=nyt]
  //         \printbibliography
  //     \end{refcontext}
  //
  // swallowed `\printbibliography`, so the document lost its bibliography
  // with no diagnostic at all. Witnesses 2606.11276, 2606.02676; audit
  // family F4(a). (`\begin{refcontext}` routes through `\refcontext` and
  // `\end{refcontext}` through `\endrefcontext`, so the environment form is
  // covered by these two.)
  DefMacro!(
    "\\lx@biblatex@refcontext[]",
    "\\@ifnextchar\\bgroup{\\@gobble}{}"
  );
  Let!("\\refcontext", "\\lx@biblatex@refcontext");
  Let!("\\newrefcontext", "\\lx@biblatex@refcontext");

  // biblatex L3408+ bibliography range separators. Define defensively.
  // NB: do NOT redefine \bibrangedash here — Perl L353 sets it to an en-dash
  // (U+2013), as do we above; a hyphen override would diverge from both Perl
  // and real biblatex. The date/time range separators are en-dashes too.
  def_macro_noop("\\bibrangessep")?;
  DefMacro!("\\bibdaterangesep", "\u{2013}");
  DefMacro!("\\bibtimerangesep", "\u{2013}");

  //
  // .bbx / .cbx style-file loading (biblatex.sty L2256-2258 / L11428-11435).
  //
  // The binding replaces biblatex.sty, so a `style=`/`bibstyle=`/`citestyle=`
  // option previously only picked author-year-vs-numeric — the style FILES
  // never loaded, and every toggle/macro they allocate stayed undefined
  // (windycity.bbx L51-129 `\providetoggle{short}…`, ext-standard.bbx L15-18,
  // fiwi.cbx L57-63, sbl.bbx→biblatex-sbl.def L205; 77 errors over 4 bundles;
  // Perl has no biblatex binding and raw-loads the whole chain). Declaration
  // surface first (argument gobblers — without them a raw .bbx errors on
  // every `\Declare*Option`), then the guarded raw load.
  //
  // {<scopes>}[<datatype>]{<key>}[<default>]{<code>} — biblatex.sty L7241.
  def_macro_noop("\\DeclareBiblatexOption{}[]{}[]{}")?;
  // biblatex's keyval-layer aliases (biblatex.sty L76-90): loaded .bbx/.cbx
  // styles call them directly (philosophy-standard.bbx; sidenotes chain).
  RawTeX!(
    r"\providecommand*{\blx@kv@defkey}{\define@key}
\providecommand*{\blx@kv@setkeys}{\setkeys}"
  );
  def_macro_noop("\\blx@kv@gdefkey{}{}[]{}")?;
  // Style-author API used by complex third-party styles (apa.bbx/cbx was the
  // regression witness — 27 undefined-CS errors when the raw style loaded):
  // declaration bodies must be SWALLOWED (their `\keypart`/`\regexp`/`\A`
  // sub-language is biber sorting spec, not TeX to execute), hooks are
  // presentational noops, punctuation gets its literal.
  def_macro_noop("\\DeclareDelimFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareDelimAlias OptionalMatch:* {}[]{}")?;
  def_macro_noop("\\DeclareLabelname OptionalMatch:* []{}")?;
  def_macro_noop("\\DeclareNosort{}")?;
  def_macro_noop("\\DeclareSortExclusion{}{}")?;
  def_macro_noop("\\DeclareSortInclusion{}{}")?;
  def_macro_noop("\\DeclareSortingNamekeyTemplate[]{}")?;
  def_macro_noop("\\DeclareUniquenameTemplate[]{}")?;
  DefMacro!("\\DeclareMultiCiteCommand{}[]{}{}", sub[(target, _wrapper, cite, _sep)] {
    if let Some(tok) = target.unlist().into_iter().find(|t| t.get_catcode() == Catcode::CS)
      && lookup_meaning(&tok).is_none()
    {
      let underlying = cite
        .unlist()
        .into_iter()
        .find(|t| t.get_catcode() == Catcode::CS)
        .unwrap_or_else(|| T_CS!("\\cite"));
      Let!(tok, underlying);
    }
    Ok(Tokens!())
  });
  def_macro_noop("\\DeclareBibliographyExtras{}")?;
  def_macro_noop("\\GenRefcontextData{}{}")?;
  def_macro_noop("\\AtEveryCite{}")?;
  def_macro_noop("\\AtBeginRefsection{}")?;
  // biblatex.sty:4124 `\AtUsedriver*{code}` (appends to the usedriver hook),
  // :4110 `\delimcontext{name}`, :12700 `\DeclareAutoCiteCommand{name}[pos]
  // {cite}{cites}` — biblatex-gost's bbx/cbx reach these three.
  def_macro_noop("\\AtUsedriver OptionalMatch:* {}")?;
  def_macro_noop("\\delimcontext{}")?;
  def_macro_noop("\\DeclareAutoCiteCommand OptionalMatch:* {}[]{}{}")?;
  // The rest of the hook family (biblatex.sty:10188/10381-10382/10741/11380-
  // 11397; blx-compat.def:155-156 `\AtBeginShorthands`/`\AtEveryLositem`),
  // reached by raw styles (philosophy-*.bbx:159/213, windycity.bbx;
  // arsclassica, sidenotes caesar_example). `\AtBeginBiblist`/
  // `\AtEveryBiblistitem` take the list name AND the body.
  def_macro_noop("\\AtBeginShorthands{}")?;
  def_macro_noop("\\AtEveryLositem{}")?;
  def_macro_noop("\\AtBeginBiblist{}{}")?;
  def_macro_noop("\\AtEveryBiblistitem{}{}")?;
  def_macro_noop("\\AtNextBibliography{}")?;
  def_macro_noop("\\AtEveryMultiCite{}")?;
  def_macro_noop("\\AtEachCitekey{}")?;
  def_macro_noop("\\AtNextCite{}")?;
  def_macro_noop("\\AtNextRefsection{}")?;
  def_macro_noop("\\AtEveryCitekey{}")?;
  def_macro_noop("\\RequireBiber[]")?;
  def_macro_noop("\\localrefcontext[]{}")?;
  DefMacro!("\\addsemicolon", ";");
  DefMacro!("\\addcolon", ":");
  DefMacro!("\\newunitpunct", ". ");
  // `\bibstring{key}` — the localisation bank is not modelled (strings render
  // through the rebuilt \thebibliography), but the keys a document reaches
  // directly through the page-string family and the common cite-context
  // strings get their english.lbx short forms (english.lbx:365-455,
  // `abbreviate=true` default); an unknown key stays the key.
  DefMacro!("\\bibstring{}", sub[(key)] {
    let key = key.to_string();
    let key = key.trim();
    let text = match key {
      "page" => "p.", "pages" => "pp.", "sequens" => "sq.", "sequentes" => "sqq.",
      "and" => "and", "andothers" => "et al.", "andmore" => "et al.",
      "editor" => "ed.", "editors" => "eds.", "byeditor" => "ed. by",
      "translator" => "trans.", "translators" => "trans.", "bytranslator" => "trans. by",
      "volume" => "vol.", "volumes" => "vols.", "number" => "no.", "edition" => "ed.",
      "chapter" => "chap.", "section" => "\u{a7}", "paragraph" => "par.",
      "column" => "col.", "columns" => "cols.", "line" => "l.", "lines" => "ll.",
      "verse" => "v.", "verses" => "vv.", "in" => "in", "idem" => "idem",
      "ibidem" => "ibid.", "opcit" => "op. cit.", "loccit" => "loc. cit.",
      "seenote" => "see n.", "quotedin" => "qtd. in", "citedas" => "henceforth cited as",
      "nodate" => "n.d.", "urlseen" => "visited on", "version" => "version",
      "phdthesis" => "PhD thesis", "mathesis" => "MA thesis",
      other => other,
    };
    Ok(Tokens::new(Explode!(text)))
  });
  // [<datatype>]{<key>}[<default>]{<code>} — biblatex.sty L7226-7228.
  // biblatex.sty:4430/4435 `\DeclareNameWrapperFormat`/`\DeclareListWrapperFormat`
  // (`[<entrytype>]{<format>}{<code>}`): render-side wrappers around name and
  // list formats — declaration-only here (biblatex-cv.sty:641-651 declares
  // them for every name field at load).
  def_macro_noop("\\DeclareNameWrapperFormat OptionalMatch:* []{}{}")?;
  def_macro_noop("\\DeclareListWrapperFormat OptionalMatch:* []{}{}")?;
  // Internals styles and documents reach directly: biblatex.sty:15791
  // `\blx@opt@loccittracker@<mode>` (loccit tracking modes; biblatex-sbl-ibid
  // .tex:200 calls `\blx@opt@loccittracker@false`) and :11077
  // `\blx@refpatch@sect{level}{code}{n}` / `@part` / `@chapter` (refsection
  // patches of sectioning commands; cmsendnotes.sty:121). Neither tracking
  // nor refsection boundaries are modelled — consume.
  for mode in ["true", "false", "context", "strict", "constrict"] {
    def_macro_noop(&s!("\\blx@opt@loccittracker@{mode}"))?;
  }
  def_macro_noop("\\blx@refpatch@sect{}{}{}")?;
  def_macro_noop("\\blx@refpatch@part{}")?;
  def_macro_noop("\\blx@refpatch@chapter{}")?;
  def_macro_noop("\\DeclareBibliographyOption[]{}[]{}")?;
  def_macro_noop("\\DeclareTypeOption[]{}[]{}")?;
  def_macro_noop("\\DeclareEntryOption[]{}[]{}")?;
  // Hook appenders (biblatex.sty L2265/L11437) — presentational init code.
  def_macro_noop("\\InitializeBibliographyStyle{}")?;
  def_macro_noop("\\InitializeCitationStyle{}")?;
  // Per-citation hooks and reset commands the style-driven docs use
  // (windycity manual). Real `\AtNextCitekey` defers its code to the next
  // citation; our cite pipeline is native, so run the code in place — the
  // idiomatic use (`\AtNextCitekey{\toggletrue{short}}\cite{x}`) sets the
  // toggle just before the citation either way.
  DefMacro!("\\AtNextCitekey{}", "#1");
  DefMacro!("\\AtNextMultiCite{}", "#1");
  def_macro_noop("\\citereset")?;
  def_macro_noop("\\newrefsegment")?;
  def_macro_noop("\\endrefsegment")?;
  def_macro_noop("\\printbiblist[]{}")?;
  // biblatex.sty:16006 `\printshorthands` = `\printbiblist{shorthand}` (kept
  // for compatibility; cms-legal-sample, cms-notes-sample).
  def_macro_noop("\\printshorthands[]")?;
  // Bibliography categories (biblatex ~L2900): filtering machinery with no
  // XML counterpart in the native pipeline — declare/assign as noops, test
  // takes the false branch. 6-bundle cluster (biblatex-abnt/-juradiss …).
  def_macro_noop("\\DeclareBibliographyCategory{}")?;
  def_macro_noop("\\addtocategory{}{}")?;
  // biblatex.sty:9955-9966 `\bibbycategory[<options>]` prints one bibliography
  // per declared category under its own heading. Categories are not modelled
  // (`\addtocategory` files nothing), so it prints the one bibliography there
  // is. gztarticle.cls:2580-2581 saves and renews it (TeX Live class census
  // 2026-09-24).
  DefMacro!("\\bibbycategory[]", "\\printbibliography[#1]");
  DefMacro!("\\ifcategory{}{}{}", "#3");
  DefMacro!("\\ifentrycategory{}{}{}", "#3");
  // biber never sentence-cases a title at the `.bib` layer (BibTeX's
  // `change.case$` "t" does, and lowercases unbraced control sequences with
  // it — `\H`→`\h`, `\TeX`→`\tex` — which is why the classic path keeps
  // `capitalize1` and its SHARED breakage); a biblatex style that wants
  // sentence case applies `\MakeSentenceCase*` at print time, protecting
  // control sequences. So a biblatex document reads titles as entered, like
  // amsrefs (`amsrefs_sty.rs`): `Erd\H{o}s` stays `Erd\H{o}s` (biblatex2bibitem,
  // shortmathj), `The \TeX book` keeps its `\TeX`. Guard:
  // `06_cluster_bibliography::biblatex_title_case_is_as_entered`.
  AssignValue!("BibTeX_title_case" => "asis");
  // The BibTeX reader digests the entries of a `.bbl` ([`bbl_flush`]), as it
  // does amsrefs' `\bib` entries (amsrefs_sty.rs). The reader's per-thread
  // state starts empty in every document.
  LoadPool!("BibTeX");
  BBL_ENTRY.with(|entry| *entry.borrow_mut() = None);
  BBL_DATALISTS.with(|lists| *lists.borrow_mut() = BblDatalists::default());
  DefMacro!("\\mkbibquote{}", "\u{201C}#1\u{201D}");
  DefMacro!("\\mkbibparens{}", "(#1)");
  DefMacro!("\\mkbibbrackets{}", "[#1]");
  // The rest of biblatex's public formatting family. Real biblatex registers
  // each as an "internal macro command" (`\blx@regimcs`, biblatex.sty:1137,
  // activated by `\blx@blxinit` inside the bibliography) AND as an always
  // available `\newrobustcmd` (biblatex.sty:13084-13093, :13217) — both are
  // stubbed here, so the names must be defined directly. `.bib` fields use
  // them freely (`title = {Review of \mkbibemph{The Last Marlin}}`,
  // `note = {… on\nopunct}` in biblatex-chicago's dates-test.bib; the
  // undefined name leaked its argument unemphasized: 7 + 4 corpus docs).
  // `\nopunct`/`\isdot`/`\newunit` act on biblatex's punctuation buffer
  // (biblatex.sty:2092-2106, :4207), which this pipeline has no analogue of
  // → no-ops; `\addperiod` is the period itself. `\lbx@initnamehook` /
  // `\lbx@inittitlehook` (biblatex.def:2121-2122) are the empty one-argument
  // hooks that the style name macros call (`name:hook`, biblatex.def:1135;
  // biblatex-chicago's raw .lbx). Guard:
  // `06_cluster_bibliography::biblatex_formatting_family_renders_in_bib_fields`.
  DefMacro!("\\mkbibemph{}", "\\emph{#1}");
  DefMacro!("\\mkbibbold{}", "\\textbf{#1}");
  DefMacro!("\\mkbibitalic{}", "\\textit{#1}");
  DefMacro!("\\mkbibsuperscript{}", "\\textsuperscript{#1}");
  // biblatex.def:401-404 — acronyms in small caps when the current font has a
  // small-caps shape. Documents use it outside any bibliography
  // (socialscienceshuberlin.tex:441 `\mkbibacro{CTAN}`). Guard:
  // `binding_singletons_56::biblatex_mkbibacro_and_standard_toggles`.
  RawTeX!(
    r"\newcommand*{\mkbibacro}[1]{\ifcsundef{\f@encoding/\f@family/\f@series/sc}{#1}{\textsc{\MakeLowercase{#1}}}}"
  );
  def_macro_noop("\\nopunct")?;
  def_macro_noop("\\isdot")?;
  def_macro_noop("\\newunit")?;
  DefMacro!("\\addperiod", ".");
  def_macro_noop("\\lbx@initnamehook{}")?;
  def_macro_noop("\\lbx@inittitlehook{}")?;
  // `\blx@inputonce{file}` — biblatex's guarded input; style .def files
  // call it directly (sbl.bbx L1 → biblatex-sbl.def; biblatex-software).
  DefPrimitive!("\\blx@inputonce{}", sub[(file)] {
    let f = do_expand(file)?.to_string();
    let f = f.trim();
    let (stem, ext) = f.rsplit_once('.').unwrap_or((f, "def"));
    let guard = s!("blx@inputonce@{f}");
    if lookup_value(&guard).is_none() {
      assign_value(&guard, Stored::from(true), Some(Scope::Global));
      let _ = input_definitions(stem, InputDefinitionOptions {
        extension: Some(Cow::Owned(ext.to_string())),
        noltxml: true,
        noerror: true,
        ..InputDefinitionOptions::default()
      });
    }
  });
  // \RequireBibliographyStyle / \RequireCitationStyle — raw-load the file
  // once (mirrors \blx@inputonce).
  DefPrimitive!("\\RequireBibliographyStyle{}", sub[(style)] {
    blx_load_style_file(&do_expand(style)?.to_string(), "bbx");
  });
  DefPrimitive!("\\RequireCitationStyle{}", sub[(style)] {
    blx_load_style_file(&do_expand(style)?.to_string(), "cbx");
  });
  // Drive the load from the package options, like real biblatex's end-of-
  // package style load: `style=<s>` sets both; `bibstyle=`/`citestyle=`
  // individually. bbx FIRST, then cbx — biblatex.sty:16439-16440
  // `\RequireBibliographyStyle{\blx@bbxfile}\RequireCitationStyle{\blx@cbxfile}`.
  // The oxref bundle depends on it: oxref.bbx:489-490 `\newtoggle`s
  // `blx@ox@autoanon`/`abbranon` and every oxref cbx `\providetoggle`s them
  // (oxnum.cbx:26); loading the cbx first made the bbx's `\newtoggle`
  // "already defined" (biblatex-oxref ×4). Guard:
  // `perfect_kernel_batch54::biblatex_loads_bbx_before_cbx`.
  let mut bibstyle: Option<String> = None;
  let mut citestyle_name: Option<String> = None;
  if let Some(opts) = lookup_vecdeque("opt@biblatex.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      let Some((k, v)) = blx_opt_kv(&opt_str) else {
        continue;
      };
      match k.as_str() {
        "style" => {
          bibstyle = Some(v.clone());
          citestyle_name = Some(v);
        },
        "bibstyle" => bibstyle = Some(v),
        "citestyle" => citestyle_name = Some(v),
        _ => {},
      }
    }
  }
  // biblatex-chicago.sty:24-31 selects the style by keyword (`notes` is the
  // default, :32 `\setkeys{cms@ldt}{notes}`) and loads `chicago-<style>`
  // through `\RequirePackage[style=…]{biblatex}` (:143); routed here as the
  // variant, the keyword sits under `opt@biblatex-chicago.sty`. The
  // chicago-notes bbx/cbx raw-load cleanly and declare the user cite commands
  // (`\runcite` chicago-notes.cbx:3164, `\headlessfullcite` :2919;
  // cms-legal-sample, cms-notes-sample). Raw-overlaying biblatex-chicago.sty
  // itself is not an option: it owns kvoptions declarations and `\patchcmd`s
  // of biblatex internals the binding does not have.
  // Guard: `perfect_kernel_batch56::biblatex_chicago_notes_loads_its_cbx`.
  if lookup_value("opt@biblatex-chicago.sty").is_some() || blx_variant_requested("biblatex-chicago")
  {
    RequirePackage!("xstring");
    let mut style = "chicago-notes";
    if let Some(opts) = lookup_vecdeque("opt@biblatex-chicago.sty") {
      for opt in opts.iter() {
        let opt_str = opt.to_string();
        style = match opt_str.trim() {
          "authordate" => "chicago-authordate",
          "authordate-trad" => "chicago-authordate-trad",
          "authordate16" => "chicago-authordate16",
          "authordate-trad16" => "chicago-authordate-trad16",
          "notes16" => "chicago-notes16",
          "notes" => "chicago-notes",
          _ => style,
        };
      }
    }
    if bibstyle.is_none() {
      bibstyle = Some(style.to_string());
    }
    if citestyle_name.is_none() {
      citestyle_name = Some(style.to_string());
    }
  }
  // biblatex.def's name formats (the binding stands in for the file): the
  // `author` list falls back to `default`, the `given-family` format
  // (biblatex.def:953/991), which, like `family-given` and
  // `family-given/given-family` (:878-935), follows `giveninits`; `initsonly`
  // (:944-950) always abbreviates. A style's declarations override these.
  for (alias, format) in [
    ("default", "given-family"),
    ("sortname", "family-given/given-family"),
    ("author", "default"),
    ("editor", "default"),
  ] {
    blx_record_name_alias(alias, format);
  }
  blx_record_name_format("initsonly", "initials");
  // biblatex.sty:16387 `\blx@kv@setkeys{blx@opt@ldt}{style=numeric}`: the
  // default style, whose numeric.bbx chains standard.bbx (its toggles, below).
  let bibstyle = bibstyle.or_else(|| Some("numeric".to_string()));
  if let Some(s) = &bibstyle {
    blx_load_style_file(s, "bbx");
  }
  if let Some(s) = &citestyle_name {
    blx_load_style_file(s, "cbx");
  }
  // The package options apply AFTER the style's `\ExecuteBibliographyOptions`
  // defaults (biblatex.sty:16439-16446: `\RequireBibliographyStyle`, then
  // `\blx@processoptions`), so `[style=phys,giveninits=false]` spells names out.
  if let Some(opts) = lookup_vecdeque("opt@biblatex.sty") {
    for opt in opts.iter() {
      blx_record_giveninits(&opt.to_string());
    }
  }
});

/// Whether a format's optional `[<entrytype>]` argument is absent or biblatex's
/// `*`, every type (biblatex.sty:4442-4445 `\blx@defformat`). A blank `[]`
/// names no type, so it defines nothing (:4447-4462 `\forcsvlist`).
fn blx_untyped(types: Option<&Tokens>) -> bool { types.is_none_or(|t| t.to_string().trim() == "*") }

/// Whether `\ExecuteBibliographyOptions`'s `[<entrytype>]` is absent or blank:
/// biblatex.sty:15081-15084 tests `\ifblank` only, so `*` names a type there.
fn blx_options_untyped(types: Option<&str>) -> bool { types.is_none_or(|t| t.trim().is_empty()) }

/// `\ExecuteBibliographyOptions{<key=value,...>}` without an entry type.
fn blx_execute_options(options: &str) {
  for opt in options.split(',') {
    blx_record_giveninits(opt);
  }
}

/// Record the name format `name`, printing given names as `form`
/// ([`blx_name_format_given_form`]). biblatex keeps formats and aliases in one
/// namespace (`\abx@nfd@*@<name>`, biblatex.sty:4462-4463 and :4502-4505), so
/// the later declaration of a name wins either way.
fn blx_record_name_format(name: &str, form: &str) {
  assign_value(
    &s!("blx@nfd@{}", name.trim()),
    Stored::from(s!("format:{form}")),
    Some(Scope::Global),
  );
}

/// Record `\DeclareNameAlias{alias}{format}` (see [`blx_record_name_format`]).
fn blx_record_name_alias(alias: &str, format: &str) {
  assign_value(
    &s!("blx@nfd@{}", alias.trim()),
    Stored::from(s!("alias:{}", format.trim())),
    Some(Scope::Global),
  );
}

/// How a name format's code prints given names: `switch` when it tests
/// `\ifgiveninits` (or the legacy `\iffirstinits`, blx-compat.def:217-222), as
/// biblatex.def's own formats do (:878-935); else `initials` when it prints the
/// given-name initials `\namepartgiveni` (apa.bbx:613-631, whose
/// `name:apa:family-given` prints the full name only to disambiguate,
/// :1034-1044; lncs.bbx:173); else `full` when it prints `\namepartgiven`
/// (biblatex-cse.bbx:61-68, although the style sets `giveninits`); else
/// `switch`, a format that prints no given name or leaves it to a bibmacro.
/// (A style-private switch between the two, geschichtsfrkl.bbx:105
/// `\ifbool{bbx:nurinit}`, reads as `initials`.)
fn blx_name_format_given_form(code: &str) -> &'static str {
  // `\<cs>` in `code`, not as the prefix of a longer name.
  let names_cs = |cs: &str| {
    code.match_indices(cs).any(|(at, _)| {
      !code[at + cs.len()..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '@')
    })
  };
  if names_cs("\\ifgiveninits") || names_cs("\\iffirstinits") {
    "switch"
  } else if names_cs("\\namepartgiveni") {
    "initials"
  } else if names_cs("\\namepartgiven") {
    "full"
  } else {
    "switch"
  }
}

/// The style the bibliography formatter keys on (`bibstyle`), for a `.bib`
/// and a `.bbl` alike: biblatex's standard styles print URLs, which the `.bst`
/// path's "Link" does not (make_bibliography.rs `style_prints_urls`;
/// OXIDIZED_DESIGN_DIVERGENCES #289), spell given names out unless the style's
/// name format or `giveninits` asks for initials ([`blx_prints_given_initials`];
/// make_bibliography.rs `style_given_name_form`), and follow biblatex's
/// punctuation tracker (make_bibliography.rs `PeriodRule`).
fn blx_bibstyle() -> &'static str {
  if blx_prints_given_initials() {
    "biblatex-giveninits"
  } else {
    "biblatex"
  }
}

/// Whether the bibliography prints given names as initials. biblatex prints an
/// entry's `author` list in the name format of that name (`\printnames{author}`,
/// standard.bbx; `\blx@getformat`, biblatex.sty:4509-4522), found through the
/// recorded aliases; a format that follows `\ifgiveninits` leaves it to the
/// `giveninits` option ([`blx_record_giveninits`]).
fn blx_prints_given_initials() -> bool {
  let mut name = String::from("author");
  // An alias cycle is an error in biblatex too; stop rather than loop.
  for _ in 0..32 {
    let declared = lookup_string(&s!("blx@nfd@{name}"));
    if let Some(target) = declared.strip_prefix("alias:") {
      name = target.to_string();
      continue;
    }
    match declared.as_str() {
      "format:initials" => return true,
      "format:full" => return false,
      _ => break,
    }
  }
  lookup_bool("biblatex_giveninits")
}

/// Record one biblatex `key[=value]` option if it is `giveninits` or its legacy
/// alias `firstinits` (blx-compat.def:224-229): print given names as initials.
/// biblatex's default is false; a bare key means true, and the last setting wins
/// (biblatex.def:878 `\ifgiveninits` reads the one switch). The bibliography
/// formatter learns it through the `bibstyle` marker set by
/// `\biblatex@printbibliography`.
fn blx_record_giveninits(opt: &str) {
  let (key, value) = blx_opt_kv(opt).unwrap_or_else(|| (opt.trim().to_string(), "true".into()));
  if matches!(key.as_str(), "giveninits" | "firstinits") {
    assign_value(
      "biblatex_giveninits",
      Stored::Bool(value == "true"),
      Some(Scope::Global),
    );
  }
}

/// `biblatex-<x>.sty` routed to this binding (`latexml_contrib::dispatch`):
/// the binding stands in for the `\RequirePackage{biblatex}` the variant
/// would do, then — for the variants in [`BIBLATEX_VARIANT_OVERLAYS`] — the
/// variant's own `.sty` is raw-input on top, so its user macros exist
/// (`\highlightname` biblatex-cv.sty:565). A variant is listed only when
/// its `.sty` is a bare `\RequirePackage{biblatex}` plus declarations the
/// binding's surface accepts; biblatex-chicago is NOT (kvoptions
/// declarations, `\patchcmd`s of biblatex internals, option clash) — its
/// style selection is mapped in `load_definitions` instead.
/// Guard: `perfect_kernel_batch56::biblatex_cv_variant_overlay`.
pub const BIBLATEX_VARIANT_OVERLAYS: &[&str] = &["biblatex-cv"];

pub fn load_variant(variant: &str) -> Result<()> {
  assign_value(
    "blx@variant",
    Stored::from(variant.to_string()),
    Some(Scope::Global),
  );
  if BIBLATEX_VARIANT_OVERLAYS.contains(&variant) {
    // The variant's own `\PassOptionsToPackage{style=…}{biblatex}` +
    // `\RequirePackage{biblatex}` (biblatex-cv.sty:12-17) reach this binding
    // through the package machinery with the style options intact, so the
    // variant runs first and the binding loads from inside it.
    let _ = input_definitions(variant, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("sty")),
      noltxml: true,
      noerror: true,
      ..InputDefinitionOptions::default()
    });
    if lookup_definition(&T_CS!("\\ver@biblatex.sty"))?.is_none() {
      load_definitions()?;
    }
    return Ok(());
  }
  load_definitions()
}

/// Is the biblatex binding being loaded on behalf of `biblatex-<x>.sty`
/// (the `latexml_contrib::dispatch` variant route, recorded by [`load_variant`])?
fn blx_variant_requested(variant: &str) -> bool { lookup_string("blx@variant") == variant }

/// A native style's `.bbx` is not loaded ([`blx_load_style_file`]), but the
/// option defaults it sets still apply: ieee.bbx:26-34 `\ExecuteBibliographyOptions
/// {giveninits, …}`, which pdflatex + biber print as "A.-T. Castro". Read the
/// file's top-level `\ExecuteBibliographyOptions` and `\RequireBibliographyStyle`
/// in file order (ieee-alphabetic.bbx:13 requires ieee before its own options),
/// as loading it would run them. A call nested in a group (ieee.bbx:21, the
/// body of `\DeclareBibliographyOption{dashed}`) runs only with that code.
fn blx_read_native_style_options(name: &str) {
  // A `.bbx` written by `filecontents` lives in the virtual file store.
  let Some(path) = find_file(
    name,
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("bbx")),
      ..Default::default()
    }),
  ) else {
    return;
  };
  let Some(text) = vfs_read(&path).or_else(|| {
    std::fs::read(&path)
      .ok()
      .map(|b| String::from_utf8_lossy(&b).into_owned())
  }) else {
    return;
  };
  for (cs, types, arg) in toplevel_calls(&text, &[
    "ExecuteBibliographyOptions",
    "RequireBibliographyStyle",
  ]) {
    if cs == "RequireBibliographyStyle" {
      blx_load_style_file(&arg, "bbx");
    } else if blx_options_untyped(types.as_deref()) {
      blx_execute_options(&arg);
    }
  }
}

/// The calls of the control sequences `names` at brace depth 0 of the TeX
/// source `text`, in order: (name, optional `[…]` argument, `{…}` argument).
/// `%` starts a comment; `\{`, `\}` and `\%` are escapes.
fn toplevel_calls(text: &str, names: &[&str]) -> Vec<(String, Option<String>, String)> {
  let code: String = text
    .lines()
    .map(|line| {
      let mut end = line.len();
      let mut chars = line.char_indices();
      while let Some((i, c)) = chars.next() {
        match c {
          '\\' => {
            chars.next();
          },
          '%' => {
            end = i;
            break;
          },
          _ => {},
        }
      }
      &line[..end]
    })
    .collect::<Vec<_>>()
    .join("\n");
  let chars: Vec<char> = code.chars().collect();
  // The balanced group opening at `chars[at]`, and the index after it.
  let group = |at: usize| -> (String, usize) {
    let mut depth = 0usize;
    let mut i = at;
    while i < chars.len() {
      match chars[i] {
        '\\' => i += 1,
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            return (chars[at + 1..i].iter().collect(), i + 1);
          }
        },
        _ => {},
      }
      i += 1;
    }
    (chars[at + 1..].iter().collect(), chars.len())
  };
  let skip_space = |mut i: usize| {
    while chars.get(i).is_some_and(|c| c.is_whitespace()) {
      i += 1;
    }
    i
  };
  let mut calls = Vec::new();
  let mut depth = 0usize;
  let mut i = 0;
  while i < chars.len() {
    match chars[i] {
      '\\' => {
        let start = i + 1;
        let mut end = start;
        while chars
          .get(end)
          .is_some_and(|c| c.is_ascii_alphabetic() || *c == '@')
        {
          end += 1;
        }
        let name: String = chars[start..end].iter().collect();
        i = end.max(start + 1);
        if depth > 0 || !names.contains(&name.as_str()) {
          continue;
        }
        let mut at = skip_space(i);
        let mut types = None;
        if chars.get(at) == Some(&'[') {
          let close = chars[at..]
            .iter()
            .position(|c| *c == ']')
            .map_or(chars.len(), |p| at + p);
          types = Some(chars[at + 1..close.min(chars.len())].iter().collect());
          at = skip_space(close + 1);
        }
        if chars.get(at) == Some(&'{') {
          let (arg, next) = group(at);
          calls.push((name, types, arg));
          i = next;
        }
        continue;
      },
      '{' => depth += 1,
      '}' => depth = depth.saturating_sub(1),
      _ => {},
    }
    i += 1;
  }
  calls
}

/// standard.bbx:4-8 allocates the toggles every standard-derived style tests
/// (`\iftoggle{bbx:doi}`, 20 + 9 errors in biblatex-ext's ext-*.bbx), and :21
/// `\ExecuteBibliographyOptions{isbn,url,doi,eprint,related}` sets them true
/// (each option's `[true]` default, :10-19); a load-time `<key>=false`
/// clears one. The native pipeline skips standard.bbx, so the skip allocates
/// them, once: the mark stands in for the file's own once-mark, which
/// biblatex.sty:1260-1274 `\blx@inputonce` sets globally
/// (`\global\cslet{blx@file@#1}\@empty`); the toggles are made as the file
/// makes them (etoolbox.sty:1169-1181). Guard:
/// `binding_singletons_56::biblatex_mkbibacro_and_standard_toggles`.
fn blx_standard_bbx_toggles() {
  const TOGGLES: [&str; 5] = ["isbn", "url", "doi", "eprint", "related"];
  if lookup_value("blx@standardtoggles").is_some() {
    return;
  }
  assign_value(
    "blx@standardtoggles",
    Stored::from(true),
    Some(Scope::Global),
  );
  let mut values = TOGGLES.map(|key| (key, true));
  if let Some(opts) = lookup_vecdeque("opt@biblatex.sty") {
    for opt in opts.iter() {
      if let Some((k, v)) = blx_opt_kv(&opt.to_string())
        && let Some(entry) = values.iter_mut().find(|(key, _)| *key == k)
      {
        entry.1 = v != "false";
      }
    }
  }
  let mut tex = String::new();
  for (key, on) in values {
    tex.push_str(&s!(
      "\\providetoggle{{bbx:{key}}}\\settoggle{{bbx:{key}}}{{{on}}}"
    ));
  }
  let _ = raw_tex(&tex);
}

/// Raw-load `<name>.bbx` / `<name>.cbx` once (biblatex.sty `\blx@inputonce`,
/// L2256-2258 / L11428-11435). Style files chain (`sbl.bbx` L1 inputs
/// `biblatex-sbl.def`; ext-*.bbx `\RequireBibliographyStyle{standard}`), so
/// the once-guard is essential. Missing file → silent (noerror), matching the
/// binding's defensive posture; the built-in styles that our native pipeline
/// already models (numeric*, alphabetic*, authoryear*, authortitle*, verbose*,
/// draft, debug, reading) are SKIPPED — their raw internals would fight the
/// native cite/bibliography closures for no gain.
fn blx_load_style_file(name: &str, ext: &str) {
  const NATIVE_STYLES: &[&str] = &[
    "numeric",
    "numeric-comp",
    "numeric-verb",
    "alphabetic",
    "alphabetic-verb",
    "authoryear",
    "authoryear-comp",
    "authoryear-ibid",
    "authoryear-icomp",
    "authortitle",
    "authortitle-comp",
    "authortitle-ibid",
    "authortitle-icomp",
    "authortitle-terse",
    "authortitle-tcomp",
    "authortitle-ticomp",
    "verbose",
    "verbose-ibid",
    "verbose-note",
    "verbose-inote",
    "verbose-trad1",
    "verbose-trad2",
    "verbose-trad3",
    "draft",
    "debug",
    "reading",
    "standard",
    // IEEE styles (biblatex-ieee): ieee.cbx/ieee-comp.cbx are numeric-comp
    // derivatives our native numeric pipeline already renders. Raw-loading
    // them ran `\patchcmd{\abx@macro@cite:comp:*}` against bibmacros our
    // `\newbibmacro` noop never defines, so biblatex-ieee raised its own
    // "Failed to update citation style" (~53 sandbox papers, error not
    // warning). Skip like the other native styles; the bibliography is
    // unchanged.
    // Only ieee-comp raised the error; ieee/ieee-alphabetic just
    // `\RequireCitationStyle{numeric-verb|alphabetic}` (already native), added
    // for consistency.
    "ieee",
    "ieee-comp",
    "ieee-alphabetic",
  ];
  let name = name.trim();
  if name.is_empty() {
    return;
  }
  let guard = s!("blx@styleloaded@{name}.{ext}");
  if lookup_value(&guard).is_some() {
    return;
  }
  assign_value(&guard, Stored::from(true), Some(Scope::Global));
  if NATIVE_STYLES.contains(&name) {
    if ext == "bbx" {
      blx_read_native_style_options(name);
      // Every built-in bibliography style but debug.bbx chains standard.bbx
      // (`\RequireBibliographyStyle{standard}`: numeric/alphabetic/authoryear/
      // authortitle/reading/draft.bbx:4; the others through those), and so do
      // third-party ones (ext-standard.bbx), so skipping it must still allocate
      // its toggles.
      if name != "debug" {
        blx_standard_bbx_toggles();
      }
    }
    return;
  }
  let _ = input_definitions(name, InputDefinitionOptions {
    extension: Some(Cow::Owned(ext.to_string())),
    noltxml: true,
    noerror: true,
    ..InputDefinitionOptions::default()
  });
  // biblatex.sty:7118-7128 (`\blx@inputonce{<style>.dbx}`): a bibliography
  // style's data model `<style>.dbx` loads with the bbx when it exists (biblatex-cv.dbx:11-30
  // `\newtoggle`s the `cv@blx:*` switches biblatex-cv.sty:27-35 then set).
  if ext == "bbx" {
    let _ = input_definitions(name, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("dbx")),
      noltxml: true,
      noerror: true,
      ..InputDefinitionOptions::default()
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn bbl_dates_rejoin_bibers_parts() {
    let parts = |list: &[(&str, &str)]| -> Vec<(String, String)> {
      list
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    };
    let date = parts(&[
      ("year", "2001"),
      ("month", "5"),
      ("day", "6"),
      ("endyear", "2001"),
      ("endmonth", "5"),
      ("endday", "8"),
      ("urlyear", "2020"),
      ("urlmonth", "3"),
      ("urlday", "4"),
    ]);
    assert_eq!(
      bbl_date(&date, "").as_deref(),
      Some("2001-05-06/2001-05-08")
    );
    assert_eq!(bbl_date(&date, "url").as_deref(), Some("2020-03-04"));
    assert_eq!(bbl_date(&date, "event"), None);
    // An empty end year is an open range.
    let open = parts(&[("year", "1990"), ("endyear", "")]);
    assert_eq!(bbl_date(&open, "").as_deref(), Some("1990/"));
  }

  #[test]
  fn bbl_names_read_back_as_bibtex() {
    assert_eq!(
      bbl_name_part(r"Martin\bibnamedelima Luther"),
      "Martin Luther"
    );
    assert_eq!(bbl_name_part(r"van\bibnamedelima der"), "van der");
    assert_eq!(
      bbl_bibtex_name("Pieter", "van der", "Berg", ""),
      "van der Berg, Pieter"
    );
    assert_eq!(
      bbl_bibtex_name("Martin Luther", "", "King", "Jr."),
      "King, Jr., Martin Luther"
    );
    assert_eq!(
      bbl_bibtex_name("", "", "{World Health Organization}", ""),
      "{World Health Organization}"
    );
    assert_eq!(
      bbl_bibtex_name("", "", "García Márquez", ""),
      "{García Márquez}"
    );
  }

  #[test]
  fn bbl_ranges_and_default_datalist() {
    assert_eq!(
      bbl_replace_macros(r"1\bibrangedash 10\bibrangessep 15", &[
        ("bibrangedash", "--"),
        ("bibrangessep", ", ")
      ]),
      "1--10, 15"
    );
    // Other control words are kept, and a longer name is not a prefix match.
    assert_eq!(
      bbl_replace_macros(r"\bibrangedashes x \emph{y}", &[("bibrangedash", "--")]),
      r"\bibrangedashes x \emph{y}"
    );
    assert!(bbl_default_datalist("nyt/global//global/global/global"));
    assert!(bbl_default_datalist("anyt/global//global/global"));
    assert!(!bbl_default_datalist(
      "nyt/apasortcite//global/global/global"
    ));
  }
}
