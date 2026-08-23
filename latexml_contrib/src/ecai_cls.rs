//! ecai.cls (ECAI / EurAI conference class).
//!
//! Covers the class's HTML-relevant frontmatter model (ecai.cls L392-851): its
//! reference-linked author/affiliation scheme, where `\author[<addr-refs>]{name}`
//! names each author and `\address[<label>]{text}` defines an affiliation, the
//! two connected by the shared label (like elsarticle). The rest of the class is
//! print layout (margins, fonts, two-column, page styles) with no HTML meaning
//! and is left to OmniBus/the kernel.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");
  RequirePackage!("url"); // ecai.cls L395: \RequirePackage{url}\urlstyle{rm}

  // --- Reference-linked author/affiliation model (ecai.cls L405-557) ---------
  // `\author[A,D]{\fnms{First}~\snm{Last}\orcid{..}\thanks{..}}` — one \author
  // per author, the optional arg a comma list of address labels. `\address[A]{..}`
  // defines affiliation "A". ecai defers the join to \makeaddresses at frontmatter
  // time; the kernel's annotation machinery does the same job eagerly: a creator
  // carries `annotations={A,D}` and each affiliation carries `label={A}`, and
  // `\lx@annotate@frontmatter` attaches every affiliation to the creator whose
  // annotations contain its label (identical to elsart_support_core). So author
  // "First Last" ends up with exactly affiliations A and D — not all of them piled
  // onto the last author, which is what the generic OmniBus `\author`/`\address`
  // produced. Witness 2501.13598 (html_feedback#6571: "Broken Affiliations"-class).
  DefMacro!(
    "\\author OptionalSemiverbatim {}",
    "\\lx@add@creator[role=author,annotations={#1}]{#2}"
  );
  DefMacro!(
    "\\address OptionalSemiverbatim {}",
    "\\lx@add@contact[label={#1},role=affiliation]{#2}"
  );
  // \fnms{first}/\snm{surname} are name-part hints (OmniBus already makes them
  // identity); keep them printing their argument inside the personname.
  def_macro_identity("\\fnms{}")?;
  def_macro_identity("\\snm{}")?;

  // ECAI frontmatter (ecai.cls L1290) — preserve paper ID as note.
  DefMacro!(
    "\\paperid{}",
    "\\@add@frontmatter{ltx:note}[role=paperid]{#1}"
  );
  def_macro_noop("\\makepaperid")?;
  // Deferred frontmatter builders (ecai.cls L657-697): the kernel emits the
  // creators/affiliations eagerly, so the class's \make* helpers are no-ops.
  def_macro_noop("\\makeauthors")?;
  def_macro_noop("\\makeaddresses")?;
  def_macro_noop("\\makeorcids")?;
  // ECAI authors use \orcid{<id>} inside \author{…} for the ORCID identifier
  // (ecai.cls renders it as \url{https://orcid.org/<id>} in the author block).
  // Route it through the kernel's `\lx@add@orcid`, which annotates the current
  // creator with an `ltx:contact[role=orcid]` — the XSLT then renders it as a
  // clickable `https://orcid.org/<id>` link (LaTeXML-structure-xhtml.xsl:767),
  // the same mechanism revtex4/quantumarticle use. A bare `ltx:note[role=orcid]`
  // only produced a dagger footnote with the raw number (html_feedback#6571).
  // Witness 2501.13598 (#6571), 2501.02040 + 3 ecai papers.
  DefMacro!("\\orcid{}", "\\lx@add@orcid{#1}");
  // {ack} environment — acknowledgments block. Emit as structural
  // ltx:acknowledgements (vs flattening into a generic section).
  // Witness 2408.16081.
  DefEnvironment!("{ack}", "<ltx:acknowledgements>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // \ecaisubmission — page-numbering toggle for submission mode. No-op
  // (ecai.cls L1100-ish flips internal `\if@ecai@subm` then issues
  // `\pagenumbering{arabic}\setcounter{page}{1}`). The visible effect
  // is page numbers in print; in HTML the page concept is meaningless.
  // Witness 2305.13804.
  def_macro_noop("\\ecaisubmission")?;
});
