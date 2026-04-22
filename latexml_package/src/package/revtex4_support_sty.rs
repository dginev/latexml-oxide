use crate::prelude::*;

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
  DefMacro!("\\altaddress", "\\altaffiliation");
  DefMacro!("\\altaffiliation", "\\affiliation");
  DefMacro!("\\andname", "and");
  DefMacro!("\\collaboration", "");
  DefMacro!("\\noaffiliation", "");

  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email [] Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#2}}");
  DefConstructor!("\\@@@homepage{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\homepage Semiverbatim", "\\@add@to@frontmatter{ltx:creator}{\\@@@homepage{#1}}");

  DefMacro!("\\firstname", "");
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
  DefMacro!("\\blankaffiliation", "");
  DefMacro!("\\checkindate", "\\today");

  DefMacro!("\\received[]{}", "\\@add@frontmatter{ltx:date}[role=received]{#2}");
  DefMacro!("\\revised[]{}", "\\@add@frontmatter{ltx:date}[role=revised]{#2}");
  DefMacro!("\\accepted[]{}", "\\@add@frontmatter{ltx:date}[role=accepted]{#2}");
  DefMacro!("\\published[]{}", "\\@add@frontmatter{ltx:date}[role=published]{#2}");

  // 5.3 Widetext
  DefMacro!("\\widetext", "");
  DefMacro!("\\endwidetext", "");
  DefMacro!("\\narrowtext", "");
  DefMacro!("\\endnarrowtext", "");
  DefMacro!("\\mediumtext", "");
  DefMacro!("\\endmediumtext", "");

  // 5.5 Acknowledgements
  Tag!("ltx:acknowledgements", auto_close => true);
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements>");
  DefConstructor!("\\endacknowledgments", "</ltx:acknowledgements>");
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\endacknowledgements", "\\endacknowledgments");

  // Section numbering style
  DefMacro!("\\thesection", "\\Roman{section}");

  // Grid / column macros
  DefMacro!("\\thepagegrid", "one");
  DefMacro!("\\onecolumngrid", "");
  DefMacro!("\\twocolumngrid", "");
  DefMacro!("\\restorecolumngrid", "");
  DefPrimitive!("\\twocolumn", None);
  DefConstructor!("\\rotatebox{Number}{}", "#2", enter_horizontal => true);
  DefMacro!("\\pagesofar", "");

  // Endnotes — Perl L119-149
  NewCounter!("endnote");
  DefConstructor!("\\endnote[]{}", "<ltx:note role='endnote' mark='#mark' xml:id='#id'>#tags#2</ltx:note>",
    mode => "internal_vertical");
  DefConstructor!("\\endnotemark[]", "<ltx:note role='endnotemark' mark='#mark' xml:id='#id'>#tags</ltx:note>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefConstructor!("\\endnotetext[]{}", "<ltx:note role='endnotetext' mark='#mark' xml:id='#id'>#2</ltx:note>",
    mode => "internal_vertical");

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
  DefConstructor!("\\pmb{}", "#1", bounded => true, require_math => true, font => { forcebold => true });
  DefMacro!("\\eqnum{}", "");
  DefMacro!("\\mathletters", "");
  DefMacro!("\\endmathletters", "");

  // Citations
  DefMacro!("\\onlinecite", "\\citealp");
  Let!("\\textcite", "\\citet");

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

  // 10. Tables — Perl L215-245
  DefEnvironment!("{ruledtabular}", "#body");
  DefEnvironment!("{quasitable}", "#body");
  DefMacro!("\\squeezetable", "");
  DefMacro!("\\toprule", "\\hline\\hline");
  DefMacro!("\\colrule", "\\hline");
  DefMacro!("\\botrule", "\\hline\\hline");
  DefMacro!("\\frstrut", "");
  DefMacro!("\\lrstrut", "");
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
  DefMacro!("\\oneapage", "");
  DefMacro!("\\printendnotes", "");

  // Turnpage
  DefEnvironment!("{turnpage}", "#body");

  // Extra
  DefMacro!("\\MakeTextLowercase", "\\lowercase");
  DefMacro!("\\MakeTextUppercase", "\\uppercase");
  DefMacro!("\\NoCaseChange", "");

  // Macro & control stubs — Perl L280-295
  DefMacro!("\\absbox", "");
  DefMacro!("\\addstuff{}{}", "");
  DefMacro!("\\appdef{}{}", "");
  DefMacro!("\\gappdef{}{}", "");
  DefMacro!("\\prepdef{}{}", "");
  DefMacro!("\\lineloop{}", "");
  DefMacro!("\\loopuntil{}", "");
  DefMacro!("\\loopwhile{}", "");
  DefMacro!("\\traceoutput", "");
  DefMacro!("\\tracingplain", "");
  DefMacro!("\\removephantombox", "");
  DefMacro!("\\removestuff", "");
  DefMacro!("\\replacestuff{}{}", "");
  DefMacro!("\\say[]", "");
  DefMacro!("\\saythe[]", "");

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
  DefMacro!("\\volumenumber{}", "#1");
  DefMacro!("\\volumeyear{}", "#1");
  DefMacro!("\\issuenumber{}", "#1");
  DefMacro!("\\bibinfo{}{}", "#2");
  DefMacro!("\\eprint{}", "eprint #1");
  DefMacro!("\\eid{}", "#1");
  DefMacro!("\\startpage{}", "\\pageref{FirstPage}{#1}");
  DefMacro!("\\endpage", "\\pageref{LastPage}{#1}");

  // Extra stubs — Perl L319-323
  DefMacro!("\\flushing", "");
  DefMacro!("\\triggerpar", "\\par");
  DefMacro!("\\fullinterlineskip", "");
  // Perl L322: \footbox as box register (used by revtex footnote handling)
  RawTeX!("\\newbox\\footbox");
  DefRegister!("\\intertabularlinepenalty", Number(100));

  DefMacro!("\\FL", "");
  DefMacro!("\\FR", "");
  DefMacro!("\\draft", "");
  DefMacro!("\\tighten", "");

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
  DefMacro!("\\@revmess{}{}", "");
  DefMacro!("\\@ptsize", "0");
  DefMacro!("\\@journal", "pra");

  // Document style options — Perl L372-393
  DefMacro!("\\ds@preprint", "\\global\\preprintstytrue \\def\\@ptsize{2}");
  DefMacro!("\\ds@twoside", "");
  DefMacro!("\\ds@draft", "");
  DefMacro!("\\ds@amsfonts", "\\@amsfontstrue");
  DefMacro!("\\ds@amssymb", "\\@amssymbolstrue");
  DefMacro!("\\ds@titlepage", "\\@titlepagefalse");
  DefMacro!("\\ds@twocolumn", "");
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
  DefMacro!("\\replace@environment{}{}", "");
  DefMacro!("\\glet@environment{}{}", "");
});
