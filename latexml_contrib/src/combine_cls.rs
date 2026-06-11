//! Stub for combine.cls (multi-document combiner).
//!
//! combine.cls (TL `latex/combine/combine.cls`) is a class for
//! merging multiple LaTeX documents into a single PDF. It defines
//! `{papers}`, `{document*}`, etc. envs that wrap sub-document
//! content. Our HTML/XML output is single-document; just render the
//! body transparently. Witness 2310.14794.
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("article");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // combine.cls L760 redefines `\import` to take ONE arg (the
  // basename of a sub-document) and `\input`s `<arg>.tex`. This
  // conflicts with the `import` PACKAGE's `\import{<dir>/}{<file>}`
  // (two-arg form) so we must NOT RequirePackage{import} here —
  // instead define `\import` as combine.cls does. Witness 2310.14794.
  DefMacro!("\\import{}", "\\input{#1}");

  // Sub-documents inside `{papers}` have their own preambles —
  // `\documentclass` + `\usepackage` calls. Real combine.cls L194-209
  // saves and re-routes these via `\c@lbdocumentclass` / `\c@lbusepackage`
  // when wrapped under `\begin{papers}`. For HTML output we simply
  // discard them inside `{papers}` so the body content survives.
  // Use a flag toggled at env begin/end; redefine the kernel CS
  // to a conditional no-op only while `\ifc@lpapers` is true.
  RawTeX!(r"\newif\ifc@lpapers");
  DefMacro!(
    T_CS!("\\begin{papers}"),
    None,
    "\\c@lpaperstrue\\let\\c@l@orig@docclass\\documentclass\
     \\let\\c@l@orig@usepackage\\usepackage\
     \\def\\documentclass{\\c@lpapers@gobble@docclass}%
     \\def\\usepackage{\\c@lpapers@gobble@usepackage}"
  );
  DefMacro!(
    T_CS!("\\end{papers}"),
    None,
    "\\let\\documentclass\\c@l@orig@docclass\
     \\let\\usepackage\\c@l@orig@usepackage\
     \\c@lpapersfalse"
  );
  // Helpers gobble optional + mandatory args.
  DefMacro!("\\c@lpapers@gobble@docclass [] {}", None);
  DefMacro!("\\c@lpapers@gobble@usepackage [] {}", None);

  // combine.cls L942: `\newenvironment{papers}[1][\cleardoublepage]{...}{...}`
  // The body just contains `\subimport`'d sub-documents (or their
  // verbatim content). Render transparently. (Definitions added
  // below to also gobble inner \documentclass / \usepackage.)
  // \pretitle / \posttitle / \preauthor / \postauthor / \predate /
  // \postdate — combine.cls hooks for sub-document title pages.
  DefMacro!("\\pretitle{}", None);
  DefMacro!("\\posttitle{}", None);
  DefMacro!("\\preauthor{}", None);
  DefMacro!("\\postauthor{}", None);
  DefMacro!("\\predate{}", None);
  DefMacro!("\\postdate{}", None);
  // `\c@l...` family — combine.cls internal hooks. No-ops since we
  // don't combine multiple sub-documents.
  DefMacro!("\\c@lbusepackage[]{}", None);
  DefMacro!("\\c@lbLoadClass", None);
  DefMacro!("\\c@lbdocumentclass[]{}", None);
});
