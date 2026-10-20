//! Stub for combine.cls (multi-document combiner).
//!
//! combine.cls (TL `latex/combine/combine.cls`) is a class for
//! merging multiple LaTeX documents into a single PDF. It defines
//! `{papers}`, `{document*}`, etc. envs that wrap sub-document
//! content. Our HTML/XML output is single-document; just render the
//! body transparently, in the class combine's options name. Witness 2310.14794.
use latexml_package::prelude::*;

LoadDefinitions!({
  // combine.cls:29 `\c@lclass`, the class each document is typeset in (article, :85-88 `book`, `report`, `letter` or
  // `memoir`; `colclass=<class>`, :30-50), :105 every other option passed to it, :111 `\LoadClass{\c@lclass}`. The
  // binding loaded article whatever the option, so `\documentclass[book]{combine}`'s `\chapter` was undefined
  // (72b review; witness 2310.14794 is an article one). combine's own switches (:89-104) set what this binding does
  // not model. Guard `perfect_kernel_batch77v::combine_book_option_loads_book`.
  RawTeX!(r"\providecommand{\c@lclass}{article}");
  for class in ["book", "report", "letter", "memoir"] {
    DeclareOption!(class, {
      RawTeX!(&s!(r"\def\c@lclass{{{class}}}"));
    });
  }
  for option in [
    "classes",
    "packages",
    "layouts",
    "folios",
    "notoc",
    "nolof",
    "nolot",
    "maintoc",
    "date",
    "noauthor",
    "notitle",
    "nomaketitle",
    "nopubindoc",
    "nopubintoc",
    "onebib",
    "combinedbib",
  ] {
    DeclareOption!(option, None);
  }
  DeclareOption!(None, {
    let option = Expand!(T_CS!("\\CurrentOption")).to_string();
    if let Some(class) = option.strip_prefix("colclass=") {
      RawTeX!(&s!(r"\def\c@lclass{{{}}}", class.trim()));
    } else {
      let class = Expand!(T_CS!("\\c@lclass")).to_string();
      RawTeX!(&s!(r"\PassOptionsToClass{{\CurrentOption}}{{{class}}}"));
    }
  });
  ProcessOptions!();
  let class = Expand!(T_CS!("\\c@lclass")).to_string();
  LoadClass!(&class);
  // combine.cls (TeX Live's) loads no amsmath: amsmath's commands for papers that use them unloaded, its "Old form"
  // error once the paper loads it; the paper's copy decides (`amsmath_sty::require_amsmath_beyond_original`,
  // KNOWN_PERL_ERRORS #993).
  amsmath_sty::require_amsmath_beyond_original(
    latexml_package::shipped_class::shipped_copy_loads("cls", "amsmath")?,
    &[],
  )?;
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
