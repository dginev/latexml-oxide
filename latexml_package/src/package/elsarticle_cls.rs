use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: elsarticle.cls.ltxml
  // Generally ignorable options
  for option in ["preprint", "final", "review", "5p", "3p", "1p",
    "12pt", "11pt", "10pt", "endfloat", "endfloats", "numafflabel",
    "doubleblind", "oneside", "twoside", "onecolumn", "twocolumn",
    "longtitle", "lefttitle", "centertitle", "reversenotenum",
    "nopreprintline", "symbold", "ussrhead", "nameyear",
    "doublespacing", "reviewcopy"].iter()
  {
    DeclareOption!(*option, None);
  }
  // Perl L28: times option pulls in txfonts
  DeclareOption!("times", {
    RequirePackage!("txfonts");
  });
  // Perl L30-32: flags for later conditional behaviour
  DeclareOption!("seceqn", { state::assign_value("@seceqn", 1i64, Scope::Global); });
  DeclareOption!("secthm", { state::assign_value("@secthm", 1i64, Scope::Global); });
  DeclareOption!("amsthm", { state::assign_value("@amsthm", 1i64, Scope::Global); });
  // Perl L33-35: natbib defaults
  DeclareOption!("authoryear", { state::assign_value("@biboptions", Stored::from("round,authoryear"), Scope::Global); });
  DeclareOption!("number", { state::assign_value("@biboptions", Stored::from("numbers"), Scope::Global); });
  DeclareOption!("numbers", { state::assign_value("@biboptions", Stored::from("numbers"), Scope::Global); });
  // Pass other options to article
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("elsart_support_core");
  RequirePackage!("fleqn");
  RequirePackage!("graphicx");
  RequirePackage!("pifont");
  // natbib with biboptions
  let natbib_opts_stored = state::lookup_value("@biboptions");
  let natbib_opts = match &natbib_opts_stored {
    Some(Stored::String(s)) => arena::with(*s, |s| s.to_string()),
    _ => "numbers".to_string(),
  };
  let natbib_opt_vec: Vec<String> = natbib_opts.split(',').map(|s| s.trim().to_string()).collect();
  RequirePackage!("natbib", options => natbib_opt_vec);
  RequirePackage!("hyperref");
  DefMacro!("\\biboptions{}", "\\setcitestyle{#1}");

  // Perl L58-67: override {enumerate}/{itemize} to accept optional arg
  DefEnvironment!("{enumerate}[]",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    mode => "internal_vertical", locked => true,
    properties => { begin_itemize("enumerate", Some("enum"), BeginItemizeOptions::default())? },
    before_digest_end => { Digest!("\\par")?; });
  DefEnvironment!("{itemize}[]",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    mode => "internal_vertical", locked => true,
    properties => { begin_itemize("itemize", Some("enum"), BeginItemizeOptions::default())? },
    before_digest_end => { Digest!("\\par")?; });

  // Newer elsarticle.cls (2018+) added {graphicalabstract} and {highlights}
  // for Elsevier journal submissions. Real templates wrap a TikZ figure or
  // bullet list inside these and Elsevier's typesetter renders separately
  // from the main body. For LaTeXML's HTML output, treat them as
  // semantically-tagged note blocks. Driver: 1907.06674.
  DefEnvironment!("{graphicalabstract}",
    "<ltx:note role='graphicalabstract'>#body</ltx:note>",
    mode => "internal_vertical", locked => true);
  DefEnvironment!("{highlights}",
    "<ltx:note role='highlights'>#body</ltx:note>",
    mode => "internal_vertical", locked => true);
});
