//! NJDnatbib.sty — Wiley's renamed copy of natbib 8.31b (`\ProvidesPackage{NJDnatbib}`, NJDnatbib.sty:29), which
//! WileyNJD-v2/v5/MRM.cls load in natbib's place (WileyNJDv5.cls:6021-6085 `\usepackage[numbers,super,…]{NJDnatbib}`,
//! MRM.cls:2441-2442 `\usepackage[numbers,super]{NJDnatbib}`). A stand-in for the author's file (author_files.rs
//! `AUTHOR_FILE_STAND_INS`): it is natbib, so the natbib binding loads with the options the class gave the copy
//! ([`natbib_fork_options`]), and the paper's copy, read after it, only provides (OXIDIZED_DESIGN_DIVERGENCES #731):
//! read raw, its `\bibsection`'s `\@ifundefined{chapter}` took the `\chapter*` branch, heading the references
//! "Bibliography" with `bib.bibxN` ids (2609.08425, 2101.06297, 2211.16474, 2411.11493, 2206.08798, 2602.03829,
//! 2408.07575, 2506.14995).
//!
//! The defaults its copies change that the PDF shows, by version (the class-family rule's dispatch on a conflict):
//! - v5 (md5 277c41da: 2602.03829, 2408.07575, 2609.08425): a class without chapters heads the bibliography
//!   "REFERENCES" (:1050-1053 `\bmsection*{…{{REFERENCES}}…}`; xelatex 2609.08425; the reviewer's probe njd_direct_v5),
//!   an amsart one `\refname` (:1065);
//! - v2 and MRM (2fa96b7a: 2101.06297, 2211.16474; 6838c57e: 2411.11493, 2206.08798; 4b333a37: 2506.14995): natbib's
//!   own `\section*{\refname…}` (:1052; probe njd_direct_v2: pdflatex "References"; MRM.cls:3191's `\refname`
//!   "REFERENCES" is the class's);
//! - every version: the superscript citation carries its postnote inside, ", p. 5" (:366-368; natbib's switch
//!   `NATBIB_SUPER_POSTNOTE_INSIDE`).
//!
//! The version is v5's class (`\@ifclassloaded{WileyNJDv5}`) or the copy's own `\bibsection` set by `\bmsection`, v5's
//! marker; a class without chapters is decided by `\chapter`, as the copy's `\@ifundefined{chapter}` does
//! (`state::class_defines_chapter`): not by a `chapter` counter, which the Wiley classes declare without a `\chapter`
//! (WileyNJDv5.cls:5728, MRM.cls:2160 `\newcounter{chapter}[part]`), and not by LaTeXML's kernel `\chapter`, which
//! stays defined under the OmniBus base (`\@ifundefined{chapter}` was false under the WileyNJDv5 binding: 2609.08425).
use latexml_package::prelude::*;

/// A renamed natbib copy's options are natbib's (natbib.sty:234-285): the natbib binding loads next with each one
/// the class gives the copy. A natbib loaded already (a class binding's, or the document's) takes them as a repeat
/// load does, as natbib's options, with an Info (natbib_sty.rs `\lx@natbib@repeat@options`) — not dropped silently
/// (MRM's binding had loaded natbib optionless before MRM.cls's `[numbers,super]{NJDnatbib}`: 2506.14995's
/// author-year where its PDF has superscripts).
pub fn natbib_fork_options() -> Result<()> {
  DefMacro!(T_CS!("\\lx@natfork@options"), None, Tokens!(), scope => Some(Scope::Global));
  DeclareOption!(None, {
    Digest!("\\xdef\\lx@natfork@options{\\lx@natfork@options,\\CurrentOption}")?;
  });
  ProcessOptions!();
  let options: Vec<String> = Expand!(T_CS!("\\lx@natfork@options"))
    .to_string()
    .split(',')
    .map(|option| option.trim().to_string())
    .filter(|option| !option.is_empty())
    .collect();
  // (the copy's own options, which its `\ProcessOptions` takes in TeX, even after a natbib loaded already:
  // `natbib.sty:options_of_a_renamed_copy`, binding/content.rs `apply_new_options_on_reload`)
  AssignValue!("natbib.sty:options_of_a_renamed_copy" => true, Some(Scope::Global));
  let loaded = require_package("natbib", NewDefault!(RequireOptions, options => options));
  AssignValue!("natbib.sty:options_of_a_renamed_copy" => false, Some(Scope::Global));
  loaded
}

LoadDefinitions!({
  natbib_fork_options()?;
  AssignValue!("NATBIB_SUPER_POSTNOTE_INSIDE" => true, Some(Scope::Global));
  // (after the copy, which is read at the end of this load: its `\bibsection` tells v5)
  RawTeX!(r"\AtEndOfPackage{\lx@njd@bibsection@title}");
  DefPrimitive!("\\lx@njd@bibsection@title", {
    let class = |name: &str| has_meaning(&T_CS!(s!("\\ver@{name}.cls")));
    let marked = matches!(
      stand_in_copy_meaning("NJDnatbib.sty", &T_CS!("\\bibsection")),
      Some(Stored::Expandable(defn)) if matches!(&defn.expansion,
        Some(ExpansionBody::Tokens(body)) if body.unlist_ref().contains(&T_CS!("\\bmsection")))
    );
    let v5 = class("WileyNJDv5") || class("wileyNJDv5") || marked;
    if v5 && !class_defines_chapter() && !class("amsart") {
      DefMacro!(T_CS!("\\lx@bibliography@title"), None, Tokenize!("REFERENCES"), scope => Some(Scope::Global));
    }
  });
});
