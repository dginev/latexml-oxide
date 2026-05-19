use crate::prelude::*;

/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
/// Routes inline macro expansion (each ~960 B of .text) through one
/// runtime call. Engine bootstrap pays parse_prototype once per entry.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


/// DEP-19 helper for identity-1 `DefMacro!("\\cs{}", "#1")` macros — the
/// CS takes one mandatory arg and expands to it unchanged. Routes
/// inline macro expansion through a single runtime call.
fn def_macro_identity(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("#1");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}


#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4_support.sty.ltxml — 433 lines
  // Support macros for RevTeX4 class (APS journals)

  RequirePackage!("hyperref");
  RequirePackage!("natbib");
  RequirePackage!("revsymb");
  RequirePackage!("url");
  RequirePackage!("longtable");
  RequirePackage!("dcolumn");

  // 4.3 Title/Author
  DefMacro!("\\title[]{}", "\\@add@frontmatter{ltx:title}{#2}");
  DefMacro!("\\doauthor{}{}{}", "#1 #2 #3");
  DefMacro!("\\address", "\\affiliation");

  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affiliation{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");
  // `\altaffiliation[note]{address}` — REVTeX4 alternative-affiliation
  // construct with an OPTIONAL leading note (e.g. `[Also at ]`).
  // Without the `[]` arg in the signature the `[Also at ]` text was
  // mis-parsed: `\affiliation{}` greedily read `[` as `#1`, emitting
  // a bare literal `[` into `<ltx:contact role='affiliation'>` and
  // dumping the rest of the note into the author name slot.
  // Witness: physics0210041 (revtex4 `\altaffiliation[Also at ]{Dept of
  // Physics, University of Oslo, ...}`). Real LaTeX's revtex4
  // `\altaffiliation` takes `[note]{address}` and prepends note to
  // address. Concatenating `#1#2` matches that semantics; when there
  // is no optional `[]`, #1 is empty and behaviour matches the legacy
  // single-arg path. SURPASS-PERL: Perl LaTeXML
  // `revtex4_support.sty.ltxml` also lacks the optional arg and has
  // the same misformatting on this witness.
  DefMacro!("\\altaddress[]{}",     "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1#2}}");
  DefMacro!("\\altaffiliation[]{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1#2}}");
  DefMacro!("\\andname", "and");
  def_macro_noop("\\collaboration")?;
  def_macro_noop("\\noaffiliation")?;

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email [] Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#2}}");
  DefConstructor!("\\@@@homepage{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\homepage Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@homepage{#1}}");

  def_macro_noop("\\firstname")?;
  DefConstructor!("\\surname{}", "#1", enter_horizontal => true);

  // 4.4 Abstract
  DefMacro!("\\abstractname", "Abstract");

  // 4.5 PACS
  DefMacro!("\\pacs{}", "\\@add@frontmatter{ltx:classification}[scheme=pacs]{#1}");

  // 4.6 Keywords
  DefMacro!("\\keywords{}", "\\@add@frontmatter{ltx:keywords}{#1}");

  // 4.7 Preprint
  DefMacro!("\\preprint{}", "\\@add@frontmatter{ltx:note}[role=preprint]{#1}");

  // Extra
  def_macro_noop("\\blankaffiliation")?;
  DefMacro!("\\checkindate", "\\today");

  DefMacro!("\\received[]{}", "\\@add@frontmatter{ltx:date}[role=received]{#2}");
  DefMacro!("\\revised[]{}", "\\@add@frontmatter{ltx:date}[role=revised]{#2}");
  DefMacro!("\\accepted[]{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#2}");
  DefMacro!("\\published[]{}", "\\@add@frontmatter{ltx:date}[role=published]{#2}");

  // 5.3 Widetext
  def_macro_noop("\\widetext")?;
  def_macro_noop("\\endwidetext")?;
  def_macro_noop("\\narrowtext")?;
  def_macro_noop("\\endnarrowtext")?;
  def_macro_noop("\\mediumtext")?;
  def_macro_noop("\\endmediumtext")?;

  // 5.5 Acknowledgements — Perl revtex4_support.sty.ltxml L100-106.
  // Perl: DefConstructor('\acknowledgments', "<ltx:acknowledgements name='#name'>",
  //   properties => sub { (name => Digest(T_CS('\acknowledgmentsname'))); });
  Tag!("ltx:acknowledgements", auto_close => true);
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => { Ok(stored_map!("name" => stomach::digest(T_CS!("\\acknowledgmentsname"))?)) });
  // Tolerant close — see omnibus_cls.rs commentary on the
  // \begin{acknowledgments} ... \bibliography ... \end{acknowledgments}
  // pattern that auto-closes the env via \bibliography opening
  // <ltx:bibliography>. Driver: 2202.04803 R=1 → R=0.
  DefConstructor!("\\endacknowledgments", sub[document, _whatsit, _props] {
    let cur = document.get_node().clone();
    let has_open = document.findnode("ancestor-or-self::ltx:acknowledgements", Some(&cur)).is_some();
    if has_open {
      document.close_element("ltx:acknowledgements")?;
    }
  });
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\endacknowledgements", "\\endacknowledgments");

  // Section numbering style
  DefMacro!("\\thesection", "\\Roman{section}");

  // Grid / column macros
  DefMacro!("\\thepagegrid", "one");
  def_macro_noop("\\onecolumngrid")?;
  def_macro_noop("\\twocolumngrid")?;
  def_macro_noop("\\restorecolumngrid")?;
  // revtex4-1.cls L4388: \do@columngrid{layout}{N}. Layout-only.
  // Witness 2406.02666 (revtex4-1 with explicit \onecolumngrid call
  // before our stub binding loads).
  def_macro_noop("\\do@columngrid{}{}")?;
  DefPrimitive!("\\twocolumn", None);
  DefConstructor!("\\rotatebox{Number}{}", "#2", enter_horizontal => true);
  def_macro_noop("\\pagesofar")?;

  // Endnotes — Perl revtex4_support.sty.ltxml L120-149.
  // For each constructor: if optional arg #1 is present, mark = arg1;
  // else RefStepCounter('endnote') AND mark = DigestText(\theendnote).
  // The make_note_tags helper in latex_constructs.rs implements exactly
  // this dispatch (mark_opt + tag_opt=None branch).
  NewCounter!("endnote");
  DefConstructor!("\\endnote[]{}", "<ltx:note role='endnote' mark='#mark' xml:id='#id'>#tags#2</ltx:note>",
    mode => "internal_vertical",
    before_digest => { neutralize_font(); },
    properties => sub[args] {
      crate::engine::latex_constructs::make_note_tags("endnote", args[0].as_ref(), None)
    });
  DefConstructor!("\\endnotemark[]", "<ltx:note role='endnotemark' mark='#mark' xml:id='#id'>#tags</ltx:note>",
    mode => "restricted_horizontal", enter_horizontal => true,
    before_digest => { neutralize_font(); },
    properties => sub[args] {
      crate::engine::latex_constructs::make_note_tags("endnote", args[0].as_ref(), None)
    });
  DefConstructor!("\\endnotetext[]{}", "<ltx:note role='endnotetext' mark='#mark' xml:id='#id'>#2</ltx:note>",
    mode => "internal_vertical",
    before_digest => { neutralize_font(); },
    properties => sub[args] {
      // Perl L143-149: mark = arg1 OR Digest(\theendnote). (No RefStepCounter.)
      let arg1 = args[0].as_ref();
      let mark = match arg1 {
        Some(m) => m.clone(),
        None => stomach::digest(T_CS!("\\theendnote"))?,
      };
      Ok(stored_map!("mark" => mark))
    });

  // 6. Math — Perl L159-176
  Let!("\\case", "\\frac");
  Let!("\\slantfrac", "\\frac");
  // Perl L161-162 passes `locked => 1` so RevTeX's \text isn't silently
  // replaced by amsmath's \text (which would miss the restricted_hmode
  // treatment RevTeX relies on for inline-text-in-math spacing).
  DefConstructor!("\\text{}", "<ltx:text _noautoclose='true'>#1</ltx:text>",
    mode => "restricted_horizontal", locked => true);

  // RevTeX3 bold math (obsolete in RevTeX4) — Perl L165-171
  DefConstructor!("\\bm{}", "#1", bounded => true, require_math => true, font => { forcebold => true });
  // Perl L166-168: `locked => 1` keeps \bbox bold-wrapped even when a
  // user or co-loaded package redefines it.
  DefConstructor!("\\bbox{}", "#1", bounded => true, require_math => true,
    font => { forcebold => true }, locked => true);
  // Perl revtex4_support.sty.ltxml L169-171: \pmb wraps content in
  // forcebold + family=blackboard + series=medium + shape=upright.
  DefConstructor!("\\pmb{}", "#1", bounded => true, require_math => true,
    font => { forcebold => true, family => "blackboard",
      series => "medium", shape => "upright" });
  // Perl revtex4_support.sty.ltxml L172:
  //   DefMacro('\eqnum {}',
  //     '\lx@equation@settag{\edef\theequation{#2}\lx@make@tags{equation}}',
  //     locked => 1);
  // The Perl body has a known bug — `#2` is out-of-range for a 1-arg macro
  // (KNOWN_PERL_ERRORS.md #15) — so it always tags the equation with the
  // counter default and silently drops the user-supplied label. Rust's
  // empty body is semantically equivalent to Perl's broken effect (same
  // dropped-label outcome) without re-implementing the buggy `#2` lookup.
  // The `locked=>true` flag is independent of the body and IS load-bearing:
  // it prevents a downstream class (revtex3_support? a sibling APS .cls?)
  // from `\renewcommand`-ing \eqnum into something that re-introduces a
  // tag-conflict. Match Perl on the lock.
  DefMacro!("\\eqnum{}", "", locked => true);
  def_macro_noop("\\mathletters")?;
  def_macro_noop("\\endmathletters")?;

  // Citations
  DefMacro!("\\onlinecite", "\\citealp");
  Let!("\\textcite", "\\citet");
  // revtex4-1/4-2 substyle bbls reference internal cite helpers
  // (`\rev@citealp`, `\rev@citealpnum`, `\rev@citet`, `\rev@citemark`)
  // that are normally let-aliased from natbib. Map them to natbib
  // equivalents so .bbl files referencing them resolve cleanly.
  // Witness 2412.13042 (revtex4-2 + main.bbl using \rev@citealp).
  Let!("\\rev@citealp",     "\\citealp");
  Let!("\\rev@citealpnum",  "\\citealpnum");
  Let!("\\rev@citet",       "\\citet");
  Let!("\\rev@citenum",     "\\citenum");
  Let!("\\rev@citemark",    "\\citenum");

  // 8. Citations and References — Perl revtex4_support.sty.ltxml L190-204
  // RevTeX3; obsolete for RevTeX4 (but semi-implemented there). Should be a
  // simple environment, but tends to be misused, so define separately.
  DefConstructor!("\\references",
    "<ltx:bibliography xml:id='#id' bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
       <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>\
       <ltx:biblist>",
    before_digest => {
      crate::engine::latex_constructs::before_digest_bibliography()
    },
    after_digest => sub[whatsit] {
      crate::engine::latex_constructs::begin_bibliography(whatsit)?;
    },
    locked => true
  );
  DefConstructor!("\\endreferences",
    sub[document, _whatsit, _props] {
      document.maybe_close_element("ltx:biblist")?;
      document.maybe_close_element("ltx:bibliography")?;
    },
    locked => true
  );

  // 10. Tables — Perl revtex4_support.sty.ltxml L215-245.
  // {quasitable} re-Lets tabular → longtable inside its body so that
  // an embedded \begin{tabular}...\end{tabular} actually digests as
  // \longtable (which can break across pages). Without these Lets,
  // a quasitable degrades to a plain tabular and loses page-break
  // capability — the entire reason the env exists.
  DefEnvironment!("{ruledtabular}", "#body");
  DefEnvironment!("{quasitable}", "#body",
    before_digest => {
      Let!(T_CS!("\\begin{tabular}"), T_CS!("\\begin{longtable}"));
      Let!(T_CS!("\\end{tabular}"), T_CS!("\\end{longtable}"));
      Let!("\\tabular", "\\longtable");
      Let!("\\endtabular", "\\endlongtable");
    });
  def_macro_noop("\\squeezetable")?;
  DefMacro!("\\toprule", "\\hline\\hline");
  DefMacro!("\\colrule", "\\hline");
  DefMacro!("\\botrule", "\\hline\\hline");
  def_macro_noop("\\frstrut")?;
  def_macro_noop("\\lrstrut")?;
  Let!("\\tableftsep", "\\tabcolsep");
  Let!("\\tabmidsep", "\\tabcolsep");
  Let!("\\tabrightsep", "\\tabcolsep");
  Let!("\\tablenote", "\\footnote");
  Let!("\\tablenotemark", "\\footnotemark");
  Let!("\\tablenotetext", "\\footnotetext");
  Let!("\\tableline", "\\colrule");
  RawTeX!("\\newcolumntype{d}{D{.}{.}{-1}}");

  // Floats
  DefPrimitive!("\\printfigures", None);
  DefPrimitive!("\\printtables", None);
  def_macro_noop("\\oneapage")?;
  def_macro_noop("\\printendnotes")?;

  // Turnpage
  DefEnvironment!("{turnpage}", "#body");

  // Extra
  DefMacro!("\\MakeTextLowercase", "\\lowercase");
  DefMacro!("\\MakeTextUppercase", "\\uppercase");
  def_macro_noop("\\NoCaseChange")?;

  // Macro & control stubs — Perl L280-295
  def_macro_noop("\\absbox")?;
  def_macro_noop("\\addstuff{}{}")?;
  def_macro_noop("\\appdef{}{}")?;
  def_macro_noop("\\gappdef{}{}")?;
  def_macro_noop("\\prepdef{}{}")?;
  def_macro_noop("\\lineloop{}")?;
  def_macro_noop("\\loopuntil{}")?;
  def_macro_noop("\\loopwhile{}")?;
  def_macro_noop("\\traceoutput")?;
  def_macro_noop("\\tracingplain")?;
  def_macro_noop("\\removephantombox")?;
  def_macro_noop("\\removestuff")?;
  def_macro_noop("\\replacestuff{}{}")?;
  def_macro_noop("\\say[]")?;
  def_macro_noop("\\saythe[]")?;

  // i18n
  DefMacro!("\\copyrightname", "??");
  DefMacro!("\\journalname", "??");
  DefMacro!("\\lofname", "List of Figures");
  DefMacro!("\\lotname", "List of Tables");
  DefMacro!("\\notesname", "Notes");
  DefMacro!("\\numbername", "number");
  DefMacro!("\\ppname", "pp");
  DefMacro!("\\tocname", "Contents");
  DefMacro!("\\volumename", "volume");

  // Document info — Perl L309-316
  def_macro_identity("\\volumenumber{}")?;
  def_macro_identity("\\volumeyear{}")?;
  def_macro_identity("\\issuenumber{}")?;
  DefMacro!("\\bibinfo{}{}", "#2");
  DefMacro!("\\eprint{}", "eprint #1");
  def_macro_identity("\\eid{}")?;
  DefMacro!("\\startpage{}", "\\pageref{FirstPage}{#1}");
  DefMacro!("\\endpage", "\\pageref{LastPage}{#1}");

  // Extra stubs — Perl L319-323
  def_macro_noop("\\flushing")?;
  DefMacro!("\\triggerpar", "\\par");
  def_macro_noop("\\fullinterlineskip")?;
  // Perl L322: \footbox as box register (used by revtex footnote handling)
  RawTeX!("\\newbox\\footbox");
  DefRegister!("\\intertabularlinepenalty", Number(100));

  def_macro_noop("\\FL")?;
  def_macro_noop("\\FR")?;
  def_macro_noop("\\draft")?;
  def_macro_noop("\\tighten")?;

  // Journal abbreviations — Perl L336-365
  DefMacro!("\\ao", "Appl.~Opt.~");
  DefMacro!("\\ap", "Appl.~Phys.~");
  DefMacro!("\\apl", "Appl.~Phys.~Lett.~");
  DefMacro!("\\apj", "Astrophys.~J.~");
  DefMacro!("\\bell", "Bell Syst.~Tech.~J.~");
  DefMacro!("\\jqe", "IEEE J.~Quantum Electron.~");
  DefMacro!("\\assp", "IEEE Trans.~Acoust.~Speech Signal Process.~");
  DefMacro!("\\aprop", "IEEE Trans.~Antennas Propag.~");
  DefMacro!("\\mtt", "IEEE Trans.~Microwave Theory Tech.~");
  DefMacro!("\\iovs", "Invest.~Opthalmol.~Vis.~Sci.~");
  DefMacro!("\\jcp", "J.~Chem.~Phys.~");
  DefMacro!("\\jmo", "J.~Mod.~Opt.~");
  DefMacro!("\\josa", "J.~Opt.~Soc.~Am.~");
  DefMacro!("\\josaa", "J.~Opt.~Soc.~Am.~A ");
  DefMacro!("\\josab", "J.~Opt.~Soc.~Am.~B ");
  DefMacro!("\\jpp", "J.~Phys.~(Paris) ");
  DefMacro!("\\nat", "Nature (London) ");
  DefMacro!("\\oc", "Opt.~Commun.~");
  DefMacro!("\\ol", "Opt.~Lett.~");
  DefMacro!("\\pl", "Phys.~Lett.~");
  DefMacro!("\\pra", "Phys.~Rev.~A ");
  DefMacro!("\\prb", "Phys.~Rev.~B ");
  DefMacro!("\\prc", "Phys.~Rev.~C ");
  DefMacro!("\\prd", "Phys.~Rev.~D ");
  DefMacro!("\\pre", "Phys.~Rev.~E ");
  DefMacro!("\\prl", "Phys.~Rev.~Lett.~");
  DefMacro!("\\rmp", "Rev.~Mod.~Phys.~");
  DefMacro!("\\pspie", "Proc.~Soc.~Photo-Opt.~Instrum.~Eng.~");
  DefMacro!("\\sjqe", "Sov.~J.~Quantum Elecron.~");
  DefMacro!("\\vr", "Vision Res.~");

  // Internal macros — Perl L370-431
  def_macro_noop("\\@revmess{}{}")?;
  DefMacro!("\\@ptsize", "0");
  DefMacro!("\\@journal", "pra");

  // Document style options — Perl L372-393
  DefMacro!("\\ds@preprint", "\\global\\preprintstytrue \\def\\@ptsize{2}");
  def_macro_noop("\\ds@twoside")?;
  def_macro_noop("\\ds@draft")?;
  DefMacro!("\\ds@amsfonts", "\\@amsfontstrue");
  DefMacro!("\\ds@amssymb", "\\@amssymbolstrue");
  DefMacro!("\\ds@titlepage", "\\@titlepagefalse");
  def_macro_noop("\\ds@twocolumn")?;
  DefMacro!("\\ds@tighten", "\\@tightenlinestrue");
  DefMacro!("\\ds@floats", "\\@floatstrue");
  DefMacro!("\\ds@eqsecnum", "\\global\\secnumberstrue");
  DefMacro!("\\ds@pra", "\\def\\@journal{pra}");
  DefMacro!("\\ds@prb", "\\def\\@journal{prb}");
  DefMacro!("\\ds@prc", "\\def\\@journal{prc}");
  DefMacro!("\\ds@prd", "\\def\\@journal{prd}");
  DefMacro!("\\ds@pre", "\\def\\@journal{pre}");
  DefMacro!("\\ds@prl", "\\def\\@journal{prl}");
  DefMacro!("\\ds@josaa", "\\def\\@journal{josaa}");
  DefMacro!("\\ds@josab", "\\def\\@journal{josab}");
  DefMacro!("\\ds@aplop", "\\def\\@journal{aplop}");
  Let!("\\ds@manuscript", "\\ds@preprint");

  // newif stubs — Perl L396-414
  TeX!(r"
  \newif\ifpreprintsty \global\preprintstyfalse
  \newif\if@amsfonts  \@amsfontsfalse
  \newif\if@amssymbols  \@amssymbolsfalse
  \newif\if@titlepage  \@titlepagefalse
  \newif\if@tightenlines \@tightenlinesfalse
  \newif\if@floats \@floatsfalse
  \newif\ifsecnumbers \global\secnumbersfalse
  \@namedef{ds@11pt}{\def\@ptsize{1}}
  \@namedef{ds@12pt}{\def\@ptsize{2}}
  \@namedef{ds@aps}{\def\@society{aps}}
  \@namedef{ds@osa}{\def\@society{osa}}
  ");

  // Environment manipulation — Perl L425-430
  DefMacro!("\\replace@command{}{}", "\\global\\let#1#2 #1");
  def_macro_noop("\\replace@environment{}{}")?;
  def_macro_noop("\\glet@environment{}{}")?;
});
