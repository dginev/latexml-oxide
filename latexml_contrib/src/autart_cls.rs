//! Stub for Elsevier's autart.cls (Automatica journal).
//!
//! autart.cls is an article-derivative. The native binding fallback to
//! OmniBus misses class-defined helpers like {ack} environment and
//! several elsart-style frontmatter macros.
use latexml_package::{package::elsart_support_core_sty, prelude::*};

use crate::shipped_class::{class_option_given, shipped_class_text};

/// autart.cls:551-580 (1112.0879's copy): the theorem environments after `{thm}`, as the class declares them.
const AUTART_THEOREMS: &[&str] = &[
  r"\newtheorem{cor}[thm]{Corollary}",
  r"\newtheorem{lem}[thm]{Lemma}",
  r"\newtheorem{claim}[thm]{Claim}",
  r"\newtheorem{axiom}[thm]{Axiom}",
  r"\newtheorem{conj}[thm]{Conjecture}",
  r"\newtheorem{fact}[thm]{Fact}",
  r"\newtheorem{hypo}[thm]{Hypothesis}",
  r"\newtheorem{assum}[thm]{Assumption}",
  r"\newtheorem{prop}[thm]{Proposition}",
  r"\newtheorem{crit}[thm]{Criterion}",
  r"\newtheorem{defn}[thm]{Definition}",
  r"\newtheorem{exmp}[thm]{Example}",
  r"\newtheorem{rem}[thm]{Remark}",
  r"\newtheorem{prob}[thm]{Problem}",
  r"\newtheorem{prin}[thm]{Principle}",
  r"\newtheorem{alg}{Algorithm}",
  r"\newtheorem{note}{Note}",
  r"\newtheorem{summ}{Summary}",
  r"\newtheorem{case}{Case}",
];

