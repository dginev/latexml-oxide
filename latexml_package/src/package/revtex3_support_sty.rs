use crate::prelude::*;

/// The option files of the REVTeX 3 distribution (v3.0 astro-ph9601167, v3.1 adap-org9608001) a paper may pass as
/// `\documentstyle` options.
const REVTEX3_OPTION_FILES: [&str; 11] = [
  "version2", "prabib", "prbbib", "osabib", "segbib", "aps10", "aps12", "osa10", "osa12", "seg10",
  "seg12",
];

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex3_support.sty.ltxml

  DeclareOption!("amsfonts", {
    RequirePackage!("amsfonts");
  });
  DeclareOption!("amssymb", {
    RequirePackage!("amssymb");
  });
  DeclareOption!("amsmath", {
    RequirePackage!("amsmath");
  });
  DeclareOption!("floats", {
    assign_value("revtex3@floats", true, Some(Scope::Global));
  });
  // The other options of REVTeX 3's revtex.sty (v3.0 <27 October 92>, astro-ph9601167; v3.1 <1 July 96>,
  // adap-org9608001), its `\ds@<option>`s, declared so that under `\documentstyle[aps,preprint]{revtex}` they are
  // REVTeX's and not left for `\compat@loadpackages`, whose missing option loads OmniBus over this binding (Perl
  // latex_constructs.pool.ltxml:137-153) and gives an `\address` after `\author{A and B}` to B alone
  // (hep-th9610203, cond-mat9403039). Rust-only: Perl declares none of them here. `11pt`, `12pt`, `twoside`,
  // `draft`, `titlepage` and `twocolumn` (revtex.sty v3.1:17-18, 30-44) are article.cls's options as well, which the
  // `\documentstyle` article underneath has used.
  //
  // The society (revtex.sty v3.1:24-27), which picks the substyle `\input \@society.sty` loads (:194).
  for society in ["aps", "osa", "seg"] {
    DeclareOption!(society, {
      assign_value("revtex3@society", society, Some(Scope::Global));
    });
  }
  // revtex.sty v3.1:21-22: the preprint layout, `\global\preprintstytrue`; `abstract` (:70-75, SEG) too, with
  // `\global\segabsstytrue`.
  for option in ["preprint", "manuscript"] {
    DeclareOption!(option, {
      assign_value("revtex3@preprint", true, Some(Scope::Global));
    });
  }
  DeclareOption!("abstract", {
    assign_value("revtex3@preprint", true, Some(Scope::Global));
    assign_value("revtex3@segabs", true, Some(Scope::Global));
  });
  // revtex.sty v3.1:53 `\global\secnumberstrue`: equations numbered within sections (aps.sty:358-365).
  DeclareOption!("eqsecnum", {
    assign_value("revtex3@eqsecnum", true, Some(Scope::Global));
  });
  // Layout only: `tighten` (:47, single spacing), `legalsize` (:76), and the journal (:55-68), which picks the
  // bibliography's typesetting (aps.sty:539-545 `prbbib`/`prabib`).
  for option in [
    "tighten", "legalsize",
    "pra", "prb", "prc", "prd", "pre", "prl", "josaa", "josab", "aplop", "geo", "tle",
  ] {
    DeclareOption!(option, None);
  }
  // Option files of the REVTeX 3 distribution, which a 2.09 `\documentstyle` option inputs after revtex.sty (latex.tex
  // `\@options`, for an option revtex.sty does not declare): `version2.sty`, the REVTeX 2 commands (its `{title}`,
  // `\author` and `{instit}` are the REVTeX 2.1 section below), `rmp.sty` (below), the bibliography and type-size
  // layouts `prabib`, `prbbib`, `osabib`, `segbib`, `aps10`, `aps12`, `osa10`,
  // `osa12`, `seg10`, `seg12`. Not in TeX Live; layout this binding does not emulate, so nothing, also when the paper
  // ships the file: raw over the binding, `rmp.sty`'s `\def\address` (hep-lat9608098, cond-mat9804141, hep-ph9610451)
  // dropped every address and added an error, which only OmniBus loaded after it used to cover.
  for option in REVTEX3_OPTION_FILES {
    DeclareOption!(option, None);
  }
  // `rmp.sty` (Rev. Mod. Phys.): otherwise title and heading layout, its headings referenced with dots, "II.A", "II.A.1"
  // (rmp.sty:140-145)
  DeclareOption!("rmp", {
    assign_value("revtex3@rmp", true, Some(Scope::Global));
  });
  ProcessOptions!();
  // aps.sty:614-630 (REVTeX 3.1): only with the `floats` option is `figure`
  // `\@float{figure}`; otherwise `\figure` is a list entry (`\vskip1pc`, the
  // caption type, "FIGURES" in preprint) that opens no TeX group, so a figure
  // ended by an enclosing `\endgroup` is clean in TeX (cond-mat9607130, whose
  // `\figure{N}` caption lines sit in `{references}`). `\table` keeps its
  // `\bgroup` either way (aps.sty:674-687). osa.sty:554-566 makes `\figure` a
  // list entry whatever the options. aipproc reaches this through its
  // `RequirePackage!("revtex3_support")` (aipproc_sty.rs:5).
  if !lookup_bool("revtex3@floats") || lookup_string("revtex3@society") == "osa" {
    for env in ["figure", "figure*"] {
      assign_value(
        &s!("\\begin{{{env}}}:tex_group"),
        TexGroup::None.as_str(),
        Some(Scope::Global),
      );
    }
  }

  RequirePackage!("revtex4_support");

  //======================================================================
  // Additional or different definitions from revtex4_support

  DefConditional!("\\ifpreprintsty");
  DefConditional!("\\ifsecnumbers");
  DefConditional!("\\ifsegabssty");
  // Perl revtex 3 internals — `\iffirstfig`/`\iffirsttab` control float
  // placement on the first page; revtex 3 papers commonly set
  // `\firstfigfalse`/`\firsttabfalse` BEFORE `\begin{document}` to bypass
  // the kludge. The Perl LaTeXML bindings for revtex don't pre-define
  // these, so users' `\firstfigfalse`/`\firsttabfalse` raise undefined-CS
  // errors. Witness: hep-th0109174, cond-mat0005077.
  DefConditional!("\\iffirstfig");
  DefConditional!("\\iffirsttab");
  // The switches the options set (revtex.sty v3.1:21-23, 53, 70-75).
  if lookup_bool("revtex3@preprint") {
    RawTeX!("\\global\\preprintstytrue");
  }
  if lookup_bool("revtex3@segabs") {
    RawTeX!("\\global\\segabsstytrue");
  }
  // aps.sty v3.1:965-968 (REVTeX 3.1, as chao-dyn9701025 ships it) `\preprint` defines `\@preprint` only under
  // `\ifpreprintsty`, and `\@maketitle` prints it first (:1006-1010), with no reset at the document's start: a number
  // is printed in the preprint style wherever it is called, and is metadata otherwise (revtex4_support_sty.rs
  // `\lx@revtex@preprint@printed`/`…@identifier`; OXIDIZED_DESIGN_DIVERGENCES #770).
  DefMacro!(
    "\\preprint",
    "\\ifpreprintsty\\expandafter\\lx@revtex@preprint@printed\\else\\expandafter\\lx@revtex@preprint@identifier\\fi"
  );
  // The substyle's numbering (aps.sty:220-227; seg.sty:247-253 the same without `\p@subsection`; osa.sty:195-202
  // with arabic sections): sections I, II, …, subsections A, B, … referenced with their section ("II A"),
  // subsubsections 1, 2, … ("II A 1").
  let society = lookup_string("revtex3@society");
  let rmp = lookup_bool("revtex3@rmp");
  RawTeX!(&s!(
    "\\def\\thesection{{{}}}\\def\\p@section{{}}\
     \\def\\thesubsection{{\\Alph{{subsection}}}}\\def\\p@subsection{{{}}}\
     \\def\\thesubsubsection{{\\arabic{{subsubsection}}}}\\def\\p@subsubsection{{{}}}\
     \\def\\theparagraph{{\\alph{{paragraph}}}}\\def\\thesubparagraph{{\\theparagraph.\\arabic{{subparagraph}}}}",
    if society == "osa" { "\\arabic{section}" } else { "\\Roman{section}" },
    if rmp {
      "\\thesection."
    } else if society == "seg" {
      ""
    } else {
      "\\thesection\\,"
    },
    if rmp { "\\thesection.\\thesubsection." } else { "\\thesection\\,\\thesubsection\\," }
  ));
  // aps.sty:358-365 (osa.sty:336-343, seg.sty:412-419): equations numbered `\theequation@prefix\arabic{equation}`;
  // under `eqsecnum` the counter is reset by each section and the prefix is the section's number in arabic, whatever
  // `\thesection` shows.
  RawTeX!("\\def\\theequation@prefix{}\\def\\theequation{\\theequation@prefix\\arabic{equation}}");
  if lookup_bool("revtex3@eqsecnum") {
    RawTeX!("\\global\\secnumberstrue\\@addtoreset{equation}{section}\\def\\theequation@prefix{\\arabic{section}.}");
  }
  // The appendix of the substyle's size file (aps12.sty:157-165, aps10.sty:178-186, osa12.sty:167-174): sections A,
  // B, …, subsections 1, 2, … ("A 1"), subsubsections a, b, …, equations A1, A2, … reset by each appendix (seg: A-1,
  // seg10.sty:205-221, seg12.sty:198-221). An unnumbered appendix steps the section counter all the same (aps.sty:280-295
  // `\@ssect`: `\ifappendixon\if@mainhead\stepcounter{section}`), so its equations are A1… too (cond-mat0006498,
  // chao-dyn9906002, gr-qc9803087, hep-th9605027).
  Let!("\\lx@revtex@saved@appendix", "\\appendix");
  RawTeX!(&s!(
    "\\def\\appendix{{\\lx@revtex@saved@appendix\\global\\appendixontrue\
     \\def\\thesubsection{{\\arabic{{subsection}}}}\\def\\thesubsubsection{{\\alph{{subsubsection}}}}\
     \\def\\theequation@prefix{{\\thesection{}}}\\@addtoreset{{equation}}{{section}}}}",
    if society == "seg" { "-" } else { "" }
  ));
  // (the appendix's `\section` is the kernel's `\@@appendix`, `\let` to it by `\appendix`)
  Let!("\\lx@revtex@saved@@appendix", "\\@@appendix");
  DefMacro!("\\@@appendix", "\\@ifstar\\lx@revtex@appendix@ssect\\lx@revtex@saved@@appendix");
  DefMacro!("\\lx@revtex@appendix@ssect", "\\stepcounter{section}\\lx@revtex@saved@@appendix*");
  RawTeX!("\\newif\\ifappendixon");

  def_macro_noop("\\eqsecnum")?;
  def_macro_noop("\\tightenlines")?;
  def_macro_noop("\\wideabs")?; // wide abstract — takes an arg, but avoid reading it

  // RevTeX's subequation numbering environment
  DefMacro!("\\mathletters",    "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endmathletters", "\\lx@equationgroup@subnumbering@end");

  //======================================================================
  // Revtex3 equation environments with $ faketext trick
  DefConditional!("\\if@lx@revtex@faketext@");
  DefConditional!("\\if@lx@revtex@nestmath@");
  DefMacro!("\\lx@revtex@faketext", "\\@lx@revtex@faketext@true\\hbox\\bgroup");
  DefMacro!("\\lx@revtex@nestmath", "\\@lx@revtex@nestmath@true\\lx@dollar@default");
  DefMacro!("\\lx@dollar@in@oldrevtex",
    "\\ifmmode\
       \\if@lx@revtex@nestmath@\\let\\@next\\lx@dollar@default\\else\\let\\@next\\lx@revtex@faketext\\fi\
     \\else\
       \\if@lx@revtex@faketext@\\let\\@next\\egroup\\else\\let\\@next\\lx@revtex@nestmath\\fi\
     \\fi\\@next");

  // The earlier latex_constructs.rs `\begin{equation}` is installed
  // with `locked => true`. Under Perl `loadLTXML`'s UNLOCK scope, the
  // redefinitions below override that lock — and Rust's `_load_binding`
  // now wraps the dispatcher in `local_state_unlocked_guard(true)`
  // (Package.pm:2318 parity), so no surgical lock-clear is needed.

  DefEnvironment!("{equation}",
    "<ltx:equation xml:id='#id'>\
     #tags\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    before_digest => {
      Let!(T_MATH!(), "\\lx@dollar@in@oldrevtex");
    },
    properties => { ref_step_counter_by_meaning("equation") },
    locked => true);

  DefEnvironment!("{equation*}",
    "<ltx:equation xml:id='#id'>\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    mode => "display_math",
    before_digest => {
      Let!(T_MATH!(), "\\lx@dollar@in@oldrevtex");
    },
    properties => { ref_step_id("equation") },
    locked => true);

  // Perl revtex3_support.sty.ltxml L90-101: DefConstructorI('\[', undef, ...,
  //   beforeDigest => sub { beginMode('display_math'); Let(T_MATH, '\lx@dollar@in@oldrevtex'); },
  //   captureBody => 1, properties => sub { RefStepID('equation') });
  DefConstructor!("\\[",
    "<ltx:equation xml:id='#id'>\
     <ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math>\
     </ltx:equation>",
    before_digest => {
      begin_mode("display_math")?;
      Let!(T_MATH!(), "\\lx@dollar@in@oldrevtex");
    },
    properties => { ref_step_id("equation") },
    capture_body => true);

  // RevTeX3 decimal-alignment macros for tabular columns.
  //   \setdec 0.000   sets the format (read up to space)
  //   \dec    1.234   aligns the value
  // Used by ~12 cluster papers (nucl-th0002021 etc.) under
  // \documentstyle[prc,aps]{revtex}. Perl LaTeXML's revtex3_support
  // also lacks these — surpass-Perl stub. Decimal alignment is
  // irrelevant for HTML; pass the value through, drop the format spec.
  RawTeX!("\\def\\setdec #1 {}");
  RawTeX!("\\def\\dec #1 {#1}");

  // \CITE — uppercase variant of \cite used in some revtex-era physics
  // papers (~11 cluster: cond-mat0003169, hep-ph0103298, etc.). Author
  // convention; not formally defined anywhere. Stub as alias to \cite
  // so the bib key is still resolved. Surpass-Perl (Perl LaTeXML also
  // errors here).
  Let!("\\CITE", "\\cite");

  //======================================================================
  // REVTeX 2.1 (revtex.sty v2.1, APS 1991): not in TeX Live, its source survives embedded in arXiv papers
  // (hep-lat9302012.tex:978-2231, cited below as `revtex.sty v2.1:<line of that file>`). Papers written for it,
  // `\documentstyle[preprint,revtex]{aps}` or `\documentstyle[version2,aps]{revtex}` (an option REVTeX 3.0's
  // revtex.sty does not declare), reach this binding through the `revtex` option or class. Rust-only: Perl LaTeXML has
  // none of these, and `\begin{instit}` stops each such paper (750 run-336 papers: hep-ph9301226, cond-mat9408057,
  // cond-mat9311041, hep-lat9402019).
  //
  // The title is the body of a `{title}` environment (revtex.sty v2.1:1340-1350), which `\begin{title}` otherwise
  // read as `\title`'s one-token argument (the title "I", cond-mat9311041), and an `{instit}` environment
  // (:1383-1389) after an `\author` is that author's institution: the affiliation of the authors before it, as
  // REVTeX 4's `\affiliation`.
  DefMacro!(T_CS!("\\begin{title}"), None, {
    Ok(environment_body_as_argument(T_CS!("\\title"), "title")?)
  });
  DefMacro!(T_CS!("\\end{title}"), None, "");
  DefMacro!(T_CS!("\\begin{instit}"), None, {
    Ok(environment_body_as_argument(T_CS!("\\affiliation"), "instit")?)
  });
  DefMacro!(T_CS!("\\end{instit}"), None, "");
  // revtex.sty v2.1:1374 `\let\moreauthors=\author`
  Let!("\\moreauthors", "\\author");
  // revtex.sty v2.1:1391: the received date, centred as "(Received #1)"
  DefMacro!("\\receipt{}", "\\lx@add@date[role=received,name={\\lx@date@received@name}]{#1}");
  // revtex.sty v2.1:1057-1058, 1064-1072: `\nonum` leaves the next section heading unnumbered (its
  // `\acknowledgments` is `\nonum\section{ACKNOWLEDGMENTS}`, :1123), the starred heading here; the paragraph ends and
  // column switches between them are kept (`\nonum\narrowtext\section`, cond-mat9303008; `\nonum` and a blank line,
  // hep-ph9206205).
  DefMacro!("\\nonum", {
    let headings = [T_CS!("\\section"), T_CS!("\\subsection"), T_CS!("\\subsubsection")];
    let between = [T_CS!("\\par"), T_CS!("\\narrowtext"), T_CS!("\\widetext"), T_CS!("\\mediumtext")];
    let mut read: Vec<Token> = Vec::new();
    while let Some(t) = read_token()? {
      read.push(t);
      if headings.contains(&t) {
        read.push(T_OTHER!("*"));
        break;
      }
      if t.get_catcode() != Catcode::SPACE && !between.contains(&t) {
        break;
      }
    }
    Ok(Tokens::new(read))
  });
  // revtex.sty v2.1:1858-1860 `\tablenotes`: a table's notes, set below it in small type
  DefMacro!("\\tablenotes{}", "\\par{\\footnotesize #1}");
});

