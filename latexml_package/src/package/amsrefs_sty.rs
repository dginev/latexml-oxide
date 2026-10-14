use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // amsrefs typesets `\bib` titles verbatim; the BibTeX pool's default
  // `capitalize1` re-casing lowercases later words INCLUDING control-sequence
  // names (`\LaTeX` → `\latex`, `\AmS` → `\ams`: amslatex-primer/amshelp).
  // Perl's amsrefs.sty.ltxml never overrides it (SHARED); pdflatex clean.
  // Guard: `perfect_kernel_batch56::sweep46_single_name_gaps`.
  AssignValue!("BibTeX_title_case" => "asis");

  // What amsrefs.sty loads (Perl amsrefs.sty.ltxml loads nothing, KNOWN_PERL_ERRORS #627): url (:45-46, when
  // url.sty exists), rkeyval (:213, `\DefineSimpleKey`, `\DefineAdditiveKey`), and unless the `lite` option
  // (:179, :214-218) textcmds and mathscinet (`\cprime`, `\germ`, `\scr`, `\Dbar`: the MathSciNet
  // transliterations a `\bib` entry carries). rkeyval and textcmds have no binding: they are read raw, so only
  // where raw loading is on (as `--preload=ar5iv.sty` has it). Witnesses 1907.08888, 2602.02342, math0410226,
  // 1503.01599, 1102.5022.
  DeclareOption!("lite", { AssignValue!("amsrefs@lite" => true, Scope::Global); });
  DeclareOption!("msc-links", { AssignValue!("amsrefs@msc-links" => true, Scope::Global); });
  DeclareOption!(None, {});
  ProcessOptions!();
  // amsrefs.sty:198-205: `msc-links` loads hyperref, whose `\href` its `\MRhref` (below) links with.
  if lookup_bool("amsrefs@msc-links") {
    RequirePackage!("hyperref");
  }
  RequirePackage!("url");
  if lookup_bool("INCLUDE_STYLES") {
    RequirePackage!("rkeyval");
  }
  if !lookup_bool("amsrefs@lite") {
    if lookup_bool("INCLUDE_STYLES") {
      RequirePackage!("textcmds");
    }
    RequirePackage!("mathscinet");
  }
  // Perl: amsrefs.sty.ltxml — Leverage the BibTeX implementation

  // Perl: LoadPool('BibTeX');
  LoadPool!("BibTeX");

  // Perl: DefParameterType('BibURL', ...) — semiverbatim URL reading
  // Perl: DefKeyVal('amsrefs', 'url', 'BibURL');
  // TODO: BibURL parameter type and amsrefs keyval not yet ported

  // \bib{key}{type}{keyval-pairs}
  // Perl: DefMacro('\bib{}{} RequiredKeyVals:amsrefs', sub { ... });
  //
  // Reads the key + entry-type as strings and parses the keyval slot
  // via `parse_amsrefs_keyvals` into individual `BibEntry::add_raw_field`
  // calls. Mirrors Perl `amsrefs.sty.ltxml:42-51`: `lc()` on each key,
  // `UnTeX()` on each value (we use the raw source — close enough for
  // the downstream `\bib@field@<type>@<field>` dispatch since the
  // handlers themselves do the digestion).
  //
  // Emits `\ProcessBibTeXEntry{<key>}` (Perl L51) so the bibtex.rs
  // orchestration drives field dispatch + entry-type prepare/complete
  // and builds the `<ltx:bibentry>` XML.
  DefMacro!("\\bib{}{}{}", sub[args] {
    use latexml_engine::bibtex::{BibEntry, register_entry, parse_amsrefs_keyvals};
    let key = if args[0].is_some() { args[0].to_string() } else { String::new() };
    let entry_type = if args[1].is_some() { args[1].to_string() } else { String::new() };
    // Perl L48: `UnTeX(shift(@rawpairs), 1)`. `untex()` — NOT `to_string()`:
    // the tokenizer eats the space that terminates a control word, so a plain
    // token→string round-trip renders `661\ndash 693` as `661\ndash693`, and
    // that string is what the `<ltx:bib-data>` BibTeX dump reproduces verbatim.
    // `untex` re-inserts the separator at the letter boundary.
    let raw_kv = if args[2].is_some() {
      args[2].clone().owned_tokens().unwrap_or_default().untex()
    } else { String::new() };
    let mut entry = BibEntry::new(key.clone(), entry_type);
    for (field, value) in parse_amsrefs_keyvals(&raw_kv) {
      entry.add_raw_field(field, value);
    }
    register_entry(&key, entry);
    // Emit `\ProcessBibTeXEntry{<key>}` to drive bibtex.rs orchestration.
    Ok(Invocation!(T_CS!("\\ProcessBibTeXEntry"),
      vec![Tokens::new(Explode!(&key))]))
  });

  // \BibSpec — ignore
  def_macro_noop("\\BibSpec{}{}")?;
  // amsrefs.sty:339 `\BibSpecAlias{new}{old}` gives a type the print spec of another; we print by type
  // ourselves, so it is ignored as `\BibSpec` is.
  def_macro_noop("\\BibSpecAlias{}{}")?;
  // amsrefs.sty:2511 `\bysame`: the rule that stands for a repeated author list.
  DefMacro!("\\bysame", "\\leavevmode\\hbox to3em{\\hrulefill}\\thinspace\\kern\\z@");
  // amsrefs.sty:651 `\resetbiblist{widest label}` sets the label width of the list it is given to (as
  // `\begin{biblist}[\resetbiblist{99}]`); the width is presentation.
  def_macro_noop("\\resetbiblist{}")?;
  // amsrefs.sty:1564-1568, 3023-3026: `\ocite`/`\ocites` cite as an object of the sentence; they cite.
  DefMacro!("\\ocite", "\\cite");
  DefMacro!("\\ocites", "\\cites");

  // amsrefs.sty:1291-1316 `\bibselect{db1,db2}` (`\bibselect*`, `[bblname]`): with `\if@filesw` it reads
  // every `\bib` of each database `<db>.ltb` (else `.ltx`, `.tex`; `\ReadBibData`, :1324-1340, the
  // `\ReadBibLoop` executes the lines that start with `\bib`, :1348-1369), writes the selected ones —
  // the cited ones, all of them for `\bibselect*` — to `<bblname>.bbl` and inputs that. With no database
  // shipped the `.bbl` of the author's run is input as it is. The entries of a database are marked
  // `ltx_bib_library`, so the bibliography prints the cited ones (latexml_post make_bibliography), as a
  // `.bib` library; `\bibselect*` keeps them all. Perl has no `\bibselect` (KNOWN_PERL_ERRORS #627).
  // Witnesses 1904.05225 (`CF.ltb`), 2103.06687, 2508.05726 (the `.bbl`).
  DefMacro!("\\bibselect", "\\@ifstar{\\lx@amsrefs@bibselect{*}}{\\lx@amsrefs@bibselect{}}");
  DefMacro!("\\lx@amsrefs@bibselect{}[]{}", sub[(star, bblname, list)] {
    let select_all = star.to_string() == "*";
    let mut records = String::new();
    let mut found = false;
    for name in do_expand(list)?.to_string().split(',') {
      let name = name.trim();
      if name.is_empty() {
        continue;
      }
      let text = ["ltb", "ltx", "tex"].iter().find_map(|ext| {
        find_file(name, Some(FindFileOptions {
          ext_type: Some(Cow::Borrowed(ext)),
          forbid_ltxml: true,
          ..Default::default()
        }))
        .filter(|path| path.ends_with(&format!(".{ext}")))
        .and_then(|path| std::fs::read(path).ok())
      });
      match text {
        Some(bytes) => {
          found = true;
          let text: String = String::from_utf8(bytes.clone())
            .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect());
          for record in amsrefs_database_records(&text) {
            if !select_all && let Some(key) = amsrefs_record_key(&record) {
              assign_value(&s!("BIBENTRY_LIBRARY:{}", key.to_lowercase()), true, Scope::Global);
            }
            records.push_str(&record);
            records.push('\n');
          }
        },
        None => Info!("missing_file", name, s!("No data file {name}.ltb (.ltx, .tex) found")),
      }
    }
    if found {
      if !records.is_empty() {
        open_mouth(Mouth::new(&records, None)?, true);
      }
      return Ok(Tokens!());
    }
    let bbl = match bblname {
      Some(name) => name.to_string(),
      None => s!("{}", do_expand(Tokens!(T_CS!("\\jobname")))?.to_string()),
    };
    Ok(Tokens!(T_CS!("\\InputIfFileExists"), T_BEGIN!(), Explode!(s!("{bbl}.bbl")), T_END!(),
      T_BEGIN!(), T_END!(), T_BEGIN!(), T_END!()))
  });

  // \cites = \cite
  Let!("\\cites", "\\cite");
  // amsrefs.sty L1467: `\citelist{ \cite{key1} \cite{key2} ... }` —
  // grouped multi-citation where each `\cite` may carry `*{prenote}`.
  // Degrade to passing the body through; each inner `\cite` renders
  // independently. Witness 2404.11319.
  DefMacro!("\\citelist{}", "#1");

  // {bibdiv} environment — amsrefs.sty.ltxml L60-68.
  // beforeDigest: beforeDigestBibliography (preamble/counter/guard setup).
  // afterDigestBegin: beginBibliography_clean + Let('\par','\relax'). The
  // `_clean` variant skips setup_pseudo_bibitem because amsrefs bibliographies
  // always use explicit `\bibitem`; the pseudo-bibitem machinery rebinds
  // `\bibitem` and would break amsrefs' own `\bib{...}{...}{...}` entries.
  // The Let('\par','\relax') silences the implicit paragraph breaks between
  // entries (amsrefs items are sibling elements, not paragraphs).
  DefEnvironment!("{bibdiv}",
    "<ltx:bibliography xml:id='#id' \
     bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
     <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>\
     #body\
     </ltx:bibliography>",
    before_digest => {
      engine::latex_constructs::before_digest_bibliography()?;
    },
    after_digest_begin => sub[whatsit] {
      engine::latex_constructs::begin_bibliography_clean(whatsit)?;
      Let!("\\par", "\\relax");
    });

  // {bibsection}[heading] — real amsrefs.sty L1251. amsrefs defines `bibdiv`
  // AS `bibsection` in article mode (`\newenvironment{bibdiv}{\bibsection}
  // {\endbibsection}`, L1265): `bibsection` is the section-heading wrapper
  // around `{biblist}`, its optional arg the heading (default `\refname`).
  // Perl amsrefs.sty.ltxml binds only `bibdiv`/`biblist`, so a paper that opens
  // its references with `\begin{bibsection}` directly hit `Environment
  // {bibsection} is not defined` and lost the whole References list (the
  // `<ltx:biblist>` then floated in an `<ltx:p>`). Shared gap with Perl; this is
  // a faithful port of the real `.sty`. Same `<ltx:bibliography>` container and
  // hooks as `bibdiv`; the optional arg is the title (begin_bibliography_clean
  // also defaults an absent title to `\refname`, matching `[\refname]`).
  // html_feedback #1393, witness arXiv 2405.18501 (`\documentclass{amsart}`,
  // `\usepackage[numeric]{amsrefs}`, 6 `\bib` entries).
  // {bibchapter}[heading] — amsrefs.sty:1242-1250, the chapter-level heading `{bibdiv}` is under a class with chapters
  // (:1265-1269), titled `\bibname`; a book that opens its references with it directly met `{bibchapter}` undefined
  // (run 345's Fatal TooManyErrors in 1612.01009, 1706.05540). Both build the same bibliography.
  DefEnvironment!("{bibsection}[Default:\\refname]",
    "<ltx:bibliography xml:id='#id' \
     bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
     <ltx:title font='#titlefont' _force_font='true'>#1</ltx:title>\
     #body\
     </ltx:bibliography>",
    before_digest => {
      engine::latex_constructs::before_digest_bibliography()?;
    },
    after_digest_begin => sub[whatsit] {
      engine::latex_constructs::begin_bibliography_clean(whatsit)?;
      Let!("\\par", "\\relax");
    });

  DefEnvironment!("{bibchapter}[Default:\\bibname]",
    "<ltx:bibliography xml:id='#id' \
     bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
     <ltx:title font='#titlefont' _force_font='true'>#1</ltx:title>\
     #body\
     </ltx:bibliography>",
    before_digest => {
      engine::latex_constructs::before_digest_bibliography()?;
    },
    after_digest_begin => sub[whatsit] {
      engine::latex_constructs::begin_bibliography_clean(whatsit)?;
      Let!("\\par", "\\relax");
    });

  // {biblist} environment. amsrefs.sty:532-569 `\@biblist[#1]` takes the list's settings (`[\resetbiblist{99}]`,
  // `[prefix=A]`), which are presentation; `{biblist*}` (:538) is the list that continues the numbering.
  DefEnvironment!("{biblist}[]", "<ltx:biblist>#body</ltx:biblist>");
  DefEnvironment!("{biblist*}[]", "<ltx:biblist>#body</ltx:biblist>");

  // \MR{...} — MathReviews link. Perl amsrefs.sty.ltxml L75-82:
  // properties closure patches old-style "12345 \# 67" → "12345:67" and
  // emits both mr= and href= AMS lookup URL. Ported directly with
  // a regex for the \# substitution.
  DefConstructor!("\\MR{}",
    "<ltx:ref href='#href' class='ltx_mathreviews'>MathReviews</ltx:ref>",
    enter_horizontal => true,
    properties => sub[args] {
      let raw = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
      static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
      let re = RE.get_or_init(|| Regex::new(r"\s+\\#\s*").unwrap());
      let mr = re.replace(&raw, ":").to_string();
      let href = format!("http://www.ams.org/mathscinet-getitem?mr={}", mr);
      Ok(stored_map!("mr" => mr, "href" => href))
    });

  // amsrefs.sty:2673-2690: the `\MR` internals a paper's own `\MR` re-dress calls (0806.1269:627-634, 1612.01009,
  // 1706.05540 and 2 more run-345 papers: `\strip@MRprefix#1\@nil`, then `\MRhref`), and `\MRhref`'s plain default;
  // with `msc-links` (which loads hyperref, above), :2933-2950's linking `\MRhref` and its `\parse@MR`.
  RawTeX!(r"\def\strip@MRprefix#1#2#3#4\@nil{\def\@tempa{#1#2#3#4}\if#1M\if#2R\def\@tempa{#3#4}\fi\fi}
\providecommand{\MRhref}[2]{#1}");
  if lookup_bool("amsrefs@msc-links") {
    RawTeX!(r"\@ifundefined{href}{}{\def\parse@MR#1 (#2)#3\@nil{\def\MR@url{http://www.ams.org/mathscinet-getitem?mr=#1}\def\@tempd{#1}\def\@tempe{#2}}\def\MRhref#1#2{\begingroup\parse@MR#1 ()\@empty\@nil\href{\MR@url}{\@tempd\vphantom{()}}\ifx\@tempe\@empty\else\ \href{\MR@url}{(\@tempe)}\fi\endgroup}}");
  }

  // \ndash, \mdash
  DefConstructor!("\\ndash", "\u{2013}"); // EN DASH
  DefConstructor!("\\mdash", "\u{2014}"); // EM DASH
});

