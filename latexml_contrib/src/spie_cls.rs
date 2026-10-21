//! Stub for spie.cls (SPIE conference proceedings).
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

/// Does the class code `text` define `\author` itself (`\def\author`, `\renewcommand{\author}`, …; not `\authorinfo`)?
fn defines_author(text: &str) -> bool {
  let code = crate::shipped_class::class_code(text);
  [
    "\\def\\author",
    "\\gdef\\author",
    "\\let\\author",
    "\\renewcommand\\author",
    "\\renewcommand{\\author}",
    "\\renewcommand*{\\author}",
    "\\newcommand{\\author}",
    "\\DeclareRobustCommand{\\author}",
  ]
  .iter()
  .any(|form| {
    code.match_indices(form).any(|(at, _)| {
      code[at + form.len()..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic() && c != '@')
    })
  })
}

LoadDefinitions!({
  // spie.cls defines no `\author` — it only `\LoadClassWithOptions{article}` (:74) —, so its documented author block
  // (:58-64: names with `\supit` marks, `\skiplinehalf`, then `\\`-separated affiliation lines) is the kernel's `\author`,
  // whose splitter reads the names and the marked affiliation lines; OmniBus's `\author` (inst_support) splits only at
  // `\and`, and read every affiliation line as more names (2210.16202's CHRONOS_s.cls, `\ProvidesClass{spie}`: 39
  // creators, no contact, where the PDF prints 5 authors over 10 affiliation lines). A class that defines its own
  // `\author` (a spieman.cls copy's `\author[a]`/`\affil[a]`) keeps OmniBus's. Guard
  // `perfect_kernel_batch77w::spie_author_block_is_the_kernel_author`.
  //
  // A copy that loads authblk — SPIE's 2015 v3.4 spie.cls, `\RequirePackage{authblk}` (its :31; the TeX Live v3.25
  // copy loads none) — writes its author block with authblk's `\author[a]{…}`/`\affil[a]{…}`: it loads authblk as the
  // copy does, whose `\author` reads the marks (the kernel's one-argument `\author` took "[" as the name and left
  // every name in the body, 2406.08304: one creator "[", 30 empty affiliations). Guard
  // `perfect_kernel_batch78m::spie_copy_loading_authblk_reads_authblk_authors`.
  let copy = crate::shipped_class::shipped_class_text("spie")?;
  let loads_authblk = copy
    .as_deref()
    .is_some_and(|text| crate::shipped_class::class_text_loads(text, "authblk"));
  let kernel_author = copy.is_some_and(|text| !defines_author(&text)) && !loads_authblk;
  if kernel_author {
    Let!("\\lx@spie@kernel@author", "\\author");
  }
  LoadClass!("OmniBus");
  if loads_authblk {
    RequirePackage!("authblk");
  }
  if kernel_author {
    // (the kernel's splitter reads the argument as written, where TeX expands the class's author-block macros when
    // `\maketitle` prints it: spie.cls:123-124 `\skiplinehalf` (`\medskip\\`) and `\skipline` (`\\`) end a line,
    // :128 `\supit` raises the mark "to cross reference author and affiliations" — the splitter's line break and
    // superscript mark)
    DefMacro!("\\author{}", sub[(names)] {
      let mut out = vec![T_CS!("\\lx@spie@kernel@author"), T_BEGIN!()];
      out.extend(names.unlist_ref().iter().map(|t| {
        if *t == T_CS!("\\skiplinehalf") || *t == T_CS!("\\skipline") {
          T_CS!("\\\\")
        } else if *t == T_CS!("\\supit") {
          T_CS!("\\textsuperscript")
        } else {
          *t
        }
      }));
      out.push(T_END!());
      Ok(Tokens::new(out))
    });
  }
  // spie.cls (TeX Live's) and spieman.cls (39 copies in a 1/20 arXiv sample) load no amsmath: amsmath's commands for
  // papers that use them unloaded, its "Old form" error only once the paper loads it (1003.3524's `\pmatrix{…}`, no
  // error in pdflatex; `amsmath_sty::require_amsmath_beyond_original`, KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
  RequirePackage!("amsthm");
  // Pre-load xcolor with [dvipsnames, table] so a paper's later
  // `\usepackage[table]{xcolor}` doesn't silently option-clash and
  // leave colortbl unloaded → `\cellcolor` undefined. spie.cls itself
  // does NOT load xcolor (only `\LoadClassWithOptions{article}`), so
  // in Perl the user's `[table]{xcolor}` is the first load and colortbl
  // comes in via the `table` option; matching that outcome here. Same
  // anti-clash pattern as svproc_cls / mnras_cls / quantumarticle_cls.
  // Witness: 1807.04749 ("undefined:\\cellcolor").
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("hyperref");
  // spie.cls L92: `\RequirePackage[superscript]{cite}[2003/11/04]` — SPIE uses
  // the cite package for superscript numeric citations. cite.sty defines the
  // no-brackets `\citen`/`\citenum`/`\citeonline` aliases SPIE templates use
  // directly. Our OmniBus base autoloads natbib's `\citenum` but NOT cite's
  // `\citen`, so without this `\citen` is undefined where Perl (raw-loads
  // spie.cls → cite) is clean. Faithful (spie.cls genuinely loads cite — not an
  // eager preload). Witness 1808.10428 (`{spie}` + `\citen`).
  RequirePackage!("cite", options => vec!["superscript".to_string()]);

  // spie.cls L107: \authorinfo{...} for author footnote — preserve.
  DefMacro!(
    "\\authorinfo{}",
    "\\@add@frontmatter{ltx:note}[role=authorinfo]{#1}"
  );
  // spie.cls:123 `\def\skiplinehalf{\medskip\\}`: in the kernel's author block it ends the names line (the
  // `\medskip` is the page's spacing); under OmniBus's `\author` it prints nothing
  if kernel_author {
    DefMacro!("\\skiplinehalf", "\\\\");
  } else {
    def_macro_noop("\\skiplinehalf")?;
  }
  DefMacro!("\\supit{}", "\\textsuperscript{#1}");
  // spieman.cls, SPIE's journal manuscript class, which the loader gives this binding ("Can't find binding for class
  // spieman (using spie)"; 87 of 99 sampled copies are its v1.10, 1703.03292's): `\linkable` (:82), an address or URL
  // set blue and underlined but not linked (run 340: 640 first errors; 2604.17554, 2604.05004); the 2006 v1.00 copies'
  // `\supscr`/`\supscrsm`, raised small text (1408.0407). spie.cls defines neither, so they are guesses a paper's own
  // definition replaces (`DefinitionOrigin::Fallback`).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(
      r"\newcommand{\linkable}[1]{\textcolor{blue}{\underline{#1}}}
\def\supscr#1{\raisebox{0.8ex}{\small #1}\hspace{0.05em}}
\def\supscrsm#1{\raisebox{0.8ex}{\footnotesize #1}\hspace{0.05em}}"
    );
  }
});
