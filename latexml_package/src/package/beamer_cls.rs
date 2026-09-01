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

#[rustfmt::skip]
LoadDefinitions!({
  // Load article.cls as the base class (beamer builds on article).
  // Don't load raw beamer.cls — its expansion chains exceed the token limit.
  RequirePackage!("article");
  // Perl beamer.cls.ltxml L30-32: "these packages probably aren't needed,
  // but let's load them anyways!" — graphicx especially IS needed: real
  // beamer's dependency chain provides \includegraphics, and theme demos
  // use it bare (sweep-11 cluster: 9 docs `undefined:\includegraphics`,
  // witness beamertheme-focus/focus-demo).
  RequirePackage!("ifpdf");
  RequirePackage!("keyval");
  RequirePackage!("graphicx");
  // Real beamer requires pgfcore (beamer.cls → beamerbasemodes → pgfcore);
  // themes then use shadings/pictures directly (epyt's
  // \pgfdeclareverticalshading, gotham). Load our pgf binding so that raw
  // theme surface resolves against the real implementations.
  RequirePackage!("pgf");

  // Perl beamer.cls.ltxml L853: DefKeyVal('beamerframe', 'fragile', '', '')
  // — declares `fragile` as a zero-argument key for the beamerframe keyset.
  // Documents using `\begin{frame}[fragile]{Title}` rely on this to parse
  // without "unknown keyval" errors. Frame env's Rust stub doesn't
  // consult keyvals yet, but the declaration itself must load.
  DefKeyVal!("beamerframe", "fragile", "");

  // Frame environment — the core beamer construct.
  // Absorbs optional overlay spec and optional title/subtitle args.
  // Perl: DefEnvironment('{frame}[][]', '<ltx:slide...>...</ltx:slide>');
  DefEnvironment!("{frame}[][]",
    "<ltx:subsection _noautoclose='1'>#body</ltx:subsection>");

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
  DefMacro!("\\only OptionalAngled {} OptionalAngled", "#2");
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

  DefMacro!("\\beamer@visible{}",   "\\beamer@visible@begin{#1}\\beamer@visible@end");
  DefConstructor!("\\beamer@visible@begin", "<ltx:inline-block class='ltx_visible'>");
  DefConstructor!("\\beamer@visible@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@invisible{}", "\\beamer@invisible@begin{#1}\\beamer@invisible@end");
  DefConstructor!("\\beamer@invisible@begin", "<ltx:inline-block class='ltx_invisible'>");
  DefConstructor!("\\beamer@invisible@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@uncovered{}", "\\beamer@uncovered@begin{#1}\\beamer@uncovered@end");
  DefConstructor!("\\beamer@uncovered@begin", "<ltx:inline-block class='ltx_uncovered'>");
  DefConstructor!("\\beamer@uncovered@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@covered{}", "\\beamer@covered@begin{#1}\\beamer@covered@end");
  DefConstructor!("\\beamer@covered@begin", "<ltx:inline-block class='ltx_covered'>");
  DefConstructor!("\\beamer@covered@end",   "</ltx:inline-block>");

  DefMacro!("\\beamer@alerted{}", "\\beamer@alerted@begin{#1}\\beamer@alerted@end");
  DefConstructor!("\\beamer@alerted@begin", "<ltx:inline-block class='ltx_alert'>");
  DefConstructor!("\\beamer@alerted@end",   "</ltx:inline-block>");

  // Frame structure
  DefMacro!("\\frametitle OptionalAngled []{}",
    "\\par\\textbf{#3}\\par");
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
  DefConstructor!("\\beamer@frametitle{}",
    "^<ltx:title class='ltx_frame_title'>#1</ltx:title>");
  DefConstructor!("\\beamer@frameshorttitle{}",
    "^<ltx:title class='ltx_frame_shorttitle'>#1</ltx:title>");
  DefConstructor!("\\beamer@framesubtitle{}",
    "^<ltx:subtitle class='ltx_frame_subtitle'>#1</ltx:subtitle>");

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
  DefEnvironment!("{block} OptionalAngled {}",
    "<ltx:theorem class='ltx_theorem_block'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>");
  DefEnvironment!("{alertblock} OptionalAngled {}",
    "<ltx:theorem class='ltx_theorem_alertblock'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>");
  DefEnvironment!("{exampleblock} OptionalAngled {}",
    "<ltx:theorem class='ltx_theorem_exampleblock'><ltx:title class='ltx_runin'>#2</ltx:title>#body</ltx:theorem>");

  // Columns environment — Perl L1230-1240 beamerbaseboxes.sty
  DefEnvironment!("{columns} OptionalAngled []", "#body");
  DefEnvironment!("{column} OptionalAngled {}", "#body");
  def_macro_noop("\\column OptionalAngled {}")?;

  // Title page macros — Perl L1010-1035
  DefMacro!("\\institute OptionalAngled []{}", "\\@add@frontmatter{ltx:creator}{\\@@@affiliation{#3}}");
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
  DefMacro!("\\titlegraphic{}",
    "\\@add@frontmatter{ltx:note}[role=titlegraphic]{#1}");
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
  DefPrimitive!("\\usetheme[]{}", sub[(_opts, names)] {
    for name in do_expand(names)?.to_string().split(',') {
      let name = name.trim();
      if !name.is_empty() {
        let _ = require_package(
          &s!("beamertheme{name}"),
          RequireOptions::default(),
        );
      }
    }
  });
  DefPrimitive!("\\usecolortheme[]{}", sub[(_opts, names)] {
    for name in do_expand(names)?.to_string().split(',') {
      let name = name.trim();
      if !name.is_empty() {
        let _ = require_package(
          &s!("beamercolortheme{name}"),
          RequireOptions::default(),
        );
      }
    }
  });
  def_macro_noop("\\usefonttheme[]{}")?;
  DefPrimitive!("\\useinnertheme[]{}", sub[(_opts, names)] {
    for name in do_expand(names)?.to_string().split(',') {
      let name = name.trim();
      if !name.is_empty() {
        let _ = require_package(
          &s!("beamerinnertheme{name}"),
          RequireOptions::default(),
        );
      }
    }
  });
  DefPrimitive!("\\useoutertheme[]{}", sub[(_opts, names)] {
    for name in do_expand(names)?.to_string().split(',') {
      let name = name.trim();
      if !name.is_empty() {
        let _ = require_package(
          &s!("beameroutertheme{name}"),
          RequireOptions::default(),
        );
      }
    }
  });
  // Theme-file surface the raw loads touch.
  def_macro_noop("\\ProcessOptionsBeamer")?;
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
  def_macro_noop("\\addtobeamertemplate{}{}{}")?;
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
  RequirePackage!("xcolor");
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
  RawTeX!(r#"
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
"#);
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
    if default_opt.is_some() {
      Info!("unexpected", "beamer",
        "\\newcommand<> with an optional-default arg: overlay dropped, default unsupported");
    }
    // Remap body ARG indices: declared #k -> #(k+1); overlay #(n+1) -> drop.
    let mut newbody = Vec::new();
    for t in body.unlist() {
      if t.get_catcode() == Catcode::ARG {
        let idx: usize = with(t.get_sym(), |s| s.parse().unwrap_or(0));
        if idx == n + 1 {
          continue; // overlay ref -> empty
        }
        newbody.push(Token {
          text: pin((idx + 1).to_string()),
          code: Catcode::ARG,
          #[cfg(feature = "token-locators")]
          loc: 0,
        });
      } else {
        newbody.push(t);
      }
    }
    let proto = s!("OptionalAngled{}", " {}".repeat(n));
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
    let mut newbeg = Vec::new();
    for t in beg.unlist() {
      if t.get_catcode() == Catcode::ARG {
        let idx: usize = with(t.get_sym(), |s| s.parse().unwrap_or(0));
        if idx == n + 1 {
          continue;
        }
        newbeg.push(Token {
          text: pin((idx + 1).to_string()),
          code: Catcode::ARG,
          #[cfg(feature = "token-locators")]
          loc: 0,
        });
      } else {
        newbeg.push(t);
      }
    }
    let mut proto = String::from("OptionalAngled");
    let plain = n.saturating_sub(usize::from(has_default));
    if has_default {
      proto.push_str(" []");
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
  def_macro_noop("\\AtBeginPart[]{}")?;
  // \subtitle<overlay>[short]{subtitle} — document-level frontmatter
  // (beamer beamerbasetitle.sty). Perl's binding never defined it (only the
  // per-frame `\framesubtitle`), so every beamer THEME demo erred
  // `undefined \subtitle` (8 TL doc bundles, 2026-08-31 corpus). Standard
  // `\lx@add@subtitle` idiom → real <ltx:subtitle> frontmatter.
  DefMacro!("\\subtitle OptionalAngled []{}", "\\lx@add@subtitle{#3}");
  // \lecture{title}{shortname} — beamer lecture frontmatter; preserve
  // the title text as ltx:note frontmatter rather than dropping it.
  DefMacro!("\\lecture{}{}",
    "\\@add@frontmatter{ltx:note}[role=lecture]{#1}");
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
});
