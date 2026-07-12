//! Stub for Interspeech.cls (Interspeech conference template).
//!
//! User-bundled conference template; not in TeX Live. The two macros
//! commonly tripped: \interspeechcameraready (camera-ready toggle)
//! and \interspeech (logo/title helpers). Gobble cleanly so the
//! frontmatter doesn't fail.
//! Witness 2409.08589, 2409.08711.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("graphicx");
  RequirePackage!("booktabs");

  // Interspeech frontmatter — preserve author content.
  def_macro_noop("\\interspeechcameraready")?;
  // INTERSPEECH2023.cls L160: `\def\ninept{\def\baselinestretch{0.95}
  // \let\normalsize\small\normalsize}` — 9-point text mode. Layout
  // adjustment, semantically irrelevant for our XML output. Witness
  // 2312.05730.
  def_macro_noop("\\ninept")?;
  // Modern ISCA Interspeech.cls (2023–2025, and the Interspeech20YY variants
  // that resolve here by version-suffix stripping) redefines \name with the
  // keyval signature `\newcommand{\name}[3][]` — one author per call:
  //   `\name[affiliation={1,*}]{First}{Last}`
  // splitting the name into First/Last and carrying a superscript affiliation
  // marker. (The old single-argument `\def\name#1` is commented out in the
  // shipped class.) Bind that shape to a structured "First Last" creator; a
  // single-arg `\name{}` would grab `[` and drop every author (witness
  // 2406.11727 → `[ [ [ …`). The optional `affiliation={…}` markers are numeric
  // cross-references into the `\address` block, which is preserved below, so we
  // drop the marker rather than smuggle its keyval text into the name.
  DefMacro!("\\name [] {} {}", "\\lx@add@creator[role=author]{#2 #3}");
  DefMacro!(
    "\\address{}",
    "\\@add@frontmatter{ltx:note}[role=address]{#1}"
  );
  DefMacro!("\\email{}", "\\@add@frontmatter{ltx:note}[role=email]{#1}");
  DefMacro!(
    "\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}"
  );
  DefMacro!(
    "\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\copyrightnotice{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
});