/// The `\newtheorem{name}[counter]{Title}` / `\newtheorem{name}{Title}[within]` declarations of a copy of autart.cls,
/// outside comments and other than `{thm}`'s (whose numbering the `secthm` option chooses), in the copy's order: each
/// environment's name and its declaration.
fn class_theorem_declarations(text: &str) -> Vec<(String, String)> {
  let re =
    Regex::new(r"\\newtheorem\{([A-Za-z]+)\}(\[[A-Za-z]+\])?\{([^{}]*)\}(\[[A-Za-z]+\])?").unwrap();
  let mut out = Vec::new();
  for line in text.lines() {
    let code = line.split('%').next().unwrap_or_default();
    for cap in re.captures_iter(code) {
      let name = &cap[1];
      if name == "thm" {
        continue;
      }
      out.push((name.to_string(), cap[0].to_string()));
    }
  }
  out
}

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // Do NOT eager-load amsthm here. Perl ships no autart binding (→ OmniBus,
  // no amsthm preload), and OmniBus already installs LAZY theorem-env
  // autoload stubs (`\begin{theorem}`/`\begin{proof}`/… each `require`
  // amsthm on first use). Preloading amsthm eagerly breaks the common
  // pattern of a document that clears a class-defined `\proof` and then
  // (re)loads amsthm to get amsthm's version:
  //     \let\proof\relax        % drop autart/class \proof
  //     \usepackage{amsthm}     % expect amsthm to (re)define \proof
  // With amsthm pre-loaded, the `\usepackage{amsthm}` is a no-op (already
  // loaded), so amsthm's `\let\proof\@proof` never re-runs and `\proof`
  // stays `\relax` → `\begin{proof}` → "{proof} environment not defined".
  // Witness arXiv:2009.00150 (autart + `\let\proof\relax` + amsthm). The
  // lazy OmniBus stub still covers papers that DON'T load amsthm themselves.

  // autart.cls L317-323: \def\ack{\section*{Acknowledgements}}
  // with \let\endack\par. Bind {ack}/{ack*} as structural
  // ltx:acknowledgements (post-processors map to canonical
  // role/styling).
  DefEnvironment!("{ack}",  "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  DefEnvironment!("{ack*}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");

  // autart.cls L537 `\newenvironment{pf}{…{\bfseries\Elproofname}…}{…}` with
  // `\def\Elproofname{PROOF.}`, plus `\@namedef{pf*}#1{…custom label…}`. autart's
  // own proof environment — distinct from amsthm's {proof} (handled lazily via
  // OmniBus), so the OmniBus theorem-env autoload never covers it and `{pf}` was
  // undefined (Perl raw-executes the cls). Bind as a semantic ltx:proof (mirrors
  // mn2e_support's {proof} pattern; self-contained, no amsthm dep — avoids the
  // eager-amsthm hazard noted below). Witness 2309.12476 (autart, `\begin{pf}` in
  // \input'd subfiles).
  // A constructor pair, as autart.cls:537-543 makes `{pf}` a plain environment and `{pf*}` a macro around `\pf`: the
  // body stays in the mode around it, so a `$$` there is display math (a DefEnvironment boxed it in restricted
  // horizontal mode, where the formula ran as text, "Script _ can only appear in math mode": 2101.05047, 14 of 40
  // sampled run-336 papers with `$$` and frontmatter). The end closes the `ltx:proof` only when one is open.
  DefMacro!("\\Elproofname", "PROOF.");
  DefConstructor!("\\pf", "<ltx:proof><ltx:title>#title</ltx:title>",
  properties => sub[_args] {
    // autart.cls:539 `{\bfseries\Elproofname}`.
    let title = digest(Tokens!(T_BEGIN!(), T_CS!("\\bfseries"), T_CS!("\\Elproofname"), T_END!()))?;
    Ok(stored_map!("title" => title))
  });
  DefConstructor!("\\endpf", sub[document, _args] {
    document.maybe_close_element("ltx:proof")?;
  });
  RawTeX!(
    r"\@namedef{pf*}#1{\par\begingroup\def\Elproofname{#1}\pf\endgroup\ignorespaces}
\expandafter\let\csname endpf*\endcsname=\endpf"
  );

  // autart.cls carries elsart.cls (its last line: "End of file `elsart.cls'"), and with it elsart's `{keyword}`
  // (autart.cls:1248-1258 of 1106.2518's copy): `\sep` between the keywords, `\PACS` and `\MSC` heading the
  // classifications that follow them (`\sep` 40 papers of the inventory, `\PACS` 5, `\MSC` 3: 1106.2518, 0807.2769),
  // the markup the elsart binding gives them (elsart_support_core_sty.rs `define_keyword_markup`).
  elsart_support_core_sty::define_keyword_markup()?;
  RawTeX!(r"\def\sep{\unskip, }");
  // autart.cls:1429-1456 (1112.0879's copy, every copy sampled): the class's math shorthands — `\e` the upright
  // exponential (35 papers: 1207.4104), `\half`/`\threehalf`/`\quart` the text-style fractions (11: 1112.0879), the number
  // sets `\Rset`, `\Nset`, `\Zset`, … (7, 1, 1: 1611.06692, 1705.01052, 2404.11194), `\nuc`/`\itnuc` the nuclide
  // superscripts and `\pol` the arrow vector, saved as `\old@vec`.
  RawTeX!(
    r"\def\nuc#1#2{\relax\ifmmode{}^{#1}{\protect\text{#2}}\else${}^{#1}$#2\fi}
\def\itnuc#1#2{\setbox\@tempboxa=\hbox{\scriptsize\it #1}\def\@tempa{{}^{\box\@tempboxa}\!\protect\text{\it #2}}\relax\ifmmode \@tempa \else $\@tempa$\fi}
\let\old@vec\vec\def\pol#1{\old@vec{#1}}
\def\half{{\textstyle\frac{1}{2}}}\def\threehalf{{\textstyle\frac{3}{2}}}\def\quart{{\textstyle\frac{1}{4}}}
\def\e{\mathop{\mathrm{e}}\nolimits}
\def\Cset{\mathbb{C}}\def\Hset{\mathbb{H}}\def\Nset{\mathbb{N}}\def\Qset{\mathbb{Q}}\def\Rset{\mathbb{R}}\def\Zset{\mathbb{Z}}"
  );
  // autart.cls:722-723: `\cv`, the authors' curriculum vitae text, and `{cv*}` with its heading (2204.06912).
  RawTeX!(r"\def\cv{\small}\@namedef{cv*}{\section*{Curriculum Vitae}\cv}");
  // autart.cls:545-580: the class's theorem set, defined when the class loads with the kernel's `\newtheorem` (amsthm
  // only under the `amsthm` option, :90-91, :518-519): `{thm}` numbered within sections under `secthm` (:545-550), the
  // statements numbered with it, `{alg}`, `{note}`, `{summ}`, `{case}` on their own counters (`{case}` 4 papers:
  // 2001.10125). A copy's own additions in that list are defined as it declares them (`{assumption}` 4 papers,
  // 1912.06343's copy :558 `\newtheorem{assumption}[thm]{Assumption}`; `{property}` 2, 1804.10435's :567), which a
  // fallback set defined lazily could not number with `thm`.
  let secthm = class_option_given("secthm")?;
  RawTeX!(if secthm {
    r"\@ifundefined{thm}{\newtheorem{thm}{Theorem}[section]}{}"
  } else {
    r"\@ifundefined{thm}{\newtheorem{thm}{Theorem}}{}"
  });
  let declared = shipped_class_text("autart")?
    .map(|text| class_theorem_declarations(&text))
    .unwrap_or_default();
  let declared = if declared.is_empty() {
    AUTART_THEOREMS
      .iter()
      .flat_map(|d| class_theorem_declarations(d))
      .collect()
  } else {
    declared
  };
  // Each only where the name is free, as OmniBus's fallback set (omnibus_cls.rs): its lazy `{cor}`, `{lem}`, … are
  // the class's, numbered with `{thm}`.
  for (name, declaration) in declared {
    raw_tex(&format!(r"\@ifundefined{{{name}}}{{{declaration}}}{{}}"))?;
  }

  // Common elsart frontmatter macros (autart inherits elsart style) —
  // preserve author-supplied content as ltx:note frontmatter.
  DefMacro!(
    "\\address[]{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#2}"
  );
  DefMacro!("\\thanksref{}", "\\textsuperscript{#1}");
  DefMacro!("\\corauthref{}", "\\textsuperscript{*#1}");
  DefMacro!(
    "\\corauth{}",
    "\\@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  DefMacro!(
    "\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}"
  );

  // autart.cls L516 defines `\qed` directly at class level (a `\Box` at the
  // end of a proof). Perl — which OmniBus-fallbacks autart and dep-scans
  // autart.cls's `\if@amsthm \RequirePackage{amsthm}` (the regex dep-scan
  // ignores the `\if@amsthm` guard) — ends up loading amsthm and so produces
  // amsthm's `\qed` (∎). We do NOT eager-load amsthm here (see the amsthm note
  // above re: 2009.00150), so mirror amsthm's `\qed`/`\ltx@qed` (∎) directly to
  // match Perl's ground-truth output. A paper that later `\usepackage{amsthm}`
  // simply re-installs the identical definitions. Used via the common
  // `\def\epf{\hfill\mbox{\qed}}` idiom OUTSIDE any proof env, so OmniBus's
  // lazy theorem-env autoload never fires. Witness 1703.03101. Sized by amsthm's own sizer (`openbox_size`), so
  // `\mbox{\qed}` alone in a cell is kept.
  // autart.cls:568-577 defines the class's own algorithm float: counter `algorithm`, listed in `lof`, its caption
  // "Algorithm N." (`\fnum@algorithm`, `\@makealgocaption`); `{algorithm}` was undefined, where a paper loads only
  // algpseudocode for the body (2106.14831, 2402.17000).
  float_sty::define_float_environment("algorithm", "lof", "")?;
  DefMacro!("\\lx@name@algorithm", "Algorithm");
  DefMacro!("\\fname@algorithm", "Algorithm");
  DefMacro!("\\fnum@font@algorithm", "\\bfseries");
  DefMacro!(
    "\\format@title@algorithm{}",
    "\\lx@tag[][. ]{\\lx@fnum@@{algorithm}}#1"
  );
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true,
    sizer => sub[whatsit] { Ok(amsthm_sty::openbox_size(whatsit)) },
    reversion => "\\qed"
  );
  Let!("\\mathqed", "\\qed");
  Let!("\\textsquare", "\\qed");
  Let!("\\qedsymbol", "\\qed");
  Let!("\\openbox", "\\qed");
});
