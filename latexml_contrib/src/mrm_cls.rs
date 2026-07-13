//! MRM.cls — Wiley "Magnetic Resonance in Medicine" journal class (the
//! WileyNJD author-template family). Not raw-loaded (a `.cls`) → the unbound
//! `\documentclass{MRM}` falls back to OmniBus, which supplies `\title`,
//! `\address`, the abstract and most of the `\org*` address helpers. This
//! binding adds only the Wiley frontmatter macros OmniBus leaves undefined so
//! the author block, ORCID, corresponding-author and funding lines render as
//! structured frontmatter instead of leaking. Witness 2509.13644.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // A real `.cls` binding owns its dependency loading (the unbound-class dep
  // scan is short-circuited once we bind MRM). MRM.cls pulls in the usual
  // AMS math stack, hyperref (for \href/\url — used by \orcid and body links)
  // and natbib (numeric \citen). Not xcolor (see the sn-jnl.cls option-conflict
  // note). Witness 2509.13644 (\text in the abstract, \href in \orcid).
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("hyperref");
  RequirePackage!("natbib");

  // \author[aff-idx]{name}{orcid} — Wiley's three-part author. Drop the numeric
  // affiliation index (its meaning is carried by the matching \address[idx]);
  // render the name as a creator and attach the ORCID as a contact.
  DefMacro!(
    "\\author[]{}{}",
    "\\lx@add@creator[role=author]{#2}\\lx@add@contact[role=orcid]{#3}"
  );
  // \orcid{id} — used as the third \author argument. Render the id as an
  // orcid.org hyperlink (matching sn-jnl.cls treatment); the class's logo
  // image isn't shipped, so the id stays as visible, linked text.
  DefMacro!("\\orcid{}", "\\href{https://orcid.org/#1}{#1}");
  // \authormark{...} — running-head author; redundant with the creator names.
  def_macro_noop("\\authormark{}")?;
  // \state{...} — a US-state address component. OmniBus deliberately omits it
  // (it collides with a \newcount \state in some classes); safe to pass through
  // here where MRM uses it purely as \orgaddress{\state{NY}, \country{USA}}.
  def_macro_identity("\\state{}")?;
  // \corres{text} — corresponding-author block (name, address, \email{...}).
  DefMacro!(
    "\\corres{}",
    "\\lx@add@frontmatter{ltx:note}[role=corresponding]{#1}"
  );
  // \finfo{text} — funding information, carrying \fundingAgency/\fundingNumber.
  DefMacro!(
    "\\finfo{}",
    "\\lx@add@frontmatter{ltx:note}[role=funding]{#1}"
  );
  def_macro_identity("\\fundingAgency{}")?;
  def_macro_identity("\\fundingNumber{}")?;
  // \citen{keys} — MRM's bare numeric citation (cite.sty convention); map to
  // \cite so references resolve rather than erroring. Witness 2509.13644.
  DefMacro!("\\citen{}", "\\cite{#1}");
  // \abstract[heading]{body} — Wiley abstract form with an optional heading.
  // Drop the heading (the standard "Abstract" label is emitted by the schema)
  // and route the body through the abstract machinery.
  DefMacro!("\\abstract[]{}", "\\lx@add@abstract{#2}");
});