/// The records of an amsrefs database (`.ltb`) that `\ReadBibLoop` (amsrefs.sty:1348-1369) executes: it `\read`s
/// the file a record at a time — a line, extended while its braces are open — and executes the records whose first
/// token is `\bib`; a record that starts with `\endinput` ends the file. `%` comments are dropped as TeX reads them.
fn amsrefs_database_records(text: &str) -> Vec<String> {
  let mut records = Vec::new();
  let mut record = String::new();
  let mut depth: i64 = 0;
  for line in text.lines() {
    // Strip the comment: the first `%` not escaped by a backslash.
    let mut code = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
      match c {
        '\\' => {
          code.push(c);
          if let Some(next) = chars.next() {
            code.push(next);
          }
        },
        '%' => break,
        '{' => {
          depth += 1;
          code.push(c);
        },
        '}' => {
          depth -= 1;
          code.push(c);
        },
        _ => code.push(c),
      }
    }
    record.push_str(&code);
    record.push('\n');
    if depth > 0 {
      continue;
    }
    depth = 0;
    let head = record.trim_start();
    if head.starts_with("\\endinput") && !head[9..].starts_with(|c: char| c.is_ascii_alphabetic()) {
      break;
    }
    if head.starts_with("\\bib") && !head[4..].starts_with(|c: char| c.is_ascii_alphabetic()) {
      records.push(head.to_string());
    }
    record.clear();
  }
  records
}

/// The key of a database record `\bib{key}{type}{…}` (or `\bib*{key}…`).
fn amsrefs_record_key(record: &str) -> Option<String> {
  let rest = record.strip_prefix("\\bib")?.trim_start();
  let rest = rest.strip_prefix('*').unwrap_or(rest).trim_start();
  let rest = rest.strip_prefix('{')?;
  rest.find('}').map(|end| rest[..end].trim().to_string())
}
