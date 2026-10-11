//! World Scientific's proceedings styles for the article class: `sprocl.sty` (8.5 × 6 in), `mprocl.sty` (9.75 × 6.5
//! in) and `stwol.sty` (two-column, smaller size), "based on a style file by Lukas Nellen", LaTeX 2.09 option styles
//! (`\documentstyle[sprocl]{article}`). Not in TeX Live and not always shipped: missing, their `\author` was article's,
//! which keeps the last call, so a paper with an `\author` per author group kept only the last group (hep-ph9611287,
//! hep-ph9611291, hep-ph9602339, hep-ex9910019, hep-ex9608006, nucl-th9809087, gr-qc9806116, hep-ph9905201; mprocl
//! gr-qc9710030). The census of the shipped copies in the 92xx-99xx arXiv (704 papers, 203 variants of sprocl.sty; 59
//! papers, 23 of mprocl.sty; 42 papers, 10 of stwol.sty; cited from hep-ph9701265's sprocl.sty) shows one document
//! syntax: `\author` and `\address` append a line each to the title block (sprocl.sty:18-21, `\@aabuffer`), so an
//! address is that of the authors given since the one before, `\abstracts{…}` the abstract (:32-38),
//! `\citeup`/`\citelow` citations (:103-110), an `{arabiclist}` (:82-89), footnotes marked `\alph` (:65). Single
//! variants add more: `\collaboration` (1 of the 10 stwol variants, stwol.sty:23-24, a title-block line as `\author`'s;
//! hep-th9610159), and in sprocl `\authora` (= `\author`), `\id` (indexed text), `\CaP` (small caption text),
//! `\clearemptypage` (1 variant each; hep-ph9708375's family), bound as conveniences a document's own definition
//! replaces; `\pnasus`/`\jtb`/`\jeem` appear only commented out (hep-ph9708375's sprocl.sty:170-178), so they are left
//! unbound. The rest is page layout. Rust-only: Perl has no binding.
use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};
use latexml_package::prelude::*;

/// The styles' common definitions.
fn ws_procl_style() -> Result<()> {
  // sprocl.sty:18-21: an `\author` line holds one or more names (`\author{N. G. DESHPANDE, B. DUTTA}`); an `\address`
  // line goes to the authors given since the previous one.
  DefMacro!("\\author{}", "\\lx@add@authors@append{#1}");
  // The address is the authors' affiliation, as Perl takes iopart's `\address` (iopart_support.sty.ltxml:53
  // `\lx@add@affiliations{#1}`) and REVTeX's `\affiliation` (revtex4_support_sty.rs): one led by an author's mark
  // (`\author{A. ABLE$^a$, B. BAKER$^b$}\address{$^a$X}\address{$^b$Y}`; hep-lat9806004's
  // `\author{C. Alexandrou$^a$, A. Bori\c{c}i$^b$, …}\address{$^a$Department of Natural Sciences, …}`) goes to the
  // authors showing that mark, any other to the authors given since the previous one.
  DefMacro!("\\address{}", sub[(content)] {
    if leads_with_mark(&content) && !queued_group_shares_the_mark(&content) {
      Ok(Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![None, Some(content)]))
    } else {
      Ok(Invocation!(
        T_CS!("\\lx@add@affiliation"),
        vec![Some(mouth::tokenize_internal("annotate=new")), Some(content)]
      ))
    }
  });
  // sprocl.sty:32-38: the abstract, a centered minipage after the title block
  DefMacro!("\\abstracts{}", "\\lx@add@abstract{#1}");
  // sprocl.sty:103-110: `\cite` as a superscript, `\citelow` on the line; both cite
  Let!("\\citeup", "\\cite");
  Let!("\\citelow", "\\cite");
  // sprocl.sty:65: footnotes marked a, b, …
  DefMacro!("\\thefootnote", "\\alph{footnote}");
  // sprocl.sty:82-89: an arabic-numbered list
  RawTeX!(
    "\\newcounter{arabiclistc}\
     \\newenvironment{arabiclist}{\\setcounter{arabiclistc}{0}\\begin{list}{\\arabic{arabiclistc}}\
     {\\usecounter{arabiclistc}\\setlength{\\parsep}{0pt}\\setlength{\\itemsep}{0pt}}}{\\end{list}}"
  );
  // the additions of single variants (hep-ph9708375's sprocl.sty:153-168 and kin): conveniences the paper's copy may not
  // have, which yield to a document's own `\newcommand` of the name (`\newcommand{\id}{{\bf 1}}`), as a fallback
  // class's guesses do (`DefinitionOrigin::Fallback`)
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    // stwol.sty:23-24: a collaboration's name, a line of the title block among the authors (as REVTeX's
    // `\collaboration`)
    Let!("\\collaboration", "\\author");
    Let!("\\authora", "\\author");
    DefMacro!("\\id{}", "#1\\index{#1}");
    // (hep-ph9708375's family: `\def\CaP#1{\footnotesize\parindent=0pt #1}`, no group)
    DefMacro!("\\CaP{}", "\\footnotesize\\parindent=0pt #1");
    Let!("\\clearemptypage", "\\clearpage");
  }
  // sprocl.sty:112: sections and subsections numbered
  RawTeX!("\\setcounter{secnumdepth}{2}");
  Ok(())
}

/// `sprocl.sty`
pub mod sprocl_sty {
  use super::*;
  LoadDefinitions!({
    ws_procl_style()?;
  });
}

/// `mprocl.sty`
pub mod mprocl_sty {
  use super::*;
  LoadDefinitions!({
    ws_procl_style()?;
  });
}

/// `stwol.sty`
pub mod stwol_sty {
  use super::*;
  LoadDefinitions!({
    ws_procl_style()?;
  });
}