/// `cs{<body>}`, the body of the environment `name` as the argument of `cs`: see [`read_environment_body`].
fn environment_body_as_argument(cs: Token, name: &str) -> Result<Tokens> {
  let mut call = vec![cs, T_BEGIN!()];
  call.extend(read_environment_body(name)?.unlist());
  call.push(T_END!());
  Ok(Tokens::new(call))
}

/// The tokens of the environment `name`'s body, up to its `\end{name}` (a nested `\end{...}` is body), which is put
/// back so that the environment ends as usual; without the spaces around it (the line ends after `\begin{title}` and
/// before `\end{title}`), which REVTeX's environments ignore (revtex.sty v2.1:1340-1350 `\ignorespaces`, `\par`).
fn read_environment_body(name: &str) -> Result<Tokens> {
  let mut body: Vec<Token> = Vec::new();
  while let Some(token) = read_token()? {
    if token.get_catcode() == Catcode::BEGIN {
      body.push(token);
      body.extend(read_balanced(ExpansionLevel::Off, false, false)?.unlist());
      body.push(T_END!());
    } else if token == T_CS!("\\end") {
      let env = read_arg(ExpansionLevel::Off)?;
      if env.to_string() == name {
        let mut end = vec![token, T_BEGIN!()];
        end.extend(env.unlist());
        end.push(T_END!());
        unread_vec(end);
        break;
      }
      body.push(token);
      body.push(T_BEGIN!());
      body.extend(env.unlist());
      body.push(T_END!());
    } else {
      body.push(token);
    }
  }
  while body
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    body.pop();
  }
  let lead = body
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  body.drain(..lead);
  Ok(Tokens::new(body))
}
