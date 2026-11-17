//! lni.cls (GI Lecture Notes in Informatics): the class is loaded raw. From v2.0 (TL 2025 ships 2025/02/05 v2.0) it
//! builds its author block by copying authblk's code (lni.cls:362 `\RequirePackage{authblk}`, :384-435
//! `\renewcommand\author[4][]`, :444-464 `\renewcommand\affil[2][]`) into accumulators only its `\maketitle` reads; that
//! block becomes the frontmatter as authblk's binding makes it: `\author[marks]{name}{e-mail}{ORCID}` an author carrying
//! its affiliation marks, its e-mail and ORCID as contacts (the class prints them in the affiliation footnote,
//! :425-432), `\affil[mark]{…}` authblk's. Under the raw class the copy's `\affil` replaced the binding's and the
//! kernel lock refused its `\author`, so the affiliations, e-mails and ORCIDs were lost and the last two arguments were
//! left as stray preamble text (2608.21389). A paper's own older copy (v0.5, v1.7) loads no authblk and keeps LaTeX's
//! `\author[]{}` (`\renewcommand{\author}{\@dblarg\@@author}`), which the kernel's reads (2203.09493, 1705.02823,
//! 2210.04484).
use latexml_package::{package::authblk_sty::define_author_and_affil, prelude::*};

LoadDefinitions!({
  // (the kernel's `\author`, before the class's authblk replaces it: a call written for lni v1 is its, below)
  Let!("\\lx@lni@kernel@author", "\\author");
  InputDefinitions!("lni", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // (the title's marks lni's own `\maketitle` prints, every version's: lni.cls:475-477 `\renewcommand\thefootnote
  // {\@arabic\c@footnote}`, v0.5's :89-93; 2407.04435 "1, 1, 1,2, 3", 1705.02823, 1705.08889)
  if lookup_bool("\\maketitle:redefined@class") {
    DefMacro!("\\lx@title@thefootnote", "\\@arabic\\c@footnote");
    // (its `\thanks` LaTeX's, each stepping the counter; set back after the title only by a `\maketitle` that says so,
    // v0.5's, article's copy (:116), not v1.7's or v2.0's (lni.cls:478-513): a body note after a v2.0 title's
    // `\thanks` 3 is 4)
    latexml_package::engine::latex_constructs::declare_title_footnote_counting(
      maketitle_resets_footnotes(),
    )?;
  }
  // (the class read raw by this binding is a raw class to K11, its title-page stores rerouted at the end of the class
  // load: lni.cls `\subtitle`; the binding's `\author`/`\affil` below stay its own)
  if package_loaded("authblk") {
    // (a call written for lni v1, `\author[running names]{names}` with no e-mail and ORCID arguments after it, is read
    // as v1's `\@dblarg\@@author` reads it, by the kernel's `\author[short]{names}`: v2.0's `[4][]` would take the next
    // tokens for them and the running names for authblk labels; pdflatex stops there, 2407.04435)
    define_author_and_affil()?;
    Let!("\\lx@lni@author", "\\author");
    // (and lni's `\affil` sets the footnote counter to its label, so a title `\thanks` is numbered after the last one:
    // lni.cls:463, v1.10's :468, 2412.04178)
    Let!("\\lx@lni@affil", "\\affil");
    DefMacro!("\\affil OptionalSemiverbatim {}", r"\lx@lni@affil[#1]{#2}\setcounter{footnote}{#1}", robust => true);
    DefMacro!(
      "\\author[]{}",
      r"\@ifnextchar\bgroup{\lx@lni@author@v{#1}{#2}}{\lx@lni@kernel@author[#1]{#2}}"
    );
    DefMacro!(
      "\\lx@lni@author@v{}{}{}{}",
      r"\if\relax\detokenize{#1}\relax\lx@lni@author{#2}\else\lx@lni@author[#1]{#2}\fi\if\relax\detokenize{#3}\relax\else\lx@add@email{#3}\fi\if\relax\detokenize{#4}\relax\else\lx@add@orcid{#4}\fi"
    );
  }
});

/// Whether the class's own `\maketitle`, which the lock refused (`\lx@dropped@maketitle`), sets the footnote counter back
/// to 0 (`\setcounter{footnote}{0}`).
fn maketitle_resets_footnotes() -> bool {
  lookup_definition(&T_CS!("\\lx@dropped@maketitle"))
    .ok()
    .flatten()
    .is_some_and(|definition| match definition.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => body
        .to_string()
        .replace(' ', "")
        .contains("\\setcounter{footnote}{0}"),
      _ => false,
    })
}
