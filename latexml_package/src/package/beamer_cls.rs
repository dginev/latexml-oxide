//! beamer.cls — Minimal stubs for beamer presentation class
//! Perl: beamer.cls.ltxml (1364 lines)
//!
//! Provides enough definitions for the beamer test to pass without loading
//! the raw beamer.cls (which exceeds the 5M token limit). Full beamer
//! support requires porting the complete Perl binding.
use crate::prelude::*;

// ---------------------------------------------------------------------------
// beamerbasecolor.sty color model (Perl beamer.cls.ltxml L1051: a TODO stub).
// `\setbeamercolor{name}{fg=…,bg=…,parent=…,use=…}` stores the palette and
// registers the xcolor-visible names `<name>.fg` / `<name>.bg` that themes
// and documents reference directly (real beamer registers them inside
// `\usebeamercolor`, which its templates run; we register eagerly at set
// time since we do not execute templates — same observable names). Witnesses:
// beamertheme-metropolis demo (`\def\couleur{alerted text.fg}`, 41 errs),
// beamertheme-gotham examples, beamertheme-epyt, beamer-amurmaple.
fn beamer_color_key(name: &str, field: &str) -> String { s!("beamer@color@{name}@{field}") }

fn beamer_color_lookup(name: &str, field: &str) -> Option<String> {
  match lookup_value(&beamer_color_key(name, field)) {
    Some(Stored::String(sym)) => Some(with(sym, |v| v.to_string())),
    _ => None,
  }
}

/// Follow beamerbasecolor's inheritance: an empty/absent fg (bg) falls back
/// to the first parent that yields one.
fn beamer_resolve(name: &str, field: &str, depth: usize) -> Option<String> {
  if depth > 16 {
    return None;
  }
  if let Some(v) = beamer_color_lookup(name, field)
    && !v.is_empty()
  {
    return Some(v);
  }
  if let Some(parents) = beamer_color_lookup(name, "parent") {
    for parent in parents.split(',') {
      let parent = parent.trim().trim_matches(['{', '}']).trim();
      if !parent.is_empty()
        && let Some(v) = beamer_resolve(parent, field, depth + 1)
      {
        return Some(v);
      }
    }
  }
  None
}

/// Quiet probe: is `name` a defined color? (State `color_<name>` from
/// color/xcolor, or raw xcolor storage `\color@<name>` — the
/// `wisdom_xcolor_internal_storage_interop` shape.)
fn beamer_color_known(name: &str) -> bool {
  let name = name.trim();
  lookup_value(&s!("color_{name}")).is_some()
    || lookup_meaning(&T_CS!(s!("\\color@{name}"))).is_some()
}

/// A color EXPR (`A!30!B`, `-A`, `A`) is registrable when every base name it
/// references is already defined; otherwise defer (a later \setbeamercolor
/// or \usebeamercolor retries).
fn beamer_expr_defined(expr: &str) -> bool {
  let expr = expr.trim();
  if expr.is_empty() {
    return false;
  }
  for segment in expr.split('!') {
    let segment = segment.trim().trim_start_matches('-').trim();
    if segment.is_empty() || segment.chars().all(|c| c.is_ascii_digit() || c == '.') {
      continue; // mix percentage
    }
    if !beamer_color_known(segment) {
      return false;
    }
  }
  true
}

/// (Re)register the xcolor names `<name>.fg` / `<name>.bg` from the stored
/// palette, when their expressions resolve. Global, like a theme's palette.
fn beamer_register_color(name: &str) -> Result<()> {
  for field in ["fg", "bg"] {
    if let Some(expr) = beamer_resolve(name, field, 0)
      && beamer_expr_defined(&expr)
    {
      digest(Tokenize!(TeXString::assembled(format!(
        "\\xglobal\\colorlet{{{name}.{field}}}{{{expr}}}"
      ))))?;
    }
  }
  Ok(())
}

/// Split a beamer color-option list at top-level commas into (key, value).
fn beamer_color_opts(opts: &str) -> Vec<(String, String)> {
  let mut out = Vec::new();
  let mut depth = 0usize;
  let mut current = String::new();
  let mut parts = Vec::new();
  for c in opts.chars() {
    match c {
      '{' => {
        depth += 1;
        current.push(c);
      },
      '}' => {
        depth = depth.saturating_sub(1);
        current.push(c);
      },
      ',' if depth == 0 => {
        parts.push(std::mem::take(&mut current));
      },
      _ => current.push(c),
    }
  }
  parts.push(current);
  for part in parts {
    let part = part.trim();
    if part.is_empty() {
      continue;
    }
    match part.split_once('=') {
      Some((k, v)) => out.push((
        k.trim().to_string(),
        v.trim().trim_matches(['{', '}']).trim().to_string(),
      )),
      None => out.push((part.to_string(), String::new())),
    }
  }
  out
}

/// Does a beamer `<…>` specification apply in presentation mode? Parts are
/// separated by `|`; a part `mode:overlays` or bare `mode` names a mode
/// (beamer, presentation, handout, trans, article, second, all); a part with
/// no mode name is an overlay spec and applies. False only when every part
/// names a mode other than beamer/presentation/all.
fn beamer_mode_spec_applies(spec: &str) -> bool {
  const MODES: &[&str] = &[
    "beamer",
    "presentation",
    "handout",
    "trans",
    "article",
    "second",
    "all",
  ];
  let mut saw_part = false;
  for part in spec.split('|') {
    let part = part.trim();
    if part.is_empty() {
      continue;
    }
    saw_part = true;
    let mode = part.split(':').next().unwrap_or("").trim();
    if !MODES.contains(&mode) || matches!(mode, "beamer" | "presentation" | "all") {
      return true;
    }
  }
  !saw_part
}

/// Halve adjacent `Catcode::PARAM` tokens (`##` -> `#`), mirroring TeX's
/// `\def` replacement-text scanning when `\beamer@doifinframe` is stored
/// (beamerbaseframe.sty:527). Single `#` tokens are preserved intact for
/// backwards compatibility with inline macro definitions in frames.
/// In real beamer (beamerbaseframe.sty:524-529), the collected frame body is
/// processed through two nested `\def` levels:
/// 1. `\loop ... \repeat` defines `\def\iterate{...}` enclosing the body.
/// 2. `\iterate` executes `\def\beamer@doifinframe{\begin{beamer@frameslide} #1 \end{beamer@frameslide}}`.
///
/// Each `\def` level collapses double-PARAM `##` to `#`. Thus `####` in the
/// source becomes `##` in step 1, and `#` in step 2.
fn halve_frame_hashes(tokens: Vec<Token>) -> Vec<Token> {
  fn halve_once(toks: Vec<Token>) -> Vec<Token> {
    let mut halved = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
      let t = toks[i];
      if t.get_catcode() == Catcode::PARAM {
        if i + 1 < toks.len() && toks[i + 1].get_catcode() == Catcode::PARAM {
          // Double PARAM: collapse `##` to `#`
          halved.push(t);
          i += 2;
        } else {
          // Single PARAM: keep as is for backward compatibility
          halved.push(t);
          i += 1;
        }
      } else {
        halved.push(t);
        i += 1;
      }
    }
    halved
  }

  // Two \def levels: \loop's \def\iterate and \def\beamer@doifinframe
  halve_once(halve_once(tokens))
}

/// Collect raw tokens from the input stream up to the matching `\end{frame}`,
/// respecting nested `\begin{frame}...\end{frame}` depth.
fn collect_frame_body() -> Result<(Vec<Token>, Option<Vec<Token>>)> {
  let end_cs = T_CS!("\\end");
  let begin_cs = T_CS!("\\begin");
  let mut body_tokens: Vec<Token> = Vec::new();
  let mut frame_depth: usize = 1;

  while let Some(tok) = read_token()? {
    if tok == begin_cs || tok == end_cs {
      let is_begin = tok == begin_cs;
      let mut lookahead = Vec::new();
      let mut matched_frame = false;

      while let Some(space_tok) = read_token()? {
        if space_tok.get_catcode() == Catcode::SPACE {
          lookahead.push(space_tok);
        } else {
          lookahead.push(space_tok);
          break;
        }
      }

      if let Some(&first_non_space) = lookahead.last()
        && first_non_space.get_catcode() == Catcode::BEGIN
      {
        let mut name = String::new();
        while let Some(name_tok) = read_token()? {
          lookahead.push(name_tok);
          if name_tok.get_catcode() == Catcode::END {
            if name.trim() == "frame" {
              matched_frame = true;
            }
            break;
          } else {
            name.push_str(&name_tok.to_string());
          }
        }
      }

      if matched_frame {
        if is_begin {
          frame_depth += 1;
          body_tokens.push(tok);
          body_tokens.extend(lookahead);
        } else {
          frame_depth -= 1;
          if frame_depth == 0 {
            let mut end_tokens = Vec::with_capacity(1 + lookahead.len());
            end_tokens.push(tok);
            end_tokens.extend(lookahead);
            return Ok((body_tokens, Some(end_tokens)));
          } else {
            body_tokens.push(tok);
            body_tokens.extend(lookahead);
          }
        }
      } else {
        body_tokens.push(tok);
        body_tokens.extend(lookahead);
      }
    } else {
      body_tokens.push(tok);
    }
  }

  Ok((body_tokens, None))
}

