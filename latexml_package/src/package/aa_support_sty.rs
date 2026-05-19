//! aa_support.sty — Astronomy & Astrophysics journal support
//! Perl: aa_support.sty.ltxml — 469 lines
//! Shared by aa.cls and aa.sty
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Class options — Perl L26-45
  for option in ["10pt", "11pt", "12pt", "twoside", "onecolumn", "twocolumn",
    "draft", "final", "referee", "leqno", "fleqn", "longauth", "rnote",
    "oldversion", "runningheads", "envcountreset", "envcountsect",
    "structabstract", "traditabstract", "letter"].iter()
  {
    DeclareOption!(*option, None);
  }
  // Perl aa_support.sty.ltxml L35-36: openbib injects inline CSS rendering
  // bib blocks as display blocks. Prior Rust stub silently dropped the CSS
  // resource — port the require_resource pattern from book_cls.rs L33-43.
  DeclareOption!("openbib", {
    use latexml_core::document::resource::Resource;
    require_resource(Resource {
      mimetype: "text/css".into(),
      content: ".ltx_bibblock{display:block;}".into(),
      ..Resource::default()
    });
  });
  DeclareOption!("cm", { RequirePackage!("textcomp"); });
  DeclareOption!("bibnumber", { RequirePackage!("natbib"); });
  DeclareOption!("bibauthoryear", { RequirePackage!("natbib"); });
  // Perl aa_support.sty.ltxml L44: default options — ensures `bibauthoryear`
  // (which loads natbib) is executed even when user loads aa.cls without
  // options, matching Perl preamble semantics.
  Digest!("\\ExecuteOptions{a4paper,twocolumn,utf8,hideoverfull,bibauthoryear}")?;
  ProcessOptions!();

  // Dependencies — Perl L47-63
  RequirePackage!("inst_support");
  RequirePackage!("calc");
  RequirePackage!("etex");
  RequirePackage!("fontenc");
  RequirePackage!("geometry");
  RequirePackage!("setspace");
  RequirePackage!("fancyhdr");
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  // Pre-load xcolor with [dvipsnames, table] so user xcolor calls
  // don't silently option-clash and miss the colortbl / dvipsnam.def
  // loads.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("url");
  RequirePackage!("enumerate");
  RequirePackage!("longtable");
  RequirePackage!("xspace");
  RequirePackage!("babel");
  RequirePackage!("rotating");

  //======================================================================
  // The Manuscript Header — Perl L66-128
  //======================================================================

  DefMacro!("\\subtitle{}", "\\@add@frontmatter{ltx:subtitle}{#1}");

  // Structured abstract (5-arg or 1-arg) — Perl aa_support.sty.ltxml L73-93.
  // aa.cls's `\abstract` is either a single-arg traditional abstract OR a
  // 5-arg structured one (context / aims / methods / results / conclusions).
  DefMacro!("\\abstract@old{}", "\\@add@frontmatter{ltx:abstract}{#1}");
  DefMacro!("\\abstract@new{}{}{}{}{}",
    "\\@add@frontmatter{ltx:abstract}[name={\\abstractname}]{\
     \\ifx.#1.\\else\\textit{Context. }#1\\par\\fi\
     \\textit{Aims. }#2\\par\
     \\textit{Methods. }#3\\par\
     \\textit{Results. }#4\\par\
     \\ifx.#5.\\else\\textit{Conclusions. }#5\\fi}");
  // Perl L88-93: `\abstract{#1}` reads one arg, peeks for T_BEGIN; if present,
  // dispatches to \abstract@new (4 more args), else to \abstract@old. Our
  // previous always-1-arg stub dumped the remaining 4 `{…}` groups into the
  // document body, reordering the frontmatter/body (arxiv 1209.2771:
  // abstract paragraphs showed up BEFORE <title> and no <abstract> emitted).
  DefMacro!("\\abstract{}", sub[(arg1)] {
    gullet::skip_spaces()?;
    let next_is_begin = gullet::if_next(T_BEGIN!())?;
    let target = if next_is_begin { "\\abstract@new" } else { "\\abstract@old" };
    let mut out = vec![T_CS!(target), T_BEGIN!()];
    out.extend(arg1.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });

  // Keywords — Perl L95-96
  DefMacro!("\\keywordname", "\\sffamily\\bfseries Key Words.");
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}[name={\\keywordname}]{#1}");

  // Running title — Perl L99-104
  DefRegister!("\\titlerunning" => Tokens!());
  DefRegister!("\\authorrunning" => Tokens!());
  def_macro_noop("\\authrun")?;
  def_macro_noop("\\titrun")?;

  // Correspondence — Perl L107-121
  DefMacro!("\\offprints{}", "\\@add@frontmatter{ltx:note}[role=offprints]{#1}");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  DefConstructor!("\\@@@mail{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\mail Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@mail{#1}}");

  // aa_support: Perl L? gobbles \journalname; surpass with content
  // preservation — A&A papers set \journalname{Astronomy & Astrophysics}
  // and the value is genuine author metadata for the JATS pipeline.
  DefMacro!("\\journalname{}", "\\@add@frontmatter{ltx:note}[role=journal]{#1}");
  DefMacro!("\\rnotename", "(Research Note)");
  DefMacro!("\\rnotname", "(RN)");
  DefMacro!("\\headnote{}", "\\@add@frontmatter{ltx:note}{#1}");
  DefMacro!("\\dedication{}", "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  DefMacro!("\\mailname", "\\it Correspondence to \\/");
  DefMacro!("\\doi{}", "\\@add@frontmatter{ltx:classification}[scheme=doi]{#1}");
  DefMacro!("\\DOI{}", "\\@add@frontmatter{ltx:note}[role=doi]{#1}");
  DefMacro!("\\received{}", "\\@add@frontmatter{ltx:date}[role=received]{#1}");
  DefMacro!("\\accepted{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#1}");

  // \idline{vol}{page} — issue identification line; preserve as note.
  DefMacro!("\\idline{}{}", "\\@add@frontmatter{ltx:note}[role=idline]{#1, #2}");
  // \msnr{number} — manuscript number; preserve as note (author metadata).
  DefMacro!("\\msnr{}", "\\@add@frontmatter{ltx:note}[role=msnr]{#1}");
  def_macro_noop("\\institutename")?;
  def_macro_noop("\\hugehead")?;
  DefMacro!("\\AALogo", "Astronomy and Astrophysics");

  //======================================================================
  // Acknowledgements — Perl L132-140
  //======================================================================

  // Perl aa_support.sty.ltxml L132-138: \acknowledgements emits
  // <ltx:acknowledgements name='#name'> where #name is the digested
  // expansion of \acknowledgmentsname. Prior Rust port silently
  // dropped the `name=` attribute — documents using the A&A binding
  // and rendering the acknowledgement section in a tagset that
  // surfaces a `@name` attribute would miss the heading.
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements name='#name'>",
    properties => sub[_args] {
      let name = stomach::digest(T_CS!("\\acknowledgmentsname"))
        .map(|d| d.to_string()).unwrap_or_default();
      Ok(stored_map!("name" => name))
    });
  DefConstructor!("\\endacknowledgements", "</ltx:acknowledgements>");
  Let!("\\acknowledgement", "\\acknowledgements");
  Let!("\\endacknowledgement", "\\endacknowledgements");
  Tag!("ltx:acknowledgements", auto_close => true);
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  DefMacro!("\\ackname", "Acknowledgements");

  //======================================================================
  // Theorem environments — Perl L142-155
  //======================================================================

  RawTeX!("\\newtheorem*{proof}{Proof}");
  RawTeX!("\\@ifundefined{corollary}{\\newtheorem{corollary}[theorem]{Corollary}}{}");
  RawTeX!("\\@ifundefined{definition}{\\newtheorem{definition}[theorem]{Definition}}{}");
  RawTeX!("\\@ifundefined{example}{\\newtheorem{example}[theorem]{Example}}{}");
  RawTeX!("\\@ifundefined{exercise}{\\newtheorem{exercise}[theorem]{Exercise}}{}");
  RawTeX!("\\@ifundefined{lemma}{\\newtheorem{lemma}[theorem]{Lemma}}{}");
  RawTeX!("\\@ifundefined{note}{\\newtheorem{note}[theorem]{Note}}{}");
  RawTeX!("\\@ifundefined{problem}{\\newtheorem{problem}[theorem]{Problem}}{}");
  RawTeX!("\\@ifundefined{proposition}{\\newtheorem{proposition}[theorem]{Proposition}}{}");
  RawTeX!("\\@ifundefined{question}{\\newtheorem{question}[theorem]{Question}}{}");
  RawTeX!("\\@ifundefined{remark}{\\newtheorem{remark}[theorem]{Remark}}{}");
  RawTeX!("\\@ifundefined{solution}{\\newtheorem{solution}[theorem]{Solution}}{}");

  DefMacro!("\\noteaddname", "Note added in proof");
  // internal_vertical mode so the "note added in proof" body
  // (typically multi-paragraph prose) doesn't trip mode mismatch.
  DefEnvironment!("{noteadd}", "<ltx:note>#body</ltx:note>",
    mode => "internal_vertical");

  // \thesaurus — undocumented, ignorable — Perl L161
  def_macro_noop("\\thesaurus{}")?;

  //======================================================================
  // Equations — allow $ within equation env — Perl L164-200
  //======================================================================

  // Perl aa_support.sty.ltxml L164-200 redefines {equation} and {equation*}
  // to `Let(T_MATH, '\lx@dollar@in@mathmode')` — making a literal `$`
  // inside the equation body a no-op instead of closing display math. A&A
  // papers commonly use the idiom `… \text ~ $\rm text$` to mix roman
  // inline in equations, which would otherwise emit
  //   Error:expected:$ Missing $ closing display math.
  // for every occurrence (arxiv 0704.3480, 0707.0739, 0803.0466, 1103.2925
  // — all {aa} papers with inline `$` inside display math).
  //
  // We re-DefEnvironment here with the same template as the base in
  // engine::latex_constructs, but adding the Let in before_digest. The
  // original is `locked => true`, but re-DefEnvironment inside the
  // aa_support load path replaces it before any document body runs.
  use crate::engine::latex_constructs::{
    after_equation, before_equation, prepare_equation_counter,
  };
  DefEnvironment!(
    "{equation}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true));
      before_equation()?;
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    },
    locked => true);
  DefEnvironment!(
    "{equation*}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'><ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("preset" => true));
      before_equation()?;
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
    },
    locked => true);

  //======================================================================
  // Figures — Perl L202-218
  //======================================================================

  def_macro_noop("\\sidecaption")?;
  def_macro_noop("\\resetsubfig{}")?;
  def_macro_noop("\\subfigures")?;

  //======================================================================
  // Tables — Perl L220-231
  //======================================================================

  // Perl: DefMacro('\longtab{}', '') with comment "just let it do the table
  // contents as usual." Usage: `\longtab{N}{table-body}` — the {N} number is
  // consumed; the table-body block flows through as normal tokens. Perl does
  // NOT define a `{longtab}` environment, so we don't either; an extra
  // DefEnvironment trips the mode-switch on `\end{...}` matching.
  def_macro_noop("\\longtab{}")?;
  Let!("\\tablefoot", "\\footnote");
  DefMacro!("\\tablefootmark{}", "\\footnotemark[$#1$]");
  DefMacro!("\\tablefoottext{}{}", "\\footnotetext[$#1$]{#2}");
  DefMacro!("\\tablefont", "\\small");
  DefMacro!("\\tablenote{}{}", "\\footnote{#2}");
  DefMacro!("\\tablecaption{}", "\\caption{#1}");

  //======================================================================
  // Typography — Perl L238-323
  //======================================================================

  DefMacro!("\\vec{}", "\\ensuremath{\\mathbf{#1}}");
  DefMacro!("\\tens{}", "\\ensuremath{\\mathsf{#1}}");

  // Perl aa_support.sty.ltxml L241/L243: expose the internal \@vec and
  // \@tens DefMath entries that Perl's \vec / \tens would otherwise
  // forward to (`\vec → \ensuremath{\@vec{#1}}`). Rust short-circuits via
  // `\mathbf` / `\mathsf` above, but author code that writes `\@vec{}`
  // or `\@tens{}` directly (or third-party bindings that Let-alias to
  // them) hit undefined-CS without these. Add as additive DefMaths; the
  // `\vec`/`\tens` entry points above keep their current output shape.
  DefMath!("\\@vec{}",  "#1",            role => "ID", font => { forcebold => true });
  DefMath!("\\@tens{}", "\\mathsf{#1}",  role => "ID");

  // \ion{symbol}{ionization} — Perl L247
  DefMacro!("\\ion{}{}", "{#1 \\textsc{#2}}");

  // \element — Perl aa_support.sty.ltxml L250:
  //   DefMacro('\element[][][][]{}', '\ensuremath{\@element[#1][#2][#3][#4]{\mathrm{#5}}}')
  // signature is FOUR optional + ONE mandatory (charge/nucleons/protons/neutrons/symbol).
  // Prior Rust port `\element{}{}` (two mandatory) caused `\element{O}/...`
  // to greedily consume `/` as #2, leaving `$` unbalanced and producing
  // `Attempt to end mode 'math' in 'math'` on multiline math like
  // `$\element{O}/\element{C}\approx \element{C}/\element{He}$`
  // (e.g. astro-ph0605551). Fix matches Perl: optional bracket args plus
  // single mandatory symbol. The body simplification (drop `\@element`
  // Constructor) is intentional — the chemical-element XMArg wrapper is
  // not used for math-parser disambiguation.
  DefMacro!("\\element[][][][]{}", "\\ensuremath{{}^{#2}\\mathrm{#5}}");
  // Perl aa_support.sty.ltxml does NOT define \isotope (only \element). User
  // papers (e.g. 2011.10587) provide their own `\newcommand\isotope[2]{...}`
  // which our pre-definition would silently shadow, producing a math-mode
  // arg-consumption cascade. Stay Perl-faithful: don't define \isotope here.

  // Symbols — Perl L271-276
  DefPrimitive!("\\sun", "\u{2609}");
  DefPrimitive!("\\diameter", "\u{2300}");
  DefPrimitive!("\\degr", "\u{00B0}");
  DefPrimitive!("\\arcmin", "\u{2032}");
  DefPrimitive!("\\arcsec", "\u{2033}");

  // Relational operators — Perl L278-292
  DefMath!("\\la", "\u{2272}", role => "RELOP", meaning => "less-than-or-similar-to");
  DefMath!("\\ga", "\u{2273}", role => "RELOP", meaning => "greater-than-or-similar-to");
  DefMath!("\\cor", "\u{2258}", role => "RELOP", meaning => "corresponds-to");
  DefMath!("\\sol", "\u{2A9D}", role => "RELOP", meaning => "similar-to-or-less-than");
  DefMath!("\\sog", "\u{2A9E}", role => "RELOP", meaning => "similar-to-or-greater-than");
  DefMath!("\\lse", "\u{2A8D}", role => "RELOP", meaning => "less-than-or-similar-to-or-equal");
  DefMath!("\\gse", "\u{2A8E}", role => "RELOP", meaning => "greater-than-or-similar-to-or-equal");
  DefMath!("\\leogr", "\u{2276}", role => "RELOP", meaning => "less-than-or-greater-than");
  DefMath!("\\grole", "\u{2277}", role => "RELOP", meaning => "greater-than-or-less-than");
  DefMath!("\\loa", "\u{2A85}", role => "RELOP", meaning => "less-than-or-approximately-equals");
  DefMath!("\\goa", "\u{2A86}", role => "RELOP", meaning => "greater-than-or-approximately-equals");
  DefMath!("\\lid", "\u{2266}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\gid", "\u{2267}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\getsto", "\u{21C6}", role => "ARROW");

  // Fractional degrees/hours via aas@@fstack constructor — Perl L296-312
  // Ports aas_support.sty.ltxml's \aas@@fstack (semantic XMApp POSTFIX form)
  DefConstructor!("\\aas@@fstack{}",
    "<ltx:XMApp role='POSTFIX'><ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/><ltx:XMTok>.</ltx:XMTok><ltx:XMWrap>#1</ltx:XMWrap></ltx:XMApp>",
    mode => "math", bounded => true,
    properties => sub[_args] {
      let script_level = state::lookup_int("script_level");
      Ok(stored_map!("scriptpos" => s!("mid{}", script_level)))
    });
  DefMacro!("\\aas@fstack{}", "\\ensuremath{\\aas@@fstack{#1}}");
  DefMacro!("\\fd", "\\aas@fstack{d}");
  DefMacro!("\\fh", "\\aas@fstack{h}");
  DefMacro!("\\fm", "\\aas@fstack{m}");
  DefMacro!("\\fs", "\\aas@fstack{s}");
  // Perl aa_support.sty.ltxml L309-311: \fdg / \farcm / \farcs — additional
  // fractional-notation shortcuts (degree, arcminute, arcsecond) missing
  // from the Rust port. Add for astronomical-paper parity.
  DefMacro!("\\fdg", "\\aas@fstack{\\circ}");
  DefMacro!("\\farcm", "\\aas@fstack{\\prime}");
  DefMacro!("\\farcs", "\\aas@fstack{\\prime\\prime}");
  DefMacro!("\\fp", "\\aas@fstack{p}");
  DefMacro!("\\fdg", "\\aas@fstack{\\circ}");
  DefMacro!("\\farcm", "\\aas@fstack{\\prime}");
  DefMacro!("\\farcs", "\\aas@fstack{\\prime\\prime}");
  DefMacro!("\\udeg", "\\!^{\\circ}");
  DefMacro!("\\uarcmin", "\\!^{\\prime}");
  DefMacro!("\\uarcsec", "\\!^{\\prime\\prime}");

  // Perl L314-324: math small caps, QED square
  DefConstructor!("\\mathsc{}", "#1", bounded => true, require_math => true,
    font => { family => "smallcaps", series => "medium", shape => "upright" });
  DefConstructor!("\\squareforqed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})");
  Let!("\\sq", "\\squareforqed");
  Let!("\\qed", "\\squareforqed");

  // Blackboard bold — Perl L326-338
  DefPrimitive!("\\bbbc", "\u{2102}");
  DefPrimitive!("\\bbbf", "\u{1D53D}");
  DefPrimitive!("\\bbbh", "\u{210D}");
  DefPrimitive!("\\bbbk", "\u{1D542}");
  DefPrimitive!("\\bbbm", "\u{1D544}");
  DefPrimitive!("\\bbbn", "\u{2115}");
  DefPrimitive!("\\bbbone", "\u{1D7D9}");
  DefPrimitive!("\\bbbp", "\u{2119}");
  DefPrimitive!("\\bbbq", "\u{211A}");
  DefPrimitive!("\\bbbr", "\u{211D}");
  DefPrimitive!("\\bbbs", "\u{1D54A}");
  DefPrimitive!("\\bbbt", "\u{1D54B}");
  DefPrimitive!("\\bbbz", "\u{2124}");

  DefMacro!("\\sq", "\u{25A1}");
  DefMacro!("\\qed", "\u{220E}");

  //======================================================================
  // Object names — Perl L357-360
  //======================================================================

  DefConstructor!("\\object Semiverbatim",
    "<ltx:text class='ltx_ast_objectname' _noautoclose='1'>#1</ltx:text>");
  def_macro_noop("\\listofobjects")?;
  DefMacro!("\\listobjectname", "List of Objects");

  //======================================================================
  // Extra stuff — Perl L364-398
  //======================================================================

  def_macro_noop("\\setitemindent{}")?;
  def_macro_noop("\\setitemitemindent{}")?;
  DefMacro!("\\andname", "and");
  DefMacro!("\\lastandname", ", and");
  // \AASection{title} — A&A old-style section header. Surpass Perl
  // gobble: route to standard \section{} so the title is rendered.
  DefMacro!("\\AASection{}", "\\section{#1}");
  def_macro_noop("\\Online")?;

  DefRegister!("\\aftertext" => Dimension::new(5 * 65536));
  DefRegister!("\\betweenumberspace" => Dimension::new(218453));  // 3.33pt
  DefRegister!("\\figcapgap" => Dimension::new(5 * 65536));
  DefRegister!("\\tabcapgap" => Dimension::new(10 * 65536));
  DefRegister!("\\instindent" => Dimension::new(0));
  // Perl L379-384: aa cls dimension registers
  DefRegister!("\\figgap" => Dimension!("1cc"));
  DefRegister!("\\headerboxheight" => Dimension!("143pt"));
  DefRegister!("\\headlineindent" => Dimension!("1.166cm"));
  DefRegister!("\\logodepth" => Dimension!("1.3cm"));

  def_macro_noop("\\leftlegendglue")?;
  def_macro_noop("\\capstrut")?;
  def_macro_noop("\\captionstyle")?;
  def_macro_noop("\\clearelargs")?;
  def_macro_noop("\\errorref")?;
  DefMacro!("\\floatcounterend", ".");
  DefMacro!("\\sectcounterend", ".");
  DefMacro!("\\floatlegendstyle", "\\bf");
  def_macro_noop("\\thisbottomragged")?;
  DefMacro!("\\ts", "\\thinspace");
  DefMacro!("\\fnmsep", "\\unskip$^,$");
  def_macro_noop("\\makeheadbox")?;
  DefMacro!("\\tnote{}", "\\footnote{#1}");
  DefMacro!("\\at", "@");
  def_macro_noop("\\citeyearpar{}")?;

  // Perl L348-349: \bib@field@default@adsurl — ADS URL in bibitem.
  // Verbatim arg lets % survive (A&A adsurls often contain %26 = &).
  // Previously unported.
  DefConstructor!("\\bib@field@default@adsurl Verbatim",
    "<ltx:bib-url href='#1'>ADS entry</ltx:bib-url>");

  // \eprint — Perl L353
  DefMacro!("\\eprint[]{}", "{\\tt\\if!#1!#2\\else#1:#2\\fi}");

  // {stopref} environment — Perl L466
  DefEnvironment!("{stopref}", "#body");

  //======================================================================
  // Journal shorthands — Perl L404-463
  //======================================================================

  DefMacro!("\\aj",       "AJ");
  DefMacro!("\\actaa",    "Acta Astron.");
  DefMacro!("\\araa",     "ARA\\&A");
  DefMacro!("\\apj",      "ApJ");
  DefMacro!("\\apjl",     "ApJ");
  DefMacro!("\\apjs",     "ApJS");
  DefMacro!("\\ao",       "Appl.~Opt.");
  DefMacro!("\\apss",     "Ap\\&SS");
  DefMacro!("\\aap",      "A\\&A");
  DefMacro!("\\aapr",     "A\\&A~Rev.");
  DefMacro!("\\aaps",     "A\\&AS");
  DefMacro!("\\azh",      "AZh");
  DefMacro!("\\baas",     "BAAS");
  DefMacro!("\\bac",      "Bull. astr. Inst. Czechosl.");
  DefMacro!("\\caa",      "Chinese Astron. Astrophys.");
  DefMacro!("\\cjaa",     "Chinese J. Astron. Astrophys.");
  DefMacro!("\\icarus",   "Icarus");
  DefMacro!("\\jcap",     "J. Cosmology Astropart. Phys.");
  DefMacro!("\\jrasc",    "JRASC");
  DefMacro!("\\mnras",    "MNRAS");
  DefMacro!("\\memras",   "MmRAS");
  DefMacro!("\\na",       "New A");
  DefMacro!("\\nar",      "New A Rev.");
  DefMacro!("\\pasa",     "PASA");
  DefMacro!("\\pra",      "Phys.~Rev.~A");
  DefMacro!("\\prb",      "Phys.~Rev.~B");
  DefMacro!("\\prc",      "Phys.~Rev.~C");
  DefMacro!("\\prd",      "Phys.~Rev.~D");
  DefMacro!("\\pre",      "Phys.~Rev.~E");
  DefMacro!("\\prl",      "Phys.~Rev.~Lett.");
  DefMacro!("\\pasp",     "PASP");
  DefMacro!("\\pasj",     "PASJ");
  DefMacro!("\\qjras",    "QJRAS");
  DefMacro!("\\rmxaa",    "Rev. Mexicana Astron. Astrofis.");
  DefMacro!("\\skytel",   "S\\&T");
  DefMacro!("\\solphys",  "Sol.~Phys.");
  DefMacro!("\\sovast",   "Sov.~Ast.");
  DefMacro!("\\ssr",      "Space~Sci.~Rev.");
  DefMacro!("\\zap",      "ZAp");
  DefMacro!("\\nat",      "Nature");
  DefMacro!("\\iaucirc",  "IAU~Circ.");
  DefMacro!("\\aplett",   "Astrophys.~Lett.");
  DefMacro!("\\apspr",    "Astrophys.~Space~Phys.~Res.");
  DefMacro!("\\bain",     "Bull.~Astron.~Inst.~Netherlands");
  DefMacro!("\\fcp",      "Fund.~Cosmic~Phys.");
  DefMacro!("\\gca",      "Geochim.~Cosmochim.~Acta.");
  DefMacro!("\\grl",      "Geochim.~Res.~Lett.");
  DefMacro!("\\jcp",      "J.~Chem.~Phys.");
  DefMacro!("\\jgr",      "J.~Geophys.~Res.");
  DefMacro!("\\jqsrt",    "J.~Quant.~Spec.~Radiat.~Transf.");
  DefMacro!("\\memsai",   "Mem.~Soc.~Astron.~Italiana");
  DefMacro!("\\nphysa",   "Nucl.~Phys.~A");
  DefMacro!("\\physrep",  "Phys.~Rep");
  DefMacro!("\\physscr",  "Phys.~Scr");
  DefMacro!("\\planss",   "Planet.~Space~Sci.");
  DefMacro!("\\procspie", "Proc.~SPIE");
  DefMacro!("\\lrr",      "Living Rev. Relativity");
  Let!("\\astap", "\\aap");
  Let!("\\apjlett", "\\apjl");
  Let!("\\apjsupp", "\\apjs");
  Let!("\\applopt", "\\ao");
});