#[rustfmt::skip]
LoadDefinitions!({
  // beamerbasefont.sty:322-323 `\Tiny` (4pt) / `\TINY` (3pt), the two sizes
  // below `\tiny` that beamer adds and its font themes use
  // (beamerfontthemeBerlinFU.sty:20; beamer-fuberlin, beamertheme-detlevcm,
  // URbeamer-DEMO). Same shape as the AMS classes' (ams_support_sty.rs).
  DefPrimitive!("\\TINY", None, font => { size => 3 });
  DefPrimitive!("\\Tiny", None, font => { size => 4 });

  // beamerbasetemplates.sty:24-30: `\ifbeamertemplateempty{name}{empty}
  // {nonempty}` gates control flow on whether `\beamer@@tmpl@<name>` is
  // undefined or `\beamer@@empty`; themes test it (beamerthemeAlbi.sty:224,
  // 301, 689 — 43 `\fi` cascade errors). The binding stands in for beamer
  // and had no templates surface here; real body, since it gates flow. Guard:
  // `perfect_kernel_batch54::beamer_template_empty_test_is_defined`.
  RawTeX!(
    r"\long\def\beamer@@empty{}
\def\expandbeamertemplate#1{\csname beamer@@tmpl@#1\endcsname}
\def\ifbeamertemplateempty#1#2#3{%
  \def\beamer@ifdo{#3}%
  \expandafter\ifx\csname beamer@@tmpl@#1\endcsname\relax\def\beamer@ifdo{#2}\fi%
  \expandafter\ifx\csname beamer@@tmpl@#1\endcsname\beamer@@empty\def\beamer@ifdo{#2}\fi%
  \beamer@ifdo}"
  );

  // Load article.cls as the base class (beamer builds on article; Perl
  // beamer.cls.ltxml:1361 `LoadClass('article')`). `RequirePackage!` looked
  // for an `article.sty` and missed silently, so no sectioning counter below
  // `section` existed: `\subsection` in a beamer deck was `undefined:
  // \thesubsection` (bfh-ci DEMO-BFHBeamer, metropolis/gotham demos).
  // Don't load raw beamer.cls — its expansion chains exceed the token limit.
  LoadClass!("article");
  // Perl beamer.cls.ltxml L30-32: "these packages probably aren't needed,
  // but let's load them anyways!" — graphicx especially IS needed: real
  // beamer's dependency chain provides \includegraphics, and theme demos
  // use it bare (sweep-11 cluster: 9 docs `undefined:\includegraphics`,
  // witness beamertheme-focus/focus-demo).
  RequirePackage!("ifpdf");
  RequirePackage!("keyval");
  RequirePackage!("graphicx");
  // beamer.cls:343-350 `\RequirePackage[papersize=…,hmargin=1cm,…]{geometry}`:
  // beamerposter.sty:176 calls `\geometry{…}` for poster sizes, which was
  // undefined (beamertheme-mirage posters ×2; SHARED — Perl beamer.cls.ltxml:30-32
  // omits it too). Guard: `perfect_kernel_batch56::beamer_requires_geometry`.
  RequirePackage!("geometry");
  // Real beamer requires pgfcore (beamer.cls → beamerbasemodes → pgfcore);
  // themes then use shadings/pictures directly (epyt's
  // \pgfdeclareverticalshading, gotham). Load our pgf binding so that raw
  // theme surface resolves against the real implementations.
  RequirePackage!("pgf");
  // beamerbasetranslator.sty:14 `\RequirePackage{translator}`: language
  // packs call `\uselanguage`/`\languagealias` (ctex-scheme-chinese-beamer
  // .def:71; mirage-beamer-zh). Guard: `perfect_kernel_batch56::beamer_loads_translator`.
  RequirePackage!("translator");
  // beamer.cls:32-49 + beamerbaseframecomponents.sty:22-23: the dimension
  // registers themes read (`\beamer@rightsidebar` unconditionally at
  // beamerthemeVerona.sty:287 → `Missing close parenthesis in Dimension
  // expr`; sidebar/inmargin outer themes, boxes). Real themes raw-load here
  // (beyond Perl, which no-ops `\usetheme`), so beamer.cls's own state must
  // exist. Defaults as in beamer.cls (0pt; margins 1cm).
  RawTeX!(
    r"\newdimen\headdp \newdimen\footheight \newdimen\sidebarheight
\newdimen\beamer@tempdim \newdimen\beamer@finalheight
\newdimen\beamer@animht \newdimen\beamer@animdp \newdimen\beamer@animwd
\newdimen\beamer@leftmargin \newdimen\beamer@rightmargin
\newdimen\beamer@leftsidebar \newdimen\beamer@rightsidebar
\newdimen\beamer@boxsize \newdimen\beamer@vboxoffset
\newdimen\beamer@descdefault \newdimen\beamer@descriptionwidth
\newdimen\beamer@sidebarwidth \newdimen\beamer@headheight
\newdimen\beamer@frametextheight \newdimen\beamer@boxheadheight
\newdimen\beamer@blockheadheight
\beamer@leftmargin=1cm \beamer@rightmargin=1cm
\newcount\beamer@slideinframe \beamer@slideinframe=1 "
  );
  // beamerbasesection.sty:20-22,172,220-222,250,292-294 section-name family,
  // beamerbaseframe.sty:745-746 `\framebreak`, and the internals gotham's font
  // theme `\patchcmd`s (beamerbasetitle.sty:49-84 `\beamer@title`/
  // `\beamer@subtitle`, beamerbasesection.sty:183-219 `\beamer@section`,
  // beamerbasenavigation.sty:327 `\sectionentry`): token macros carrying
  // beamer's assignment lines so the patches find their search strings (they
  // are never invoked — article sectioning stays in charge). Witnesses
  // thubeamer-example-en (`\frametitle{\secname}`), beamer-amurmaple-doc,
  // gotham-example* ("Patching title failed" ×4), tikz-relay/tikz-sfc
  // BeamerAnimation (`\beamer@slideinframe`).
  RawTeX!(
    r"\def\secname{}\def\subsecname{}\def\subsubsecname{}
\def\lastsection{}\def\lastsubsection{}

\def\insertsection{}\def\insertsectionhead{}\def\insertsubsectionhead{}\def\insertsubsubsectionhead{}
\def\insertsectionnumber{\@arabic\c@section}\def\insertsubsectionnumber{\@arabic\c@subsection}
\def\framebreak{}\def\noframebreak{}
\long\def\beamer@title[#1]#2{\def\inserttitle{#2}\def\beamer@shorttitle{#1}}
\long\def\beamer@subtitle[#1]#2{\def\insertsubtitle{#2}\def\beamer@shortsubtitle{#1}}
\def\sectionentry#1#2#3#4#5{\def\insertsectionhead{#2}\def\insertsectionheadnumber{#1}\def\insertpartheadnumber{#5}}
\long\def\beamer@section[#1]#2{\refstepcounter{section}%
  \def\insertsectionhead{\hyperlink{Navigation\the\c@page}{#1}}%
  \edef\insertsectionhead{\noexpand\hyperlink{Navigation\the\c@page}{\unexpanded{#1}}}}
\long\def\beamer@subsection[#1]#2{\refstepcounter{subsection}%
  \def\insertsubsectionhead{\hyperlink{Navigation\the\c@page}{#1}}%
  \edef\insertsubsectionhead{\noexpand\hyperlink{Navigation\the\c@page}{\unexpanded{#1}}}}
\long\def\beamer@@frametitle[#1]#2{{\gdef\insertframetitle{{#2\ifnum\beamer@autobreakcount>0\relax{}\space\usebeamertemplate*{frametitle continuation}\fi}}\gdef\beamer@frametitle{#2}\gdef\beamer@shortframetitle{#1}}}"
  );
  // The last two carry beamerbasesection.sty:283 `\beamer@subsection` and
  // beamerbaselocalstructure.sty:30 `\beamer@@frametitle`, the targets of
  // beamerfontthememetropolis.sty:278/:284/:293 `\patchcmd`s ("Patching
  // section title failed", "Patching frame title failed": metropolis demo).
  // Guard: `perfect_kernel_batch56::beamer_metropolis_font_theme_patches_apply`.

  // Perl beamer.cls.ltxml L853: DefKeyVal('beamerframe', 'fragile', '', '')
  // — declares `fragile` as a zero-argument key for the beamerframe keyset.
  // Documents using `\begin{frame}[fragile]{Title}` rely on this to parse
  // without "unknown keyval" errors. Frame env's Rust stub doesn't
  // consult keyvals yet, but the declaration itself must load.
  DefKeyVal!("beamerframe", "fragile", "");

  // Frame environment — the core beamer construct.
  // Absorbs optional overlay spec and optional title/subtitle args.
  // Perl: DefEnvironment('{frame}[][]', '<ltx:slide...>...</ltx:slide>');
  // beamerbaseframe.sty:91 `\beamer@inframetrue` inside the frame body: the
  // BFH inner theme's `\sectionpage` (beamerinnerthemeBFH.sty:438-448) opens
  // a `\frame[plain]{…}` of its own unless `\ifbeamer@inframe` — a frame
  // nested in the frame (`<ltx:subsection> isn't allowed in <ltx:subsection>`,
  // DEMO-BFHBeamer ×2; Perl never loads themes). Local, so `\end{frame}`
  // restores it.
  DefMacro!(
    "\\lx@beamer@frame@start",
    "\\csname beamer@@tmpl@background\\endcsname"
  );
  DefEnvironment!("{frame}[][]",
    "<ltx:subsection _noautoclose='1'>#body</ltx:subsection>",
    before_digest => { Let!("\\ifbeamer@inframe", "\\iftrue"); },
    after_digest_begin => sub[whatsit] {
      let is_fragile = whatsit.get_args().iter().flatten().any(|arg| {
        arg.to_string().contains("fragile")
      });
      if !is_fragile {
        let (body, end_tokens_opt) = collect_frame_body()?;
        if let Some(end_tokens) = end_tokens_opt {
          let halved = halve_frame_hashes(body);
          let mut reinject = Vec::with_capacity(1 + halved.len() + end_tokens.len());
          reinject.push(T_CS!("\\lx@beamer@frame@start"));
          reinject.extend(halved);
          reinject.extend(end_tokens);
          unread_expansion(Tokens::new(reinject));
          return Ok(Vec::new());
        } else if !body.is_empty() {
          let mut reinject = Vec::with_capacity(1 + body.len());
          reinject.push(T_CS!("\\lx@beamer@frame@start"));
          reinject.extend(body);
          unread_expansion(Tokens::new(reinject));
          return Ok(Vec::new());
        }
      }
      unread_one(T_CS!("\\lx@beamer@frame@start"));
      Ok(Vec::new())
    });
  // Beamer's COMMAND form `\frame<overlays>[<default>][options]{contents}`
  // (beamerbaseframe.sty). DefEnvironment also installs a bare `\frame` CS,
  // but that one opens the subsection and waits for an `\end{frame}` that
  // never comes — the dangling `_noautoclose` subsection then swallows every
  // later `\section`/`\subsection` (malformed:ltx beamer-sectioning family,
  // 185 errors / 40 oracle-clean docs; min-repro
  // `\section{S}\frame{f}\section{T}`). Route the braced short form through
  // the environment so the subsection closes.
  DefMacro!(
    "\\frame OptionalAngled [][] {}",
    "\\begin{frame}[#2][#3]#4\\end{frame}"
  );

  // Overlay specification commands — stub as no-ops
  // Rust's \alt{}{}/\only/\onslide/\temporal/\pause take the
  // "always-true" branch (first arg, or body) — faithful to what a
  // reader expects from beamer slides printed as a continuous
  // document. See Perl L793-834 for the full overlay/pause machinery.
  // \only<spec>{stuff} — the leading angle-spec must be CONSUMED (the old
  // `\only{}` identity ate `{stuff}` but left `<handout>` to typeset AND
  // execute never-taken branches: beamerswitch.cls L226's
  // `\only<handout>{\pgfpagesuselayout…}` ran its pgfpages payload and
  // printed `¡handout¿`; Perl routes through \beamer@ifnextcharospec,
  // beamer.cls.ltxml L745). The trailing-spec form `\only{stuff}<spec>` is
  // absorbed by the OptionalAngled tail.
  // beamer's mode specifications (beamerbasemodes.sty:33-70 `\mode<…>`, and
  // the `<…>` of `\only`/`\alt`/`\uncover`: `\beamer@masterdecode`): a spec
  // whose parts all name OTHER modes — `<handout>`, `<handout:0| trans:0>`,
  // `<article>` — makes the action a no-op in the presentation mode this
  // binding lives in (beamerswitch.cls:226 `\only<handout>{\pgfpagesuselayout…}`
  // with pgfpages unloaded; Perl beamer.cls.ltxml:435 `matchesMode` answers
  // handout=beamer, a Perl-origin bug). A part with no mode name (`<2->`) or
  // naming `beamer`/`presentation`/`all` applies.
  // Guard: `perfect_kernel_batch56::beamer_only_discards_other_mode_specs`.
  // beamerbasetitle.sty:213 `\subject{text}` = `\hypersetup{pdfsubject=…}`
  // (PDF metadata; shipunov lecture-slides-ex, beamerswitch-example).
  def_macro_noop("\\subject{}")?;
  DefMacro!("\\only OptionalAngled {} OptionalAngled", sub[args] {
    let mut it = args.into_iter();
    let spec: Option<Tokens> = it.next().unwrap().into();
    let body: Tokens = it.next().unwrap().into();
    let spec2: Option<Tokens> = it.next().unwrap().into();
    let spec = spec.or(spec2);
    let applies = spec.as_ref().is_none_or(|s| beamer_mode_spec_applies(&s.to_string()));
    Ok(if applies { body } else { Tokens!() })
  });
  // pgfpages.sty `\pgfpagesuselayout{layout}[options]` (beamerswitch.cls:226
  // `\handoutlayout`): page-imposition layout, no XML.
  def_macro_noop("\\pgfpagesuselayout{}[]")?;
  def_macro_noop("\\onslide")?;
  DefMacro!("\\temporal OptionalAngled {}{}{}", "#3");
  def_macro_noop("\\pause")?;
  DefMacro!("\\alt OptionalAngled {}{} OptionalAngled", "#2");

  // Perl beamer.cls.ltxml L796-798 dispatches \visible/\uncover/
  // \invisible via \alt to the \beamer@{visible,uncovered,…}
  // inline-block markers, but that routing needs the BeamerAngled
  // parameter type + \beamer@ifnextcharospec overlay dispatcher Rust
  // hasn't ported. Keep the body-passthrough stubs for now — the
  // markers below are still defined and usable directly by advanced
  // beamer styles that invoke them without angle-spec preprocessing.
  DefMacro!("\\visible OptionalAngled {}", "#2");
  DefMacro!("\\uncover OptionalAngled {}", "#2");
  DefMacro!("\\invisible OptionalAngled {}", "");

  DefMacro!(
    "\\beamer@visible{}",
    "\\beamer@visible@begin{#1}\\beamer@visible@end"
  );
  DefConstructor!(
    "\\beamer@visible@begin",
    "<ltx:inline-block class='ltx_visible'>"
  );
  DefConstructor!("\\beamer@visible@end", "</ltx:inline-block>");

  DefMacro!(
    "\\beamer@invisible{}",
    "\\beamer@invisible@begin{#1}\\beamer@invisible@end"
  );
  DefConstructor!(
    "\\beamer@invisible@begin",
    "<ltx:inline-block class='ltx_invisible'>"
  );
  DefConstructor!("\\beamer@invisible@end", "</ltx:inline-block>");

  DefMacro!(
    "\\beamer@uncovered{}",
    "\\beamer@uncovered@begin{#1}\\beamer@uncovered@end"
  );
  DefConstructor!(
    "\\beamer@uncovered@begin",
    "<ltx:inline-block class='ltx_uncovered'>"
  );
  DefConstructor!("\\beamer@uncovered@end", "</ltx:inline-block>");

  DefMacro!(
    "\\beamer@covered{}",
    "\\beamer@covered@begin{#1}\\beamer@covered@end"
  );
  DefConstructor!(
    "\\beamer@covered@begin",
    "<ltx:inline-block class='ltx_covered'>"
  );
  DefConstructor!("\\beamer@covered@end", "</ltx:inline-block>");

  DefMacro!(
    "\\beamer@alerted{}",
    "\\beamer@alerted@begin{#1}\\beamer@alerted@end"
  );
  DefConstructor!(
    "\\beamer@alerted@begin",
    "<ltx:inline-block class='ltx_alert'>"
  );
  DefConstructor!("\\beamer@alerted@end", "</ltx:inline-block>");

  // Frame structure
  DefMacro!("\\frametitle OptionalAngled []{}", "\\par\\textbf{#3}\\par");
  def_macro_noop("\\framesubtitle OptionalAngled {}")?;

  // Perl beamer.cls.ltxml L961-963: internal frame title constructors
  // that \frame@ / \beamer@frame@replay invoke to lift title/subtitle
  // onto the enclosing slide element via `^` float-to-parent. Rust
  // \frame stubs as ltx:subsection, so `^<ltx:title>` floats to that.
  // Unported until now, so beamer themes that invoke
  // \beamer@frametitle{...} directly (bypassing \frametitle) hit
  // undefined-CS errors. The three constructors all carry the same
  // float semantics, differing only in element (title vs subtitle)
  // and CSS class.
  DefConstructor!(
    "\\beamer@frametitle{}",
    "^<ltx:title class='ltx_frame_title'>#1</ltx:title>"
  );
  DefConstructor!(
    "\\beamer@frameshorttitle{}",
    "^<ltx:title class='ltx_frame_shorttitle'>#1</ltx:title>"
  );
  DefConstructor!(
    "\\beamer@framesubtitle{}",
    "^<ltx:subtitle class='ltx_frame_subtitle'>#1</ltx:subtitle>"
  );

  // beamerbaseframe.sty:730 `\newcounter{framenumber}` (Perl beamer.cls.ltxml:933
  // `NewCounter('framenumber')`): appendixnumberbeamer.sty:43 reads
  // `\theframenumber`/`\value{framenumber}` (metropolis, gotham, pure-minimalistic
  // demos). The stub frame never steps it, so `\insertframenumber` stays empty.
  RawTeX!(r"\newcounter{framenumber}");
  // beamerbasemisc.sty:25 `\headcommand` — navigation bookkeeping, not rendered
  // (appendixnumberbeamer.sty:46 calls it directly; Perl errors too).
  def_macro_noop("\\headcommand{}")?;
  // Insert counters
  def_macro_noop("\\insertframenumber")?;
  def_macro_noop("\\insertslidenumber")?;
  def_macro_noop("\\insertpagenumber")?;
  def_macro_noop("\\insertoverlaynumber")?;

  // Overlay environments
  DefEnvironment!("{onlyenv}", "#body");
  DefEnvironment!("{altenv}{}{}{}{}", "#body");
  DefEnvironment!("{alertenv}", "#body");
  DefEnvironment!("{uncoverenv}", "#body");
  DefEnvironment!("{actionenv}", "#body");
  DefEnvironment!("{visibleenv}", "#body");
  DefEnvironment!("{invisibleenv}", "#body");
  DefEnvironment!("{overlayarea}{}{}", "#body");
  DefEnvironment!("{overprint}", "#body");

  // Block environments — Perl L1189 beamerbaseblocks.sty
  DefEnvironment!(
    "{block} OptionalAngled {}",
    "<ltx:theorem class='ltx_theorem_block'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>"
  );
  DefEnvironment!(
    "{alertblock} OptionalAngled {}",
    "<ltx:theorem class='ltx_theorem_alertblock'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>"
  );
  DefEnvironment!(
    "{exampleblock} OptionalAngled {}",
    "<ltx:theorem class='ltx_theorem_exampleblock'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>"
  );

  // Columns environment — Perl L1230-1240 beamerbaseboxes.sty
  DefEnvironment!("{columns} OptionalAngled []", "#body");
  DefEnvironment!("{column} OptionalAngled {}", "#body");
  // beamerbasecolor.sty:149 `{beamercolorbox}[keys]{color}` — the coloured box
  // every theme builds on (mirage's `pullquote`, beamerthemeMirage.sty:82-92);
  // article mode (:241) collapses it to a transparent pair, the shape a class
  // loaded on article takes here. Perl no-ops `\usetheme` and never meets it.
  // `{beamerboxesrounded}` (beamerbaseboxes.sty:36-138) stays UNDEFINED on
  // purpose: themes `\xpatchcmd` its macro body (beamerthemeMirage.sty:35-37),
  // and a constructor definition is not a body xpatch can re-scan (`\scan…`
  // split + `_` errors); undefined, xpatch's failure branch is silent.
  // Guard: `perfect_kernel_batch56::beamer_theme_compat_aliases_and_colorbox`.
  DefEnvironment!(
    "{beamercolorbox} [] {}",
    "<ltx:inline-block class='ltx_beamercolorbox'>#body</ltx:inline-block>"
  );
  def_macro_noop("\\column OptionalAngled {}")?;

  // Title page macros — Perl L1010-1035
  DefMacro!(
    "\\institute OptionalAngled []{}",
    "\\@add@frontmatter{ltx:creator}{\\@@@affiliation{#3}}"
  );
  // beamerbasetitle.sty:148/169 and :233/238: `\inst{n}` is defined locally
  // inside `\insertauthor`/`\insertinstitute` as the superscript affiliation
  // mark; our `\author`/`\institute` digest their argument at once, where it
  // was `undefined:\inst` (beamertheme-detlevcm, beamerstructure2; Perl too).
  DefMacro!("\\inst{}", "\\textsuperscript{#1}");
  // The constructor \institute expands into was never defined here — every
  // beamer doc using \institute logged `undefined:\@@@affiliation` (sweep-11
  // cluster: 16 docs, witness beamerthemeconcrete/demo-cbernoulli). Same
  // ltx:contact form as elsart_support_core / cas_dc_cls.
  DefConstructor!(
    "\\@@@affiliation{}",
    "^ <ltx:contact role='affiliation'>#1</ltx:contact>"
  );
  // \logo{content} and \titlegraphic{content} typically wrap
  // \includegraphics or similar visual content. Surpass Perl
  // (which doesn't define them) by routing to ltx:note so any
  // \includegraphics inside resolves and the graphic is preserved.
  DefMacro!("\\logo{}", "\\@add@frontmatter{ltx:note}[role=logo]{#1}");
  // beamerbasetitle.sty:199-200: `\titlegraphic` STORES into
  // `\inserttitlegraphic` (placed only by the title-page template). The old
  // note emission digested the argument at once — beamerthemeVerona.sty:169
  // wraps it in a bare tikz `\node…;` meant for the title-page picture
  // (`\node` undefined, beamer-verona ×2).
  RawTeX!(r"\def\inserttitlegraphic{}\long\def\titlegraphic#1{\def\inserttitlegraphic{#1}}");
  // Beamer's \titlepage lives INSIDE a frame (ltx:subsection with
  // _noautoclose), where the schema forbids ltx:subtitle/ltx:date — plain
  // \maketitle flushed the frontmatter "here" and produced the sweep-12
  // `malformed:ltx:subtitle` cluster (12 docs; witnesses
  // beamerthemecelestia demos, beamerthemeNord, beamertheme-simpleplus).
  // Route through the document-top fallback placement (legal for the full
  // FrontMatter group) instead. The faithful Perl frame model (ltx:slide in
  // ltx:slidesequence) is the tracked follow-up in CLUSTERS.md.
  DefMacro!("\\titlepage", "\\lx@frontmatter@fallback");
  // Beamer docs call \maketitle inside frames too (the Celestia demos'
  // `\begin{frame}\maketitle\end{frame}`) — route it the same way.
  DefMacro!("\\maketitle", "\\lx@frontmatter@fallback");
  // beamerbasetitle.sty L31-32: divider-slide templates
  // (beamerinnerthemedefault.sty L114-140 render the section name).
  // SHARED-FAILURE with Perl (no definition there either); absorb for now —
  // semantic \insertsection mapping is the follow-up. 9-doc cluster,
  // witnesses beamerauxtheme examples, bfh-ci DEMO-BFHBeamer.
  def_macro_noop("\\sectionpage")?;
  def_macro_noop("\\partpage")?; // beamerbasetitle.sty:30 — the part is already emitted by \part
  def_macro_noop("\\subsectionpage")?;
  def_macro_noop("\\insertauthor")?;
  def_macro_noop("\\inserttitle")?;
  def_macro_noop("\\insertsubtitle")?;
  def_macro_noop("\\insertdate")?;
  def_macro_noop("\\insertinstitute")?;
  def_macro_noop("\\insertshortauthor[]")?;
  def_macro_noop("\\insertshortdate[]")?;
  def_macro_noop("\\insertshortinstitute[]")?;
  def_macro_noop("\\insertshorttitle[]")?;
  def_macro_noop("\\insertshortpart[]")?;
  def_macro_noop("\\insertshortsubtitle[]")?;
  def_macro_noop("\\inserttotalframenumber")?;
  def_macro_noop("\\insertmainframenumber")?;
  def_macro_noop("\\insertappendixframenumber")?;

  // Perl L1013-1045 beamerTODO navigation + page-range \insert*s.
  // All are stomach-time no-ops under Rust's continuous-document
  // rendering (beamer's slide-tracking state machine is not ported).
  // Shipping the stubs prevents undefined-CS errors in beamer themes
  // that reference them via `\setbeamertemplate{footline}` bodies.
  def_macro_noop("\\insertnavigation{}")?;
  def_macro_noop("\\insertsectionnavigation{}")?;
  def_macro_noop("\\insertsectionnavigationhorizontal{}{}{}")?;
  def_macro_noop("\\insertsubsectionnavigation{}")?;
  def_macro_noop("\\insertsubsectionnavigationhorizontal{}{}{}")?;
  def_macro_noop("\\insertverticalnavigation{}")?;
  def_macro_noop("\\insertsubsection")?;
  def_macro_noop("\\insertsubsubsection")?;
  def_macro_noop("\\insertframestartpage")?;
  def_macro_noop("\\insertframeendpage")?;
  def_macro_noop("\\insertsubsectionstartpage")?;
  def_macro_noop("\\insertsubsectionendpage")?;
  def_macro_noop("\\insertsectionstartpage")?;
  def_macro_noop("\\insertsectionendpage")?;
  def_macro_noop("\\insertpartstartpage")?;
  def_macro_noop("\\insertpartendpage")?;
  def_macro_noop("\\insertpresentationstartpage")?;
  def_macro_noop("\\insertpresentationendpage")?;
  def_macro_noop("\\insertappendixstartpage")?;
  def_macro_noop("\\insertappendixendpage")?;
  def_macro_noop("\\insertdocumentstartpage")?;
  def_macro_noop("\\insertdocumentendpage")?;

  // Theme commands — Perl L1246-1253 noops them (beamerTODO). With the
  // color model above, raw theme files load usefully: their palette
  // (\setbeamercolor) and \definecolor calls register the very names the
  // demo documents reference (epyt acolor1-5, amurmaple AmurmapleRed,
  // gotham/metropolis structure colors). Template/geometry internals in the
  // themes are absorbed by the surface noops below.
  // beamerbasethemes.sty:18-29: `\usetheme[opts]{names}` (and the colour/
  // font/inner/outer siblings) = `\beamer@calltheme{opts}{names}{prefix}`
  // → `\usepackage[{opts}]{prefix<name>}` per name — the theme options are
  // ORDINARY PACKAGE OPTIONS, which the theme's `\ProcessOptionsBeamer`
  // (beamerbaseoptions.sty:15-29) applies with `\setkeys{\@currname}` over
  // its `\DeclareOptionBeamer` keys. The former loader dropped the options
  // and no-op'd `\ProcessOptionsBeamer`, so `\usetheme[sidebar]{Verona}` never
  // set `\ifbeamer@sidebar` and the theme installed its "defined only with
  // the 'sidebar' option" stub (beamer-verona-sidebar; Perl no-ops
  // `\usetheme`). Guard: `perfect_kernel_batch54::usetheme_options_reach_the_theme`.
  // beamerbasecompatibility.sty:510-583, 620-635 verbatim (`\newcommand` →
  // `\providecommand`): the `\beamertemplate…` aliases old themes still call
  // (beamerouterthememiniframes.sty:49 `\beamertemplatedotitem` via mirage's
  // `\usetheme[compress]{Singapore}`, shipunov lecture-slides
  // `\beamertemplatearticlebibitems`). Each reduces to `\setbeamertemplate`/
  // `\setbeamercovered`/`\setbeamercolor`, all present above.
  RawTeX!(
    r"\def\beamertemplatedefaulttoc{\setbeamertemplate{sections/subsections in toc}[default]}
\def\beamertemplatenumberedsubsectiontoc{\setbeamertemplate{sections/subsections in toc}[subsections numbered]}
\def\beamertemplatenumberedsectiontoc{\setbeamertemplate{sections/subsections in toc}[sections numbered]}
\def\beamertemplatenumberedcirclesectiontoc{\setbeamertemplate{sections/subsections in toc}[circle]}
\def\beamertemplatenumberedsquaresectiontoc{\setbeamertemplate{sections/subsections in toc}[square]}
\def\beamertemplatenumberedballsectiontoc{\setbeamertemplate{sections/subsections in toc}[ball]}
\def\beamertemplateballtoc{\setbeamertemplate{sections/subsections in toc}[ball unnumbered]}
\def\beamertemplatedotitem{\setbeamertemplate{itemize items}[circle]}
\def\beamertemplatetriangleitem{\setbeamertemplate{itemize items}[default]}
\def\beamertemplatesquareitem{\setbeamertemplate{itemize items}[square]}
\def\beamertemplateballitem{\setbeamertemplate{items}[ball]}
\def\beamertemplateenumeratealpha{\setbeamertemplate{enumerate items}[default]}
\def\beamertemplateenumeratecircle{\setbeamertemplate{enumerate items}[circle]}
\def\beamertemplateenumeratesquare{\setbeamertemplate{enumerate items}[square]}
\def\beamertemplatelargepartpage{\setbeamertemplate{part page}[default]}
\def\beamertemplateboldpartpage{\setbeamerfont{part}{size=\large}%
\setbeamerfont{part name}{series=\bfseries}%
\setbeamertemplate{part page}[default]}
\def\beamertemplatelargetitlepage{\setbeamertemplate{title page}[default]}
\def\beamertemplateboldtitlepage{\setbeamerfont{title}{size=\large,series=\bfseries}%
\setbeamertemplate{title page}[default]}
\def\beamertemplateboldcenterframetitle{\setbeamerfont{frametitle}{size={},series=\bfseries}
\setbeamerfont{framesubtitle}{series=\bfseries}
\setbeamertemplate{frametitle}[default][center]}
\def\beamertemplateboldframetitle{\setbeamerfont{frametitle}{series=\bfseries}
\setbeamerfont{framesubtitle}{series=\bfseries}
\setbeamertemplate{frametitle}[default]}
\def\beamertemplatelargeframetitle{\setbeamertemplate{frametitle}[default]}
\def\beamertemplateboldblocks{\setbeamerfont{block title}{size={},series=\bfseries}\setbeamertemplate{blocks}[default]}
\def\beamertemplatelargeblocks{\setbeamertemplate{blocks}[default]}
\def\beamertemplateshadowblocks{\setbeamertemplate{blocks}[rounded][shadow=true]}
\def\beamertemplateplaintoc{\beamertemplatedefaulttoc}
\def\beamertemplatecircleminiframeinverted{\beamertemplatecircleminiframe%
\usebeamercolor{mini frames}{use=structure,fg=white,bg=structure.bg}}
\def\beamertemplatesphereminiframe{\beamertemplatecircleminiframe}
\def\beamertemplatesphereminiframeinverted{\beamertemplatecircleminiframeinverted}
\def\beamertemplatelightsectionheads{\usesectionheadtemplate{\hfill\insertsectionhead}{\hfill\color{fg!50!bg}\insertsectionhead}
\usesubsectionheadtemplate{\insertsubsectionhead}{\color{fg!50!bg}\insertsubsectionhead}}
\let\beamertemplatedarksectionheads=\beamertemplatedefaultsectionheads
\def\usecontinuationtemplate{\setbeamertemplate{frametitle continuation}}
\def\beamertemplatecontinuationroman{\setbeamertemplate{frametitle continuation}[default]}
\def\beamertemplatecontinuationtext{\setbeamertemplate{frametitle continuation}[from second]}
\def\beamertemplateroundedbuttons{\setbeamertemplate{button}[default]}
\def\beamertemplateoutlinebuttons{\setbeamercolor*{button border}{parent=structure,use=structure,fg=structure.fg!50!bg}}
\def\beamertemplatesolidbuttons{\setbeamercolor*{button}{use=structure,bg=structure.fg!50!bg,fg=white}
\setbeamercolor*{button border}{use=structure,fg=structure.fg!50!bg}}
\def\usetemplatequotation#1#2{
\setbeamertemplate{quotation begin}{#1}
\setbeamertemplate{quote begin}{#1}
\setbeamertemplate{quotation end}{#2}
\setbeamertemplate{quote end}{#2}}
\def\beamertemplateheadempty{\setbeamertemplate{headline}{}}
\def\beamertemplatefootempty{\setbeamertemplate{footline}{}}
\def\beamertemplatefootpagenumber{\setbeamertemplate{footline}[page number]}
\def\beamertemplatecaptionownline{\setbeamertemplate{caption}[caption name own line]}
\def\beamertemplatecaptionnwithnumber{\setbeamertemplate{caption}[numbered]}
\def\beamertemplateroundedblocks{\setbeamertemplate{blocks}[rounded]}
\def\beamertemplatetheoremsunnumbered{\setbeamertemplate{theorems}[default]}
\def\beamertemplatetheoremsnumbered{\setbeamertemplate{theorems}[numbered]}
\def\beamertemplatetheoremsamslike{\setbeamertemplate{theorems}[ams style]}
\def\beamertemplatetheoremssimple{\setbeamertemplate{theorems}[normal font]}
\def\beamertemplatenavigationsymbolsempty{\setbeamertemplate{navigation symbols}{}}
\def\beamertemplatenavigationsymbolsframe{\setbeamertemplate{navigation symbols}[only frame symbol]}
\def\beamertemplatenavigationsymbolsvertical{\setbeamertemplate{navigation symbols}[vertical]}
\def\beamertemplatenavigationsymbolshorizontal{\setbeamertemplate{navigation symbols}[horizontal]}
\def\beamertemplatedefaultsectionheads{%
\setbeamertemplate{section in head/foot}[default]
\setbeamertemplate{subsection in head/foot}[default]
\setbeamertemplate{section in head/foot shaded}[default]
\setbeamertemplate{subsection in head/foot shaded}[default]
}
\def\beamertemplatecircleminiframe{\setbeamertemplate{mini frames}[default]}
\def\beamertemplateticksminiframe{\setbeamertemplate{mini frames}[tick]}
\def\beamertemplateboxminiframe{\setbeamertemplate{mini frames}[box]}
\providecommand{\beamertemplategridbackground}[1][]{\setbeamertemplate{background}[grid][step={#1}]}
\providecommand{\beamertemplateshadingbackground}[2]{\setbeamertemplate{background canvas}[vertical shading][bottom={#1},top={#2}]}
\providecommand\usealerttemplate[2]{\setbeamertemplate{alerted text begin}{#1}\setbeamertemplate{alerted text end}{#2}}
\providecommand\usestructuretemplate[2]{\setbeamertemplate{structure begin}{#1}\setbeamertemplate{structure end}{#2}}
\providecommand{\beamertemplatebookbibitems}{\setbeamertemplate{bibliography item}[book]}
\providecommand{\beamertemplatearticlebibitems}{\setbeamertemplate{bibliography item}[article]}
\providecommand{\beamertemplatetextbibitems}{\setbeamertemplate{bibliography item}[text]}
\providecommand{\beamertemplatearrowbibitems}{\setbeamertemplate{bibliography item}[triangle]}
\providecommand{\beamertemplateonlinebibitems}{\setbeamertemplate{bibliography item}[online]}
\def\beamertemplatetransparentcovereddynamic{\setbeamercovered{highly dynamic}}
\def\beamertemplatetransparentcovereddynamicmedium{\setbeamercovered{dynamic}}
\def\beamertemplatetransparentcovered{\setbeamercovered{transparent}}
\def\beamertemplatetransparentcoveredmedium{\setbeamercovered{transparent=10}}
\def\beamertemplatetransparentcoveredhigh{\setbeamercovered{transparent=5}}
\def\beamertemplatetransparentcoveredhighest{\setbeamercovered{transparent=2}}"
  );
  RawTeX!(
    r"\def\beamer@calltheme#1#2#3{\def\beamer@themelist{#2}\@for\beamer@themename:=\beamer@themelist\do{\usepackage[{#1}]{#3\beamer@themename}}}
\newcommand*\usetheme[2][]{\beamer@calltheme{#1}{#2}{beamertheme}}
\newcommand*\usecolortheme[2][]{\beamer@calltheme{#1}{#2}{beamercolortheme}}
\newcommand*\usefonttheme[2][]{\beamer@calltheme{#1}{#2}{beamerfonttheme}}
\newcommand*\useinnertheme[2][]{\beamer@calltheme{#1}{#2}{beamerinnertheme}}
\newcommand*\useoutertheme[2][]{\beamer@calltheme{#1}{#2}{beameroutertheme}}
\def\ProcessOptionsBeamer{\let\@tempa\@empty
  \ifx\@currext\@clsextension\else
  \@for\CurrentOption:=\@classoptionslist\do{\@ifundefined{KV@\@currname @\CurrentOption}{}{\edef\@tempa{\@tempa,\CurrentOption,}}}\fi
  \edef\@tempa{\noexpand\setkeys{\@currname}{\@tempa\@ptionlist{\@currname.\@currext}}}\@tempa}"
  );
  // Theme-file surface the raw loads touch.
  // beamerbaseoptions.sty:34-38: the theme option layer is keyval —
  // `\DeclareOptionBeamer{key}[default]{code}` → `\define@key{\@currname}`
  // and `\ExecuteOptionsBeamer{…}` → `\setkeys{\@currname}`. The raw themes
  // define their internals through the DEFAULTS: beamerouterthemesidebar.sty:30-32
  // `\DeclareOptionBeamer{left}{\def\beamer@sidebarside{left}}` +
  // `\ExecuteOptionsBeamer{left}`, beamerinnerthemerounded.sty:11-12
  // (`\beamer@themerounded@shadow`); as no-ops, every Berkeley/Madrid/
  // sidebar/rounded document errored `undefined:\beamer@sidebarside` (8 docs,
  // sweep 30). `\ProcessOptionsBeamer` stays a no-op (`\usetheme` drops the
  // user options). Perl no-ops `\usetheme` itself.
  // beamer.cls:144-156: `\beamer@size` = the size .clo the class inputs at
  // :363; themes read it (beamerthemeAlbi.sty:192 `size/.expanded=\beamer@size`
  // as a pgfkeys choice). The class options set it; here the default plus the
  // same option keywords through the `\define@key` remap below. Guard:
  // `perfect_kernel_batch54::beamer_size_option_is_recorded`.
  RawTeX!(r"\def\beamer@size{{size11.clo}}");
  RawTeX!(concat!(
    r"\def\DeclareOptionBeamer#1{\@ifnextchar[{\beamer@dokv{#1}}{\beamer@dokv{#1}[]}}",
    r"\long\def\beamer@dokv#1[#2]#3{\define@key{\@currname}{#1}[{#2}]{#3}}",
    r"\def\ExecuteOptionsBeamer#1{\setkeys{\@currname}{#1}}"
  ));
  RawTeX!(concat!(
    r"\DeclareOptionBeamer{bigger}{\def\beamer@size{{size12.clo}}}",
    r"\DeclareOptionBeamer{smaller}{\def\beamer@size{{size10.clo}}}",
    r"\DeclareOptionBeamer{8pt}{\def\beamer@size{{size8.clo}}}",
    r"\DeclareOptionBeamer{9pt}{\def\beamer@size{{size9.clo}}}",
    r"\DeclareOptionBeamer{10pt}{\def\beamer@size{{size10.clo}}}",
    r"\DeclareOptionBeamer{11pt}{\def\beamer@size{{size11.clo}}}",
    r"\DeclareOptionBeamer{12pt}{\def\beamer@size{{size12.clo}}}",
    r"\DeclareOptionBeamer{14pt}{\def\beamer@size{{size14.clo}}}",
    r"\DeclareOptionBeamer{17pt}{\def\beamer@size{{size17.clo}}}",
    r"\DeclareOptionBeamer{20pt}{\def\beamer@size{{size20.clo}}}"
  ));
  def_macro_noop("\\defbeamertemplateparent{}[]{}[]")?;
  def_macro_noop("\\defbeamertemplatealias{}{}{}")?;
  DefConditional!("\\ifbeamer@compress");
  // Beamer's full `\newif` surface (grep `\newif\if…` over the real beamer
  // sources). Third-party themes read these inside skipped conditional
  // branches; an UNDEFINED `\if…` there is invisible to the meaning-counting
  // body skipper (tex.web §366 semantics, Conditional.pm:117), so its paired
  // `\fi` closes the OUTER frame early and the real `\else`/`\fi` surface as
  // orphans (beamerthemeCelestia.sty L467 `\ifbeamer@plainframe`, witness
  // Celestia-demo-*). Initial states mirror beamer's own: `\newif` = false,
  // then the true-setters beamer runs unconditionally (beamerbasemodes.sty
  // L19-21 blocks/ams/amssymb + keywords, L47 inpresentation;
  // beamerbasesection.sty L32 inlecture; beamer.cls L106 notesnormals;
  // presentation-mode suppressreplacements).
  RawTeX!(
    r"\newif\ifbeamer@altmode \newif\ifbeamer@ams \newif\ifbeamer@amssymb
      \newif\ifbeamer@anotherslide \newif\ifbeamer@articleactive
      \newif\ifbeamer@articlehyperref \newif\ifbeamer@articleutf
      \newif\ifbeamer@articlexcolor \newif\ifbeamer@autobreak
      \newif\ifbeamer@autopdfinfo \newif\ifbeamer@blocks
      \newif\ifbeamer@centered \newif\ifbeamer@colbox@ignorebg
      \newif\ifbeamer@colbox@rounded \newif\ifbeamer@colheight
      \newif\ifbeamer@containsverbatim \newif\ifbeamer@countsect
      \newif\ifbeamer@dogeometry \newif\ifbeamer@dosecondmode
      \newif\ifbeamer@draftmode \newif\ifbeamer@frameswithnotesonly
      \newif\ifbeamer@ignorenonframe \newif\ifbeamer@inappendix
      \newif\ifbeamer@inlecture \newif\ifbeamer@inpresentation
      \newif\ifbeamer@isfragile \newif\ifbeamer@keywords
      \newif\ifbeamer@localanotherslide \newif\ifbeamer@noframenumbering
      \newif\ifbeamer@notes \newif\ifbeamer@notesnormals
      \newif\ifbeamer@onlytextwidth \newif\ifbeamer@pausesections
      \newif\ifbeamer@pausesubsections \newif\ifbeamer@plainframe
      \newif\ifbeamer@plusencountered \newif\ifbeamer@sansmath
      \newif\ifbeamer@sb@subsection \newif\ifbeamer@sbt
      \newif\ifbeamer@secheader \newif\ifbeamer@shrink
      \newif\ifbeamer@sidebardark \newif\ifbeamer@sidebartab
      \newif\ifbeamer@slidehaszoom \newif\ifbeamer@suppressreplacements
      \newif\ifbeamer@theme@subsection \newif\ifbeamer@tree@showhooks
      \newif\ifbeamer@twoscreensnotes \newif\ifbeamer@twoscreenstext
      \beamer@blockstrue \beamer@amstrue \beamer@amssymbtrue
      \beamer@keywordstrue \beamer@inpresentationtrue \beamer@inlecturetrue
      \beamer@notesnormalstrue \beamer@suppressreplacementstrue
      \beamer@theme@subsectiontrue"
  );
  def_macro_noop("\\usebeamertemplate OptionalMatch:* OptionalMatch:* OptionalMatch:* {}")?;
  def_macro_noop("\\usebeamerfont OptionalMatch:* {}")?;
  def_macro_noop("\\setbeamertemplate{}{}")?;
  DefPrimitive!("\\setbeamercolor OptionalMatch:* {}{}", sub[(star, name, opts)] {
    let name = do_expand(name)?.to_string().trim().to_string();
    let opts = do_expand(opts)?.to_string();
    if star.is_some() {
      // Starred form RESETS the entry before applying (beamerbasecolor).
      for field in ["fg", "bg", "parent"] {
        assign_value(&beamer_color_key(&name, field), Stored::String(pin("")), Some(Scope::Global));
      }
    }
    for (key, val) in beamer_color_opts(&opts) {
      match key.as_str() {
        "fg" | "bg" | "parent" => {
          assign_value(&beamer_color_key(&name, &key), Stored::String(pin(&val)), Some(Scope::Global));
        },
        // `use=` ensures the referenced palette entries are computed before
        // this one's expressions are evaluated.
        "use" => {
          for used in val.split(',') {
            let used = used.trim().trim_matches(['{', '}']).trim();
            if !used.is_empty() {
              beamer_register_color(used)?;
            }
          }
        },
        _ => {},
      }
    }
    beamer_register_color(&name)?;
  });
  def_macro_noop("\\setbeamerfont{}{}")?;
  def_macro_noop("\\setbeamersize{}")?;
  def_macro_noop("\\setbeamercovered{}")?;
  DefPrimitive!("\\addtobeamertemplate{}{}{}", sub[(name, pre, post)] {
    let name = do_expand(name)?.to_string().trim().to_string();
    let cs = T_CS!(s!("\\beamer@@tmpl@{name}"));
    let mut new_body = pre.unlist();
    if let Ok(Some(defn)) = lookup_definition(&cs)
      && let Some(ExpansionBody::Tokens(toks)) = defn.get_expansion()
    {
      new_body.extend(toks.clone().unlist());
    }
    new_body.extend(post.unlist());
    def_macro(cs, None, ExpansionBody::Tokens(Tokens::new(new_body)), None)?;
    Ok(Vec::new())
  });
  // `\defbeamertemplate*{name}{option}[args]...{body}`: we do not execute
  // templates, but the DECLARATION must register beamer's existence marker
  // `\beamer@@tmpop@<name>@<option>` (beamerbasetemplates.sty L59) — themes
  // (gotham) probe it from `\setbeamertemplate{name}[option]` and error
  // "template ... does not exist" otherwise.
  DefPrimitive!("\\defbeamertemplate OptionalMatch:* {}{}[][]{}", sub[(_star, name, option, _n, _od, _body)] {
    let name = do_expand(name)?.to_string().trim().to_string();
    let option = do_expand(option)?.to_string().trim().to_string();
    let marker = T_CS!(s!("\\beamer@@tmpop@{name}@{option}"));
    if lookup_meaning(&marker).is_none() {
      def_macro(marker, None, ExpansionBody::Tokens(Tokens!()), None)?;
    }
  });
  DefConditional!("\\ifbeamer@inframe");

  // The default palette: real beamer's beamercolorthemedefault.sty is plain
  // `\setbeamercolor` calls — raw-load it through our implementation above so
  // `normal text.fg`, `alerted text.fg`, `structure.fg`, … exist exactly as
  // beamer defines them (beamer.cls defines beamer@blendedblue first).
  // Navigation/footline/headline — no-ops
  def_macro_noop("\\beamertemplatenavigationsymbolsempty")?;
  DefMacro!("\\beamerbutton{}", "#1");
  DefMacro!("\\beamergotobutton{}", "#1");
  DefMacro!("\\beamerskipbutton{}", "#1");
  DefMacro!("\\beamerreturnbutton{}", "#1");
  def_macro_noop("\\hypersetup{}")?;

  // Beamer list environments — Perl L1160-1179
  DefEnvironment!("{itemize} OptionalAngled",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    mode => "internal_vertical", locked => true);
  DefEnvironment!("{enumerate} OptionalAngled []",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    mode => "internal_vertical");
  // Perl beamer.cls.ltxml L1174-1179: description's \item[label] renders
  // labels via \makelabel which beamer rebinds to \descriptionlabel
  // (defined in ams_support_sty:188 as bold+space). Same pattern
  // enumitem_sty:444 and ieeetran_cls:287 use.
  DefEnvironment!("{description} OptionalAngled",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    mode => "internal_vertical", locked => true);

  // Theorems — Perl L1193-1230
  // Perl L412/L1054 parity: etoolbox + xcolor (beamer really uses xxcolor;
  // xcolor supplies \colorlet etc. — cursolatex witness).
  RequirePackage!("etoolbox");
  // beamerbasecompatibility.sty:309 (graphbox.sty:122 `\fibox@includegraphics`
  // tests its overlay spec with it).
  Let!("\\beamer@ifempty", "\\ifblank");
  RequirePackage!("xcolor");
  // beamerbaseoverlay.sty:597 `\renewcommand<>{\color}{\alt#1{\beameroriginal
  // {\color}}{\beamer@gobbleoptionalinsp}}` and the same wrapper for the
  // `\text<font>` commands (:590-596): each takes an `<overlay>` spec first
  // (Perl beamer.cls.ltxml:1345-1356 `%BEAMER_WRAPPED`). Without it `\color
  // <2>{red}` read `<` as the color ("Can't find color named '<'",
  // xskak_and_beamer 34 errors). The overlay is taken as always-active, as
  // for `\only`. Guard: `perfect_kernel_batch54::beamer_color_and_text_commands_take_an_overlay`.
  for cs in [
    "color",
    "textbf",
    "textit",
    "textmd",
    "textnormal",
    "textrm",
    "textsc",
    "textsf",
    "textsl",
    "texttt",
    "textup",
  ] {
    let orig = T_CS!(s!("\\beamer@original@{cs}"));
    let_i(&orig, &T_CS!(s!("\\{cs}")), None);
    let params = parse_parameters("OptionalAngled", &T_CS!(s!("\\{cs}")), true)?;
    def_macro(T_CS!(s!("\\{cs}")), params, Tokens!(orig), None)?;
  }
  RequirePackage!("amsthm");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // Perl beamer.cls.ltxml L1311: beamer always loads hyperref (real beamer
  // does too, via hyperref's kernel hooks). Without it `\url`/`\href` are
  // undefined in every beamer document that doesn't load hyperref itself —
  // the beamertheme-* TL doc corpus errored `undefined \url` en masse
  // (perfect-kernel sweep 2026-08-31). Option list matches Perl's.
  RequirePackage!("hyperref");
  // Perl beamer.cls.ltxml L1201-1239: theorem + German-compat envs.
  // `\translate{}` is an identity pass-through in Rust, so bare English
  // names match Perl's expansion.
  RawTeX!(
    r#"
\newcommand{\ExampleInline}[1]{\translate{Example}: \ignorespaces#1}
\newcommand{\BeispielInline}[1]{Beispiel: \ignorespaces#1}
\newtheorem{theorem}{Theorem}
\newtheorem{corollary}[theorem]{Corollary}
\newtheorem{fact}[theorem]{Fact}
\newtheorem{lemma}[theorem]{Lemma}
\newtheorem{problem}[theorem]{Problem}
\newtheorem{solution}[theorem]{Solution}
\newtheorem{definition}[theorem]{Definition}
\newtheorem{definitions}[theorem]{Definitions}
\newtheorem{example}[theorem]{Example}
\newtheorem{examples}[theorem]{Examples}
\newtheorem{Beispiel}[theorem]{Beispiel}
\newtheorem{Beispiele}[theorem]{Beispiele}
\newtheorem{Loesung}[theorem]{L\"osung}
\newtheorem{Satz}[theorem]{Satz}
\newtheorem{Folgerung}[theorem]{Folgerung}
\newtheorem{Fakt}[theorem]{Fakt}
\newenvironment{Beweis}{\begin{proof}[Beweis.]}{\end{proof}}
\newenvironment{Lemma}{\begin{lemma}}{\end{lemma}}
\newenvironment{Proof}{\begin{proof}}{\end{proof}}
\newenvironment{Theorem}{\begin{theorem}}{\end{theorem}}
\newenvironment{Problem}{\begin{problem}}{\end{problem}}
\newenvironment{Corollary}{\begin{corollary}}{\end{corollary}}
\newenvironment{Example}{\begin{example}}{\end{example}}
\newenvironment{Examples}{\begin{examples}}{\end{examples}}
\newenvironment{Definition}{\begin{definition}}{\end{definition}}
"#
  );
  def_macro_noop("\\pushQED{}")?;
  def_macro_noop("\\popQED")?;
  def_macro_noop("\\qedhere")?;

  // beamer overlay-aware definition forms (beamerbasemodes.sty):
  // `\newcommand<>{\cmd}[n]{body}` defines \cmd taking an optional
  // <overlay> whose spec body refers to as #(n+1). Our simplified overlay
  // policy (always-true branch) maps the overlay to EMPTY: \cmd =
  // OptionalAngled + n args; body's declared #k shift to #(k+1), #(n+1)
  // refs are dropped. Witness cursolatex L50
  // (`\newcommand<>{\aalert}[1]{\begin{alertenv}#2…#1…}` — 31
  // misdefined:# when the un-supported `<>` broke the definition).
  DefPrimitive!("\\lx@beamer@defcmd@angle {}[][]{}", sub[(cmd_tks, nargs_opt, default_opt, body)] {
    let cmd = cmd_tks.unlist().into_iter().find(|t| t.get_catcode() == Catcode::CS);
    let Some(cmd) = cmd else { return Ok(Vec::new()); };
    let n: usize = nargs_opt
      .map(|t: Tokens| t.to_string().trim().parse().unwrap_or(0))
      .unwrap_or(0);
    let has_default = default_opt.is_some();
    let packed_body = body.pack_parameters()?;
    // Remap body ARG indices: declared #k -> #(k+1); overlay #(n+1) -> #1.
    let mut newbody = Vec::new();
    for t in packed_body.unlist() {
      if t.get_catcode() == Catcode::ARG {
        let idx: usize = with(t.get_sym(), |s| s.parse().unwrap_or(0));
        let remapped = if idx == n + 1 { 1 } else { idx + 1 };
        newbody.push(Token {
          text: pin(remapped.to_string()),
          code: Catcode::ARG,
          #[cfg(feature = "token-locators")]
          loc: 0,
        });
      } else {
        newbody.push(t);
      }
    }
    let mut proto = String::from("OptionalAngled");
    let plain = n.saturating_sub(usize::from(has_default));
    if let Some(default) = default_opt {
      proto.push_str(&format!(" [Default:{}]", default.untex()));
    }
    for _ in 0..plain {
      proto.push_str(" {}");
    }
    let params = parse_parameters(&proto, &cmd, true)?;
    def_macro(cmd, params, ExpansionBody::Tokens(Tokens::new(newbody)), None)?;
    Ok(Vec::new())
  });
  RawTeX!(
    r"\let\lx@beamer@plainnewcommand\newcommand
\def\newcommand{\@ifnextchar<{\lx@beamer@newcommand@o}{\lx@beamer@plainnewcommand}}
\def\lx@beamer@newcommand@o<>{\lx@beamer@defcmd@angle}
\let\lx@beamer@plainrenewcommand\renewcommand
\def\renewcommand{\@ifnextchar<{\lx@beamer@renewcommand@o}{\lx@beamer@plainrenewcommand}}
\def\lx@beamer@renewcommand@o<>{\lx@beamer@defcmd@angle}
\let\lx@beamer@plainnewenvironment\newenvironment
\def\newenvironment{\@ifnextchar<{\lx@beamer@newenv@o}{\lx@beamer@plainnewenvironment}}
\def\lx@beamer@newenv@o<>{\lx@beamer@defenv@angle}
\let\lx@beamer@plainrenewenvironment\renewenvironment
\def\renewenvironment{\@ifnextchar<{\lx@beamer@renewenv@o}{\lx@beamer@plainrenewenvironment}}
\def\lx@beamer@renewenv@o<>{\lx@beamer@defenv@angle}"
  );
  // `\newenvironment<>{name}[n][default]{begin}{end}` — same overlay policy
  // for the environment flavor (cursolatex L81
  // `\newenvironment<>{LaTeXoutput}[1][]{\begin{actionenv}#2…}`).
  DefPrimitive!("\\lx@beamer@defenv@angle {}[][]{}{}", sub[(name_tks, nargs_opt, default_opt, beg, end)] {
    let name = name_tks.to_string().trim().to_string();
    let n: usize = nargs_opt
      .map(|t: Tokens| t.to_string().trim().parse().unwrap_or(0))
      .unwrap_or(0);
    let has_default = default_opt.is_some();
    let packed_beg = beg.pack_parameters()?;
    let mut newbeg = Vec::new();
    for t in packed_beg.unlist() {
      if t.get_catcode() == Catcode::ARG {
        let idx: usize = with(t.get_sym(), |s| s.parse().unwrap_or(0));
        let remapped = if idx == n + 1 { 1 } else { idx + 1 };
        // Same `<#1>` bracketing as `\lx@beamer@defcmd@angle` above.
        if idx == n + 1 {
          newbeg.push(T_OTHER!("<"));
        }
        newbeg.push(Token {
          text: pin(remapped.to_string()),
          code: Catcode::ARG,
          #[cfg(feature = "token-locators")]
          loc: 0,
        });
        if idx == n + 1 {
          newbeg.push(T_OTHER!(">"));
        }
      } else {
        newbeg.push(t);
      }
    }
    let mut proto = String::from("OptionalAngled");
    let plain = n.saturating_sub(usize::from(has_default));
    if let Some(default) = default_opt {
      proto.push_str(&format!(" [Default:{}]", default.untex()));
    }
    for _ in 0..plain {
      proto.push_str(" {}");
    }
    let cs = T_CS!(s!("\\{name}"));
    let params = parse_parameters(&proto, &cs, true)?;
    def_macro(cs, params, ExpansionBody::Tokens(Tokens::new(newbeg)), None)?;
    def_macro(T_CS!(s!("\\end{name}")), None, ExpansionBody::Tokens(end), None)?;
    Ok(Vec::new())
  });

  // Mode commands — Perl L448-476. The old `\mode<>{}` prototype hit the
  // def_parser literal-char fallback whose `Token` reader eats one ARBITRARY
  // token per literal: `\mode<presentation>{X}` consumed exactly `<pr` and
  // leaked `esentation>{X}` into the document (beamerswitch witness; agent
  // probe `\meaning\mode` = `macro:#1#2#3->`). Faithful shape: star form
  // no-op (Perl L455), angle-spec + brace body = run body when the spec
  // matches the presentation mode (Perl matchesCurrentMode), angle-spec
  // alone = switchmode (mode-blind noop here — the till-next-\mode gobble
  // needs the processline machinery; specs that would DISABLE text in
  // presentation mode are the rare case).
  RawTeX!(
    r"\def\mode{\@ifstar\lx@beamer@modeoutsideframe\lx@beamer@mode@}
\def\lx@beamer@modeoutsideframe{}
\def\lx@beamer@mode@<#1>{\@ifnextchar\bgroup{\lx@beamer@modeinline<#1>}{\lx@beamer@switchmode<#1>}}
\long\def\lx@beamer@modeinline<#1>#2{\lx@beamer@ifpresmode{#1}{#2}{}}
\def\lx@beamer@switchmode<#1>{}"
  );
  // {spec}{yes}{no}: yes when the spec names the presentation-family mode
  // (presentation / beamer / all / second) — beamer_cls IS the presentation
  // context, mirroring Perl matchesCurrentMode(getCurrentMode()).
  DefMacro!("\\lx@beamer@ifpresmode{}{}{}", sub[(spec, yes, no)] {
    let spec_str = spec.to_string().to_lowercase();
    let matches = spec_str.contains("presentation")
      || spec_str.contains("beamer")
      || spec_str.contains("all")
      || spec_str.trim().is_empty();
    Ok(if matches { yes } else { no })
  });
  // Perl L493-495: \presentation / \article / \common route to
  // \mode<…>. Since the Rust \mode dispatcher is already a no-op for
  // all overlay modes, the three become empty stubs. Including them
  // keeps preamble-level `\mode<all>` equivalents (from example
  // beamer style files) from throwing undefined-CS errors.
  def_macro_noop("\\presentation")?;
  def_macro_noop("\\common")?;
  // `\article` would clash with LaTeX `\article` docclass naming in
  // principle, but LaTeXML's catcode + class-file routing keeps the
  // control sequence distinct from the class name string. Perl ships
  // this alias unconditionally.
  def_macro_noop("\\article")?;

  // Perl L414-416: beamer TODO CSes (expand to warnings under Perl;
  // Rust matches by absorbing args and emitting nothing — same
  // behaviour for slide-order rendering without the beamerTODO warning.
  def_macro_noop("\\jobnamebeamerversion{}")?;
  def_macro_noop("\\includeslide{}")?;
  def_macro_noop("\\setjobnamebeamerversion")?;

  // Misc commands
  // Perl beamer.cls.ltxml L810-813 wraps \alert in \alertenv which threads
  // through \beamer@alerted@begin/end (inline-block markers defined
  // above). Routing through those requires BeamerAngled overlay parsing
  // (unported), so keep the \textbf fallback — the markers remain defined
  // and usable directly by styles that invoke them without angle-spec.
  DefMacro!("\\alert OptionalAngled {}", "\\textbf{#2}");
  DefMacro!("\\structure OptionalAngled {}", "#2");
  DefMacro!("\\emph OptionalAngled {}", "\\textit{#2}");
  def_macro_noop("\\AtBeginSection[]{}")?;
  def_macro_noop("\\AtBeginSubsection[]{}")?;
  def_macro_noop("\\AtBeginSubsubsection[]{}")?;
  def_macro_noop("\\AtBeginPart[]{}")?;
  // \subtitle<overlay>[short]{subtitle} — document-level frontmatter
  // (beamer beamerbasetitle.sty). Perl's binding never defined it (only the
  // per-frame `\framesubtitle`), so every beamer THEME demo erred
  // `undefined \subtitle` (8 TL doc bundles, 2026-08-31 corpus). Standard
  // `\lx@add@subtitle` idiom → real <ltx:subtitle> frontmatter.
  DefMacro!("\\subtitle OptionalAngled []{}", "\\lx@add@subtitle{#3}");
  // beamerbasesection.sty:45-93, the lecture layer: `\lecture<ov>[short]
  // {title}{label}` (:48-66), the `lecture` counter (:45-47),
  // `\AtBeginLecture{code}` CAPTURES its body into `\beamer@atbeginlecture`
  // (:71-73; beamerthemeVerona.sty:354-362 registers a whole `[plain]` frame
  // that must not run as document content), `\insertlecture`/
  // `\insertshortlecture` (:83-88). The title stays frontmatter (an
  // `ltx:note[role=lecture]`); the at-begin hook is not run (a structuring
  // decision beyond this layer). Perl beamer.cls.ltxml:847 "TODO: Support me!".
  RawTeX!(
    r"\newcounter{lecture}
\renewcommand\thelecture{\@arabic\c@lecture}
\let\insertlecturenumber\thelecture
\newcommand\AtBeginLecture[1]{\def\beamer@atbeginlecture{#1}}
\AtBeginLecture{}
\def\beamer@lecturename{}\def\beamer@lectureshortname{}
\newcommand*\insertlecture{\beamer@lecturename}
\newcommand*\insertshortlecture{\beamer@lectureshortname}"
  );
  DefMacro!(
    "\\lecture OptionalAngled []{}{}",
    "\\stepcounter{lecture}\\def\\beamer@lecturename{#3}\\def\\beamer@lectureshortname{#2}\\@add@frontmatter{ltx:note}[role=lecture]{#3}"
  );
  def_macro_noop("\\againframe OptionalAngled []{}")?;
  def_macro_noop("\\appendix")?;
  def_macro_noop("\\note OptionalAngled []{}")?;
  def_macro_noop("\\beamerdefaultoverlayspecification{}")?;

  // Translation stubs
  def_macro_identity("\\translate{}")?;

  // Color-related. `\usebeamercolor*[fg|bg]{name}` (re)computes the palette
  // entry, registers `<name>.fg`/`.bg`, defines the local colors `fg`/`bg`
  // templates reference, and with an optional applies that color.
  DefPrimitive!("\\usebeamercolor OptionalMatch:* []{}", sub[(_star, opt, name)] {
    let name = do_expand(name)?.to_string().trim().to_string();
    beamer_register_color(&name)?;
    for field in ["fg", "bg"] {
      if let Some(expr) = beamer_resolve(&name, field, 0)
        && beamer_expr_defined(&expr)
      {
        digest(Tokenize!(TeXString::assembled(format!("\\colorlet{{{field}}}{{{expr}}}"))))?;
      }
    }
    if let Some(which) = opt {
      let which = do_expand(which)?.to_string();
      let which = which.trim();
      if (which == "fg" || which == "bg")
        && let Some(expr) = beamer_resolve(&name, which, 0)
        && beamer_expr_defined(&expr)
      {
        digest(Tokenize!(TeXString::assembled(format!("\\color{{{name}.{which}}}"))))?;
      }
    }
  });
  // `\ifbeamercolorempty[fg|bg]{name}{true}{false}` — themes branch on it.
  DefMacro!("\\ifbeamercolorempty[]{}", sub[(opt, name)] {
    let name = do_expand(name)?.to_string().trim().to_string();
    let field = opt.map(|o| o.to_string()).unwrap_or_else(|| "fg".to_string());
    let empty = beamer_resolve(&name, field.trim(), 0).map(|e| e.is_empty()).unwrap_or(true);
    if empty { Tokens!(T_CS!("\\@firstoftwo")) } else { Tokens!(T_CS!("\\@secondoftwo")) }
  });

  // Hyperlink
  DefMacro!("\\hyperlink{}{}", "#2");
  DefMacro!("\\hypertarget{}{}", "#2");

  // The default palette LAST (needs xcolor's \definecolor and our \mode /
  // \setbeamercolor above): real beamer's beamercolorthemedefault.sty is
  // plain `\setbeamercolor` calls — raw-load it through our implementation
  // so `normal text.fg`, `alerted text.fg`, `structure.fg`, … exist exactly
  // as beamer defines them (beamer.cls defines beamer@blendedblue first).
  digest(Tokenize!(TeXString::assembled(
    "\\definecolor{beamer@blendedblue}{rgb}{0.2,0.2,0.7}".to_string()
  )))?;
  InputDefinitions!("beamercolorthemedefault", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // beamerbasecolor.sty L387-388: plain `structure`/`beamerstructure` are
  // xcolor aliases of `structure.fg` (themes write `\textcolor{structure}`).
  digest(Tokenize!(TeXString::assembled(
    "\\colorlet{structure}{structure.fg}\\colorlet{beamerstructure}{structure.fg}".to_string()
  )))?;
  // Baseline current-color pair: templates reference `fg`/`bg` (and
  // `parent.fg`/`parent.bg`) outside any \usebeamercolor context; beamer
  // always has SOME current pair (normal text's black-on-white default).
  digest(Tokenize!(TeXString::assembled(
    "\\colorlet{fg}{black}\\colorlet{bg}{white}\\colorlet{parent.fg}{black}\\colorlet{parent.bg}{white}"
      .to_string()
  )))?;
  // beamerbasenavigation.sty:340-351 — the TOC/navigation display-style
  // setters the sidebar OUTER theme calls at load (beamerouterthemesidebar
  // .sty:33 `\beamer@nav@subsectionstyle{show/shaded/hide}`); pure
  // presentation, accepted and ignored (LaTeX-Course, sweep-37 regression
  // once 55c routed theme loading for real).
  RawTeX!(
    r"\providecommand\beamer@nav@sectionstyle[1]{}\providecommand\beamer@nav@subsectionstyle[1]{}\providecommand\beamer@nav@subsubsectionstyle[1]{}"
  );
});
